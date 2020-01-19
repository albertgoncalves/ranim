mod growth_lib;

use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use growth_lib::{Node, Point};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use sdl2_window::Sdl2Window;
use std::io;
use std::io::Write;

const WINDOW_EDGE: f64 = 800.0;
const WINDOW_EDGE_HALF: f64 = WINDOW_EDGE / 2.0;
const WINDOW_EDGE_HALF_MINUS: f64 = -WINDOW_EDGE_HALF;

const ANTI_ALIAS: u8 = 4;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const CYAN: [f32; 4] = [0.5, 1.0, 0.87, 0.5];

const LINE_WIDTH: f64 = 1.15;
const RADIUS: f64 = 4.0;
const RADIUS_2: f64 = RADIUS * 2.0;
const RADIUS_4: f64 = RADIUS * 4.0;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF / 3.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;
const WALK_RNG_UPPER: f64 = 0.15;
const WALK_RNG_LOWER: f64 = -WALK_RNG_UPPER;

const NEIGHBOR_RADIUS_SQUARED: f64 = 1000.0;
const SEARCH_RADIUS_SQUARED: f64 = 2000.0;

const DRAG_ATTRACT: f64 = 35.0;
const DRAG_REJECT: f64 = 25.0;

const BOUNDS: growth_lib::Bounds = growth_lib::Bounds {
    lower: Point {
        x: WINDOW_EDGE_HALF_MINUS,
        y: WINDOW_EDGE_HALF_MINUS,
    },
    upper: Point {
        x: WINDOW_EDGE_HALF,
        y: WINDOW_EDGE_HALF,
    },
};

fn render(gl: &mut GlGraphics, args: &RenderArgs, nodes: &[Node]) {
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(DARK_GRAY, gl);
        {
            let node: &Node = nodes.last().unwrap();
            graphics::ellipse(
                CYAN,
                [
                    node.point.x - RADIUS_2,
                    node.point.y - RADIUS_2,
                    RADIUS_4,
                    RADIUS_4,
                ],
                transform,
                gl,
            );
        }
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
    let mut nodes: ArrayVec<[Node; growth_lib::CAPACITY]> = ArrayVec::new();
    growth_lib::init_nodes(&mut rng, &uniform_init, &mut nodes);
    let mut frames: u16 = 0;
    let mut elapsed: f64 = 0.0;
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if growth_lib::NODES_CAP_LIMIT < nodes.len() {
                nodes.clear();
                growth_lib::init_nodes(&mut rng, &uniform_init, &mut nodes);
            } else {
                growth_lib::update_nodes(
                    &mut rng,
                    &uniform_walk,
                    &mut nodes,
                    BOUNDS,
                    NEIGHBOR_RADIUS_SQUARED,
                    SEARCH_RADIUS_SQUARED,
                    DRAG_ATTRACT,
                    DRAG_REJECT,
                );
            }
            render(&mut gl, &args, &nodes);
            frames += 1;
        }
        if let Some(args) = event.update_args() {
            elapsed += args.dt;
            if 1.0 < elapsed {
                print!("\r{}", f64::from(frames) / elapsed);
                io::stdout().flush().unwrap();
                frames = 0;
                elapsed = 0.0;
            }
        }
    }
    println!()
}
