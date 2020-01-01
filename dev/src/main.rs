use graphics::Transformed;
use piston::event_loop;
use piston::input::RenderEvent;
use piston::window;
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
    fn new(mut rng: rand::rngs::ThreadRng) -> Self {
        Self {
            x: rng.sample(rand::distributions::Uniform::new_inclusive(
                RNG_POINT_X_LOWER,
                RNG_POINT_X_UPPER,
            )),
            y: rng.sample(rand::distributions::Uniform::new_inclusive(
                RNG_POINT_Y_LOWER,
                RNG_POINT_Y_UPPER,
            )),
        }
    }
}

struct State {
    gl: opengl_graphics::GlGraphics,
    rng: rand::rngs::ThreadRng,
    range: rand::distributions::Uniform<f64>,
    counter: u16,
    a: Point,
    b: Point,
}

impl State {
    fn new(
        gl: opengl_graphics::GlGraphics,
        rng: rand::rngs::ThreadRng,
    ) -> Self {
        Self {
            gl,
            rng,
            range: rand::distributions::Uniform::new_inclusive(
                RNG_RANGE_LOWER,
                RNG_RANGE_UPPER,
            ),
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

    fn render(&mut self, args: &piston::input::RenderArgs) {
        let line: [f64; 4] = [self.a.x, self.a.y, self.b.x, self.b.y];
        self.gl.draw(args.viewport(), |context, gl| {
            let transform: graphics::math::Matrix2d = context.transform.trans(
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
    let opengl: opengl_graphics::OpenGL = opengl_graphics::OpenGL::V3_2;
    let mut window: glutin_window::GlutinWindow =
        window::WindowSettings::new("ranim", [FRAME_WIDTH, FRAME_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut state: State = State::new(
        opengl_graphics::GlGraphics::new(opengl),
        rand::thread_rng(),
    );
    let mut events: event_loop::Events =
        event_loop::Events::new(event_loop::EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            state.render(&args);
            state.update();
        }
    }
}
