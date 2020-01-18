mod webs_lib;

use webs_lib::{Edge, Node, Point, Rect};

use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
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
const CYAN: [f32; 4] = [0.5, 1.0, 0.87, 1.0];
const TEAL: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

const LINE_WIDTH: f64 = 0.8;
const RADIUS: f64 = 3.5;
const RADIUS_2: f64 = RADIUS * 2.0;
const RECT_PAD: f64 = 17.5;
const RECT_PAD_2: f64 = RECT_PAD * 2.0;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF;
const POINT_RNG_LOWER: f64 = WINDOW_EDGE_HALF_MINUS;

const NODES_LIMIT: usize = webs_lib::NODES_CAP - 2;
const EDGES_LIMIT: usize = webs_lib::EDGES_CAP - 3;

const INSERT_FRAME_INTERVAL: u16 = 10;

unsafe fn render(gl: &mut GlGraphics, args: &RenderArgs, edges: &[Edge]) {
    let n: usize = edges.len() - 1;
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(DARK_GRAY, WINDOW_RECT, transform, gl);
        {
            let edge: &Edge = &edges[n];
            let a: &Point = &(*edge.a).point;
            let b: &Point = &(*edge.b).point;
            let rect: Rect = webs_lib::bounds(a, b);
            graphics::rectangle(
                TEAL,
                [
                    rect.x - RECT_PAD,
                    rect.y - RECT_PAD,
                    rect.width + RECT_PAD_2,
                    rect.height + RECT_PAD_2,
                ],
                transform,
                gl,
            );
            graphics::line(
                CYAN,
                LINE_WIDTH,
                [a.x, a.y, b.x, b.y],
                transform,
                gl,
            );
            graphics::ellipse(
                CYAN,
                [a.x - RADIUS, a.y - RADIUS, RADIUS_2, RADIUS_2],
                transform,
                gl,
            );
            graphics::ellipse(
                CYAN,
                [b.x - RADIUS, b.y - RADIUS, RADIUS_2, RADIUS_2],
                transform,
                gl,
            );
        }
        for edge in edges.iter().take(n) {
            let a: &Point = &(*edge.a).point;
            let b: &Point = &(*edge.b).point;
            graphics::line(
                LIGHT_GRAY,
                LINE_WIDTH,
                [a.x, a.y, b.x, b.y],
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
    let mut nodes: ArrayVec<[Node; webs_lib::NODES_CAP]> = ArrayVec::new();
    let mut edges: ArrayVec<[Edge; webs_lib::EDGES_CAP]> = ArrayVec::new();
    let mut counter: u16 = 0;
    unsafe {
        webs_lib::init(&mut rng, &uniform, &mut nodes, &mut edges);
        while let Some(event) = events.next(&mut window) {
            if let Some(args) = event.render_args() {
                if (NODES_LIMIT < nodes.len()) || (EDGES_LIMIT < edges.len()) {
                    nodes.clear();
                    edges.clear();
                    webs_lib::init(&mut rng, &uniform, &mut nodes, &mut edges);
                } else if INSERT_FRAME_INTERVAL < counter {
                    webs_lib::insert(
                        &mut rng, &uniform, &mut nodes, &mut edges,
                    );
                    counter = 0;
                }
                webs_lib::update(&mut nodes);
                render(&mut gl, &args, &edges);
                counter += 1;
            }
        }
    }
}
