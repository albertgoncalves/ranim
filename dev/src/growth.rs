mod growth_lib;

use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use growth_lib::{Node, Point};
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
const WINDOW_RECT: [f64; 4] = [
    WINDOW_EDGE_HALF_MINUS,
    WINDOW_EDGE_HALF_MINUS,
    WINDOW_EDGE,
    WINDOW_EDGE,
];
const ANTI_ALIAS: u8 = 4;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

const LINE_WIDTH: f64 = 1.15;
const RADIUS: f64 = 3.0;
const RADIUS_2: f64 = RADIUS * 2.0;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF / 3.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;
const WALK_RNG_UPPER: f64 = 0.15;
const WALK_RNG_LOWER: f64 = -WALK_RNG_UPPER;

const RELOAD_FRAME_INTERVAL: u16 = 60 * 8;

fn render(gl: &mut GlGraphics, args: &RenderArgs, nodes: &[Node]) {
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(DARK_GRAY, WINDOW_RECT, transform, gl);
        for node in nodes {
            let x: f64 = node.point.x;
            let y: f64 = node.point.y;
            graphics::ellipse(
                LIGHT_GRAY,
                [x - RADIUS, y - RADIUS, RADIUS_2, RADIUS_2],
                transform,
                gl,
            );
            let left: &Point = &nodes[node.left_index].point;
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [left.x, left.y, x, y],
                transform,
                gl,
            )
        }
    })
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
    let uniform_init: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let uniform_walk: Uniform<f64> =
        Uniform::new_inclusive(WALK_RNG_LOWER, WALK_RNG_UPPER);
    let mut nodes: ArrayVec<[Node; growth_lib::NODES_CAP]> = ArrayVec::new();
    growth_lib::init(&mut rng, &uniform_init, &mut nodes);
    let mut counter: u16 = 0;
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if RELOAD_FRAME_INTERVAL < counter {
                nodes.clear();
                growth_lib::init(&mut rng, &uniform_init, &mut nodes);
                counter = 0;
            } else {
                for node in &mut nodes {
                    node.point.x += rng.sample(uniform_walk);
                    node.point.y += rng.sample(uniform_walk);
                }
                let index: usize = rng.gen_range(0, nodes.len() - 1);
                growth_lib::insert(&mut nodes, index);
            }
            render(&mut gl, &args, &nodes);
            counter += 1;
        }
    }
}
