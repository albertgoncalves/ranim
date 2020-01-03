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

const WINDOW_WIDTH: f64 = 400.0;
const WINDOW_HEIGHT: f64 = 400.0;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.35];

const LINE_WIDTH: f64 = 1.15;
const PAD: f64 = 10.0;
const PAD_2: f64 = PAD * 2.0;

const UPPER_BOUND: f64 = 300.0;
const LOWER_BOUND: f64 = -UPPER_BOUND;
const START_SPEED: f64 = 0.0;

const N: usize = 20;
const M: usize = N - 1;
const K: f64 = 0.015;
const L: f64 = 7.5;

const RELOAD: u16 = 60 * 8;

#[allow(clippy::comparison_chain)]
fn update(
    xs: &mut [f64],
    ys: &mut [f64],
    x_speeds: &mut [f64],
    y_speeds: &mut [f64],
) {
    for i in 0..N {
        for j in i..N {
            if xs[i] < xs[j] {
                x_speeds[i] += K;
                x_speeds[j] -= K;
            } else if xs[j] < xs[i] {
                x_speeds[i] -= K;
                x_speeds[j] += K;
            }
            if ys[i] < ys[j] {
                y_speeds[i] += K;
                y_speeds[j] -= K;
            } else if ys[j] < ys[i] {
                y_speeds[i] -= K;
                y_speeds[j] += K;
            }
        }
    }
    for i in 0..N {
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
            let x: f64 = xs[M];
            let y: f64 = ys[M];
            let x_speed: f64 = x - (x_speeds[M] * L);
            let y_speed: f64 = y - (y_speeds[M] * L);
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
                [min_x - PAD, min_y - PAD, width + PAD_2, height + PAD_2],
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
        for i in 0..M {
            let x: f64 = xs[i];
            let y: f64 = ys[i];
            let x_speed: f64 = x - (x_speeds[i] * L);
            let y_speed: f64 = y - (y_speeds[i] * L);
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
    let mut counter: u16 = RELOAD;
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> = Uniform::new_inclusive(LOWER_BOUND, UPPER_BOUND);
    let mut xs: [f64; N] = [0.0; N];
    let mut ys: [f64; N] = [0.0; N];
    let mut x_speeds: [f64; N] = [0.0; N];
    let mut y_speeds: [f64; N] = [0.0; N];
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            if RELOAD < counter {
                counter = 0;
                for i in 0..N {
                    xs[i] = rng.sample(range);
                    ys[i] = rng.sample(range);
                    x_speeds[i] = START_SPEED;
                    y_speeds[i] = START_SPEED;
                }
            } else {
                counter += 1;
                update(&mut xs, &mut ys, &mut x_speeds, &mut y_speeds);
            }
            render(&mut gl, &args, &xs, &ys, &x_speeds, &y_speeds);
        }
    }
}
