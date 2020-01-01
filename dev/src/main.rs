extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

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

const FRAME_WIDTH: f64 = 800.0;
const FRAME_HEIGHT: f64 = 700.0;
const HALF_FRAME_WIDTH: f64 = FRAME_WIDTH / 2.0;
const HALF_FRAME_HEIGHT: f64 = FRAME_HEIGHT / 2.0;
const FRAME_RECT: [f64; 4] = [0.0, 0.0, FRAME_WIDTH, FRAME_HEIGHT];

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

const RNG_POINT_X_LOWER: f64 = 0.0;
const RNG_POINT_X_UPPER: f64 = FRAME_WIDTH;
const RNG_POINT_Y_LOWER: f64 = 0.0;
const RNG_POINT_Y_UPPER: f64 = FRAME_HEIGHT;

const RNG_RANGE_LOWER: f64 = -1.0;
const RNG_RANGE_UPPER: f64 = 1.0;

const LINE_WIDTH: f64 = 1.0;

const RELOAD: u16 = 60;

struct Point {
    x: f64,
    y: f64,
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
        }
    }
}

struct State {
    gl: GlGraphics,
    rng: ThreadRng,
    range: Uniform<f64>,
    counter: u16,
    a: Point,
    b: Point,
}

impl State {
    fn new(gl: GlGraphics, rng: ThreadRng) -> Self {
        Self {
            gl,
            rng,
            range: Uniform::new_inclusive(RNG_RANGE_LOWER, RNG_RANGE_UPPER),
            counter: 0,
            a: Point::new(rng),
            b: Point::new(rng),
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
        self.a = Point::new(self.rng);
        self.b = Point::new(self.rng);
    }

    fn update(&mut self) {
        if RELOAD < self.counter {
            self.reset();
        } else {
            self.counter += 1;
            self.a.x += self.rng.sample(self.range);
            self.a.y += self.rng.sample(self.range);
            self.b.x += self.rng.sample(self.range);
            self.b.y += self.rng.sample(self.range);
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        let line: [f64; 4] = [self.a.x, self.a.y, self.b.x, self.b.y];
        self.gl.draw(args.viewport(), |context, gl| {
            let transform: Matrix2d = context.transform.trans(
                (args.window_size[0] / 2.0) - HALF_FRAME_WIDTH,
                (args.window_size[1] / 2.0) - HALF_FRAME_HEIGHT,
            );
            graphics::clear(LIGHT_GRAY, gl);
            graphics::rectangle(DARK_GRAY, FRAME_RECT, transform, gl);
            graphics::line(LIGHT_GRAY, LINE_WIDTH, line, transform, gl);
        });
    }
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: GlutinWindow =
        WindowSettings::new("ranim", [FRAME_WIDTH, FRAME_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut state: State =
        State::new(GlGraphics::new(opengl), rand::thread_rng());
    let mut events: Events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            state.render(&args);
            state.update();
        }
    }
}
