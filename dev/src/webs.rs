#![allow(clippy::cast_lossless)]

mod webs_lib;

use webs_lib::{Edge, Node, Point};

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
use std::io;
use std::io::Write;
use std::time::Instant;

struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

macro_rules! make_rect {
    ($x1:expr, $y1:expr, $x2:expr, $y2:expr $(,)?) => {{
        let (x, width): (f64, f64) = {
            if $x1 < $x2 {
                ($x1, $x2 - $x1)
            } else {
                ($x2, $x1 - $x2)
            }
        };
        let (y, height): (f64, f64) = {
            if $y1 < $y2 {
                ($y1, $y2 - $y1)
            } else {
                ($y2, $y1 - $y2)
            }
        };
        Rect {
            x,
            y,
            width,
            height,
        }
    }};
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
            let a_x: f64 = a.x as f64;
            let a_y: f64 = a.y as f64;
            let b_x: f64 = b.x as f64;
            let b_y: f64 = b.y as f64;
            let rect: Rect = make_rect!(a_x, a_y, b_x, b_y);
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
                [a_x, a_y, b_x, b_y],
                transform,
                gl,
            );
            graphics::ellipse(
                webs_lib::CYAN,
                [
                    a_x - webs_lib::RADIUS,
                    a_y - webs_lib::RADIUS,
                    webs_lib::RADIUS_2,
                    webs_lib::RADIUS_2,
                ],
                transform,
                gl,
            );
            graphics::ellipse(
                webs_lib::CYAN,
                [
                    b_x - webs_lib::RADIUS,
                    b_y - webs_lib::RADIUS,
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
            let a_x: f64 = a.x as f64;
            let a_y: f64 = a.y as f64;
            let b_x: f64 = b.x as f64;
            let b_y: f64 = b.y as f64;
            graphics::line(
                webs_lib::LIGHT_GRAY,
                webs_lib::LINE_WIDTH,
                [a_x, a_y, b_x, b_y],
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
    let uniform: Uniform<f32> = Uniform::new_inclusive(
        webs_lib::POINT_RNG_LOWER,
        webs_lib::POINT_RNG_UPPER,
    );
    let mut nodes: ArrayVec<[Node; webs_lib::NODES_CAP]> = ArrayVec::new();
    let mut edges: ArrayVec<[Edge; webs_lib::EDGES_CAP]> = ArrayVec::new();
    let mut counter: u16 = 0;
    let mut frames: u16 = 0;
    let mut elapsed: f32 = 0.0;
    let mut clock: Instant = Instant::now();
    unsafe {
        webs_lib::init(&mut rng, uniform, &mut nodes, &mut edges);
        while let Some(event) = events.next(&mut window) {
            if let Some(args) = event.render_args() {
                if (webs_lib::NODES_LIMIT < nodes.len())
                    || (webs_lib::EDGES_LIMIT < edges.len())
                {
                    nodes.clear();
                    edges.clear();
                    webs_lib::init(&mut rng, uniform, &mut nodes, &mut edges);
                } else if webs_lib::INSERT_FRAME_INTERVAL < counter {
                    webs_lib::insert(
                        &mut rng, uniform, &mut nodes, &mut edges,
                    );
                    counter = 0;
                }
                webs_lib::update(&mut nodes);
                counter += 1;
                render(&mut gl, &args, &edges);
                frames += 1;
                elapsed += clock.elapsed().as_secs_f32();
                clock = Instant::now();
                if 1.0 < elapsed {
                    print!("{:>8.2} fps\r", (frames as f32) / elapsed);
                    io::stdout().flush().unwrap();
                    frames = 0;
                    elapsed -= 1.0;
                }
            }
        }
    }
    println!()
}
