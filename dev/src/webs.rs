mod webs_lib;

use webs_lib::{Edge, Node, Point};

use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use sdl2_window::Sdl2Window;
use std::io;
use std::io::Write;

struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn make_rect(a: &webs_lib::Point, b: &webs_lib::Point) -> Rect {
    let x1: f64 = a.x;
    let x2: f64 = b.x;
    let y1: f64 = a.y;
    let y2: f64 = b.y;
    let (x, width): (f64, f64) = {
        if x1 < x2 {
            (x1, x2 - x1)
        } else {
            (x2, x1 - x2)
        }
    };
    let (y, height): (f64, f64) = {
        if y1 < y2 {
            (y1, y2 - y1)
        } else {
            (y2, y1 - y2)
        }
    };
    Rect {
        x,
        y,
        width,
        height,
    }
}

unsafe fn render(gl: &mut GlGraphics, args: &RenderArgs, edges: &[Edge]) {
    let n: usize = edges.len() - 1;
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(webs_lib::DARK_GRAY, gl);
        {
            let edge: &Edge = &edges[n];
            let a: &Point = &(*edge.a).point;
            let b: &Point = &(*edge.b).point;
            let rect: Rect = make_rect(a, b);
            graphics::rectangle(
                webs_lib::TEAL,
                [
                    rect.x - webs_lib::RECT_PAD,
                    rect.y - webs_lib::RECT_PAD,
                    rect.width + webs_lib::RECT_PAD_2,
                    rect.height + webs_lib::RECT_PAD_2,
                ],
                transform,
                gl,
            );
            graphics::line(
                webs_lib::CYAN,
                webs_lib::LINE_WIDTH,
                [a.x, a.y, b.x, b.y],
                transform,
                gl,
            );
            graphics::ellipse(
                webs_lib::CYAN,
                [
                    a.x - webs_lib::RADIUS,
                    a.y - webs_lib::RADIUS,
                    webs_lib::RADIUS_2,
                    webs_lib::RADIUS_2,
                ],
                transform,
                gl,
            );
            graphics::ellipse(
                webs_lib::CYAN,
                [
                    b.x - webs_lib::RADIUS,
                    b.y - webs_lib::RADIUS,
                    webs_lib::RADIUS_2,
                    webs_lib::RADIUS_2,
                ],
                transform,
                gl,
            );
        }
        for edge in edges.iter().take(n) {
            let a: &Point = &(*edge.a).point;
            let b: &Point = &(*edge.b).point;
            graphics::line(
                webs_lib::LIGHT_GRAY,
                webs_lib::LINE_WIDTH,
                [a.x, a.y, b.x, b.y],
                transform,
                gl,
            );
        }
    });
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Sdl2Window = WindowSettings::new(
        "ranim",
        [webs_lib::WINDOW_EDGE, webs_lib::WINDOW_EDGE],
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .samples(webs_lib::ANTI_ALIAS)
    .vsync(true)
    .build()
    .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f64> = Uniform::new_inclusive(
        webs_lib::POINT_RNG_LOWER,
        webs_lib::POINT_RNG_UPPER,
    );
    let mut nodes: ArrayVec<[Node; webs_lib::NODES_CAP]> = ArrayVec::new();
    let mut edges: ArrayVec<[Edge; webs_lib::EDGES_CAP]> = ArrayVec::new();
    let mut counter: u16 = 0;
    let mut frames: u16 = 0;
    let mut elapsed: f64 = 0.0;
    unsafe {
        webs_lib::init(&mut rng, &uniform, &mut nodes, &mut edges);
        while let Some(event) = events.next(&mut window) {
            if let Some(args) = event.render_args() {
                if (webs_lib::NODES_LIMIT < nodes.len())
                    || (webs_lib::EDGES_LIMIT < edges.len())
                {
                    nodes.clear();
                    edges.clear();
                    webs_lib::init(&mut rng, &uniform, &mut nodes, &mut edges);
                } else if webs_lib::INSERT_FRAME_INTERVAL < counter {
                    webs_lib::insert(
                        &mut rng, &uniform, &mut nodes, &mut edges,
                    );
                    counter = 0;
                }
                webs_lib::update(&mut nodes);
                render(&mut gl, &args, &edges);
                frames += 1;
                counter += 1;
            }
            if let Some(args) = event.update_args() {
                elapsed += args.dt;
                if 1.0 < elapsed {
                    print!("\r{:>7.2} fps", f64::from(frames) / elapsed);
                    io::stdout().flush().unwrap();
                    frames = 0;
                    elapsed = 0.0;
                }
            }
        }
    }
    println!()
}
