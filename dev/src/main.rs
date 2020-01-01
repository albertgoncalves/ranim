use glutin_window::GlutinWindow as Window;
use graphics::{clear, math, rectangle, types, Transformed};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

const FRAME_WIDTH: f64 = 900.0;
const FRAME_HEIGHT: f64 = 550.0;
const HALF_FRAME_WIDTH: f64 = FRAME_WIDTH / 2.0;
const HALF_FRAME_HEIGHT: f64 = FRAME_HEIGHT / 2.0;

const LIGHT_GRAY: [f32; 4] = [0.965, 0.965, 0.965, 1.0];
const DARK_GRAY: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

const SQUARE_WIDTH: f64 = 150.0;
const MINUS_HALF_SQUARE_WIDTH: f64 = -1.0 * (SQUARE_WIDTH / 2.0);

/* NOTE: Animation state. */
pub struct State {
    gl: GlGraphics,
    rotation: f64,
}

impl State {
    fn render(&mut self, args: &RenderArgs) {
        let square: types::Rectangle =
            rectangle::square(0.0, 0.0, SQUARE_WIDTH);
        let rotation: f64 = self.rotation;
        self.gl.draw(args.viewport(), |c, gl| {
            let transform: math::Matrix2d = c
                .transform
                .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
            let offset_transform: math::Matrix2d = transform
                .trans(-1.0 * HALF_FRAME_WIDTH, -1.0 * HALF_FRAME_HEIGHT);
            clear(LIGHT_GRAY, gl);
            rectangle(
                DARK_GRAY,
                [0.0, 0.0, FRAME_WIDTH, FRAME_HEIGHT],
                offset_transform,
                gl,
            );
            rectangle(
                LIGHT_GRAY,
                square,
                transform
                    .rot_rad(rotation)
                    .trans(MINUS_HALF_SQUARE_WIDTH, MINUS_HALF_SQUARE_WIDTH),
                gl,
            );
        });
    }
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn update(&mut self, args: &UpdateArgs) {
        self.rotation += 1.0 * args.dt;
    }
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Window =
        WindowSettings::new("ranim", [FRAME_WIDTH, FRAME_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut state: State = State {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
    };
    let mut events: Events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            state.render(&args);
        }
        if let Some(args) = e.update_args() {
            state.update(&args);
        }
    }
}
