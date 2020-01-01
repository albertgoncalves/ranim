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

const FRAME_WIDTH: f64 = 800.0;
const FRAME_HEIGHT: f64 = 700.0;
const HALF_FRAME_WIDTH: f64 = FRAME_WIDTH / 2.0;
const HALF_FRAME_HEIGHT: f64 = FRAME_HEIGHT / 2.0;
const FRAME_RECT: [f64; 4] = [0.0, 0.0, FRAME_WIDTH, FRAME_HEIGHT];

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const LINE_WIDTH: f64 = 1.15;

const RNG_POINT_X_LOWER: f64 = 0.0;
const RNG_POINT_X_UPPER: f64 = FRAME_WIDTH;
const RNG_POINT_Y_LOWER: f64 = 0.0;
const RNG_POINT_Y_UPPER: f64 = FRAME_HEIGHT;
const START_SPEED: f64 = 0.0;

const N: usize = 20;
const K: f64 = 0.015;
const L: f64 = 7.5;

const RELOAD: u16 = 60 * 5;

macro_rules! init_array {
    ($t:ty, $n:expr, $f:expr $(,)*) => {{
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

impl Point {
    fn new(mut rng: ThreadRng) -> Self {
        Self {
            x: rng.sample(Uniform::new_inclusive(
                RNG_POINT_X_LOWER,
                RNG_POINT_X_UPPER,
            )),
            y: rng.sample(Uniform::new_inclusive(
                RNG_POINT_Y_LOWER,
                RNG_POINT_Y_UPPER,
            )),
            x_speed: START_SPEED,
            y_speed: START_SPEED,
        }
    }
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
        let transform: Matrix2d = context.transform.trans(
            (args.window_size[0] / 2.0) - HALF_FRAME_WIDTH,
            (args.window_size[1] / 2.0) - HALF_FRAME_HEIGHT,
        );
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(DARK_GRAY, FRAME_RECT, transform, gl);
        for point in points {
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
        WindowSettings::new("ranim", [FRAME_WIDTH, FRAME_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let rng: ThreadRng = rand::thread_rng();
    let mut counter: u16 = 0;
    let mut points: [Point; N] = init_array!(Point, N, || { Point::new(rng) });
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            render(&mut gl, &args, &points);
            if RELOAD < counter {
                counter = 0;
                for point in &mut points {
                    *point = Point::new(rng);
                }
            } else {
                counter += 1;
                update(&mut points)
            }
        }
    }
}
