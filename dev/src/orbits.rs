use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2_window::Sdl2Window;

const WINDOW_EDGE: f64 = 800.0;
const WINDOW_EDGE_HALF: f64 = WINDOW_EDGE / 2.0;
const WINDOW_EDGE_HALF_MINUS: f64 = -WINDOW_EDGE_HALF;

const ANTI_ALIAS: u8 = 4;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const TEAL: [f32; 4] = [0.17, 0.82, 0.76, 0.35];

const LINE_WIDTH: f64 = 1.15;
const RECT_PAD: f64 = 10.0;
const RECT_PAD_2: f64 = RECT_PAD * 2.0;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF;
const POINT_RNG_LOWER: f64 = WINDOW_EDGE_HALF_MINUS;

const SPEED_INIT: f64 = 0.0;
const SPEED_INCREMENT: f64 = 0.015;
const TRAIL: f64 = 7.5;

const CAPACITY: usize = 20;
const CAPACITY_MINUS_1: usize = CAPACITY - 1;

const RELOAD_FRAME_INTERVAL: u16 = 60 * 8;

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Orbiter {
    pos: Point,
    speed: Point,
}

unsafe fn update(orbiters: &mut [Orbiter]) {
    for i in 0..CAPACITY {
        for j in i..CAPACITY {
            let a: *mut Orbiter = &mut orbiters[i] as *mut Orbiter;
            let b: *mut Orbiter = &mut orbiters[j] as *mut Orbiter;
            if (*a).pos.x < (*b).pos.x {
                (*a).speed.x += SPEED_INCREMENT;
                (*b).speed.x -= SPEED_INCREMENT;
            } else if (*b).pos.x < (*a).pos.x {
                (*a).speed.x -= SPEED_INCREMENT;
                (*b).speed.x += SPEED_INCREMENT;
            }
            if (*a).pos.y < (*b).pos.y {
                (*a).speed.y += SPEED_INCREMENT;
                (*b).speed.y -= SPEED_INCREMENT;
            } else if (*b).pos.y < (*a).pos.y {
                (*a).speed.y -= SPEED_INCREMENT;
                (*b).speed.y += SPEED_INCREMENT;
            }
        }
    }
    for o in orbiters {
        o.pos.x += o.speed.x;
        o.pos.y += o.speed.y;
    }
}

fn render(gl: &mut GlGraphics, args: &RenderArgs, orbiters: &[Orbiter]) {
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(DARK_GRAY, gl);
        {
            let o: &Orbiter = &orbiters[CAPACITY_MINUS_1];
            let x: f64 = o.pos.x;
            let y: f64 = o.pos.y;
            let x_speed: f64 = x - (o.speed.x * TRAIL);
            let y_speed: f64 = y - (o.speed.y * TRAIL);
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
                TEAL,
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
        for o in orbiters.iter().take(CAPACITY_MINUS_1) {
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [
                    o.pos.x,
                    o.pos.y,
                    o.pos.x - (o.speed.x * TRAIL),
                    o.pos.y - (o.speed.y * TRAIL),
                ],
                transform,
                gl,
            );
        }
    });
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Sdl2Window =
        WindowSettings::new("ranim", [WINDOW_EDGE, WINDOW_EDGE])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .samples(ANTI_ALIAS)
            .vsync(true)
            .build()
            .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut orbiters: [Orbiter; CAPACITY] = [Orbiter {
        pos: Point { x: 0.0, y: 0.0 },
        speed: Point { x: 0.0, y: 0.0 },
    }; CAPACITY];
    let mut counter: u16 = RELOAD_FRAME_INTERVAL + 1;
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if RELOAD_FRAME_INTERVAL < counter {
                for o in &mut orbiters {
                    o.pos.x = rng.sample(uniform);
                    o.pos.y = rng.sample(uniform);
                    o.speed.x = SPEED_INIT;
                    o.speed.y = SPEED_INIT;
                }
                counter = 0;
            } else {
                unsafe {
                    update(&mut orbiters);
                }
                counter += 1;
            }
            render(&mut gl, &args, &orbiters);
        }
    }
}
