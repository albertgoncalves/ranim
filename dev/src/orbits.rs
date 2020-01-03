use glutin_window::GlutinWindow;
use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::mem;

const WINDOW_WIDTH: f64 = 400.0;
const WINDOW_HEIGHT: f64 = 400.0;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.35];

const LINE_WIDTH: f64 = 1.15;
const PAD: f64 = 10.0;
const PAD_2: f64 = PAD * 2.0;

const UPPER_BOUND: f64 = 300.0;
const LOWER_BOUND: f64 = -UPPER_BOUND;
const START_SPEED: f64 = 0.0;

const N: usize = 20;
const K: f64 = 0.015;
const L: f64 = 7.5;

const RELOAD: u16 = 60 * 8;

macro_rules! array {
    ($t:ty, $n:expr, $f:expr $(,)?) => {{
        let mut xs: [mem::MaybeUninit<$t>; $n] =
            unsafe { mem::MaybeUninit::uninit().assume_init() };
        for x in &mut xs[..] {
            *x = mem::MaybeUninit::new($f())
        }
        unsafe { mem::transmute::<_, [$t; $n]>(xs) }
    }};
}

struct Point {
    x: f64,
    y: f64,
    x_speed: f64,
    y_speed: f64,
}

macro_rules! point {
    ($r:expr, $u:expr, $s:expr $(,)?) => {
        Point {
            x: $r.sample($u),
            y: $r.sample($u),
            x_speed: $s,
            y_speed: $s,
        }
    };
}

#[allow(clippy::comparison_chain)]
fn update(points: &mut [Point]) {
    for i in 0..N {
        for j in i..N {
            if points[i].x < points[j].x {
                points[i].x_speed += K;
                points[j].x_speed -= K;
            } else if points[j].x < points[i].x {
                points[i].x_speed -= K;
                points[j].x_speed += K;
            }
            if points[i].y < points[j].y {
                points[i].y_speed += K;
                points[j].y_speed -= K;
            } else if points[j].y < points[i].y {
                points[i].y_speed -= K;
                points[j].y_speed += K;
            }
        }
    }
    for point in points.iter_mut() {
        point.x += point.x_speed;
        point.y += point.y_speed;
    }
}

fn render(gl: &mut GlGraphics, args: &RenderArgs, points: &[Point]) {
    gl.draw(args.viewport(), |context, gl| {
        let transform: Matrix2d = context
            .transform
            .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        graphics::clear(DARK_GRAY, gl);
        {
            let point: &Point = &points[N - 1];
            let x_speed: f64 = point.x - (point.x_speed * L);
            let y_speed: f64 = point.y - (point.y_speed * L);
            let (min_x, width): (f64, f64) = {
                if point.x < x_speed {
                    (point.x, x_speed - point.x)
                } else {
                    (x_speed, point.x - x_speed)
                }
            };
            let (min_y, height): (f64, f64) = {
                if point.y < y_speed {
                    (point.y, y_speed - point.y)
                } else {
                    (y_speed, point.y - y_speed)
                }
            };
            graphics::rectangle(
                CYAN,
                [min_x - PAD, min_y - PAD, width + PAD_2, height + PAD_2],
                transform,
                gl,
            );
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [point.x, point.y, x_speed, y_speed],
                transform,
                gl,
            );
        }
        for point in points.iter().take(N - 1) {
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [
                    point.x,
                    point.y,
                    point.x - (point.x_speed * L),
                    point.y - (point.y_speed * L),
                ],
                transform,
                gl,
            );
        }
    });
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: GlutinWindow =
        WindowSettings::new("ranim", [WINDOW_WIDTH, WINDOW_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut counter: u16 = 0;
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> = Uniform::new_inclusive(LOWER_BOUND, UPPER_BOUND);
    let mut points: [Point; N] =
        array!(Point, N, || point!(rng, range, START_SPEED));
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            render(&mut gl, &args, &points);
            if RELOAD < counter {
                counter = 0;
                for point in &mut points {
                    *point = point!(rng, range, START_SPEED);
                }
            } else {
                counter += 1;
                update(&mut points)
            }
        }
    }
}
