#![allow(clippy::cast_lossless)]

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
use sdl2_window::Sdl2Window;
use std::io;
use std::io::Write;
use std::time::Instant;

fn render(gl: &mut GlGraphics, args: &RenderArgs, nodes: &[Node]) {
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(growth_lib::DARK_GRAY, gl);
        {
            let node: &Node = nodes.last().unwrap();
            let x: f64 = node.point.x as f64;
            let y: f64 = node.point.y as f64;
            graphics::ellipse(
                growth_lib::CYAN,
                [
                    x - growth_lib::RADIUS_2,
                    y - growth_lib::RADIUS_2,
                    growth_lib::RADIUS_4,
                    growth_lib::RADIUS_4,
                ],
                transform,
                gl,
            );
        }
        for node in nodes {
            let x: f64 = node.point.x as f64;
            let y: f64 = node.point.y as f64;
            graphics::ellipse(
                growth_lib::LIGHT_GRAY,
                [
                    x - growth_lib::RADIUS,
                    y - growth_lib::RADIUS,
                    growth_lib::RADIUS_2,
                    growth_lib::RADIUS_2,
                ],
                transform,
                gl,
            );
            let left: &Point = &nodes[node.left_index].point;
            let left_x: f64 = left.x as f64;
            let left_y: f64 = left.y as f64;
            graphics::line(
                growth_lib::LIGHT_GRAY,
                growth_lib::LINE_WIDTH,
                [left_x, left_y, x, y],
                transform,
                gl,
            )
        }
    })
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Sdl2Window = WindowSettings::new(
        "ranim",
        [growth_lib::WINDOW_EDGE, growth_lib::WINDOW_EDGE],
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .samples(growth_lib::ANTI_ALIAS)
    .vsync(true)
    .build()
    .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform_init: Uniform<f32> = Uniform::new_inclusive(
        growth_lib::POINT_RNG_LOWER,
        growth_lib::POINT_RNG_UPPER,
    );
    let uniform_walk: Uniform<f32> = Uniform::new_inclusive(
        growth_lib::WALK_RNG_LOWER,
        growth_lib::WALK_RNG_UPPER,
    );
    let mut nodes: ArrayVec<[Node; growth_lib::CAPACITY]> = ArrayVec::new();
    growth_lib::init_nodes(&mut rng, uniform_init, &mut nodes);
    let mut frames: u16 = 0;
    let mut elapsed: f32 = 0.0;
    let mut clock: Instant = Instant::now();
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if growth_lib::NODES_CAP_LIMIT < nodes.len() {
                nodes.clear();
                growth_lib::init_nodes(&mut rng, uniform_init, &mut nodes);
            } else {
                growth_lib::update_nodes(&mut rng, uniform_walk, &mut nodes);
            }
            render(&mut gl, &args, &nodes);
            frames += 1;
            elapsed += clock.elapsed().as_secs_f32();
            clock = Instant::now();
            if 1.0 < elapsed {
                print!("\r{:>7.2} fps", (frames as f32) / elapsed);
                io::stdout().flush().unwrap();
                frames = 0;
                elapsed = 0.0;
            };
        }
    }
    println!()
}
