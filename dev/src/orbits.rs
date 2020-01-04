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

const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 500.0;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.35];

const LINE_WIDTH: f64 = 1.15;
const RECT_PAD: f64 = 10.0;
const RECT_PAD_2: f64 = RECT_PAD * 2.0;

const POINT_RNG_UPPER: f64 = 450.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;
const POINT_SPEED_INIT: f64 = 0.0;

const CAPACITY: usize = 20;
const CAPACITY_MINUS_1: usize = CAPACITY - 1;
const SPEED_INCREMENT: f64 = 0.015;
const RENDER_SCALE: f64 = 7.5;

const RELOAD_FRAME_INTERVAL: u16 = 60 * 8;

#[allow(clippy::comparison_chain)]
fn update(
    xs: &mut [f64],
    ys: &mut [f64],
    x_speeds: &mut [f64],
    y_speeds: &mut [f64],
) {
    for i in 0..CAPACITY {
        for j in i..CAPACITY {
            if xs[i] < xs[j] {
                x_speeds[i] += SPEED_INCREMENT;
                x_speeds[j] -= SPEED_INCREMENT;
            } else if xs[j] < xs[i] {
                x_speeds[i] -= SPEED_INCREMENT;
                x_speeds[j] += SPEED_INCREMENT;
            }
            if ys[i] < ys[j] {
                y_speeds[i] += SPEED_INCREMENT;
                y_speeds[j] -= SPEED_INCREMENT;
            } else if ys[j] < ys[i] {
                y_speeds[i] -= SPEED_INCREMENT;
                y_speeds[j] += SPEED_INCREMENT;
            }
        }
    }
    for i in 0..CAPACITY {
        xs[i] += x_speeds[i];
        ys[i] += y_speeds[i];
    }
}

fn render(
    gl: &mut GlGraphics,
    args: &RenderArgs,
    xs: &[f64],
    ys: &[f64],
    x_speeds: &[f64],
    y_speeds: &[f64],
) {
    gl.draw(args.viewport(), |context, gl| {
        let transform: Matrix2d = context
            .transform
            .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        graphics::clear(DARK_GRAY, gl);
        {
            let x: f64 = xs[CAPACITY_MINUS_1];
            let y: f64 = ys[CAPACITY_MINUS_1];
            let x_speed: f64 = x - (x_speeds[CAPACITY_MINUS_1] * RENDER_SCALE);
            let y_speed: f64 = y - (y_speeds[CAPACITY_MINUS_1] * RENDER_SCALE);
            let (min_x, width): (f64, f64) = {
                if x < x_speed {
                    (x, x_speed - x)
                } else {
                    (x_speed, x - x_speed)
                }
            };
            let (min_y, height): (f64, f64) = {
                if y < y_speed {
                    (y, y_speed - y)
                } else {
                    (y_speed, y - y_speed)
                }
            };
            graphics::rectangle(
                CYAN,
                [
                    min_x - RECT_PAD,
                    min_y - RECT_PAD,
                    width + RECT_PAD_2,
                    height + RECT_PAD_2,
                ],
                transform,
                gl,
            );
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [x, y, x_speed, y_speed],
                transform,
                gl,
            );
        }
        for i in 0..CAPACITY_MINUS_1 {
            let x: f64 = xs[i];
            let y: f64 = ys[i];
            let x_speed: f64 = x - (x_speeds[i] * RENDER_SCALE);
            let y_speed: f64 = y - (y_speeds[i] * RENDER_SCALE);
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [x, y, x_speed, y_speed],
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
    let mut counter: u16 = RELOAD_FRAME_INTERVAL + 1;
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut xs: [f64; CAPACITY] = [0.0; CAPACITY];
    let mut ys: [f64; CAPACITY] = [0.0; CAPACITY];
    let mut x_speeds: [f64; CAPACITY] = [0.0; CAPACITY];
    let mut y_speeds: [f64; CAPACITY] = [0.0; CAPACITY];
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if RELOAD_FRAME_INTERVAL < counter {
                counter = 0;
                for i in 0..CAPACITY {
                    xs[i] = rng.sample(range);
                    ys[i] = rng.sample(range);
                    x_speeds[i] = POINT_SPEED_INIT;
                    y_speeds[i] = POINT_SPEED_INIT;
                }
            } else {
                counter += 1;
                update(&mut xs, &mut ys, &mut x_speeds, &mut y_speeds);
            }
            render(&mut gl, &args, &xs, &ys, &x_speeds, &y_speeds);
        }
    }
}
