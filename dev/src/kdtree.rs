mod kdtree_lib;

use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use kdtree_lib::{Bounds, Point, Tree};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2_window::Sdl2Window;

unsafe fn render(
    gl: &mut GlGraphics,
    args: &RenderArgs,
    point: &Point,
    trees: &[Tree],
    neighbors: &mut ArrayVec<[*const Point; kdtree_lib::CAPACITY]>,
) {
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(kdtree_lib::DARK_GRAY, gl);
        let n: usize = neighbors.len();
        for neighbor in neighbors.drain(..n) {
            graphics::ellipse(
                kdtree_lib::RED,
                [
                    (*neighbor).x - kdtree_lib::RADIUS_2,
                    (*neighbor).y - kdtree_lib::RADIUS_2,
                    kdtree_lib::RADIUS_4,
                    kdtree_lib::RADIUS_4,
                ],
                transform,
                gl,
            );
        }
        for tree in trees {
            let point: &Point = &tree.point;
            let x: f64 = point.x;
            let y: f64 = point.y;
            let bounds: &Bounds = &tree.bounds;
            let line: [f64; 4] = if tree.horizontal {
                [x, bounds.lower.y, x, bounds.upper.y]
            } else {
                [bounds.lower.x, y, bounds.upper.x, y]
            };
            graphics::ellipse(
                kdtree_lib::LIGHT_GRAY,
                [
                    x - kdtree_lib::RADIUS,
                    y - kdtree_lib::RADIUS,
                    kdtree_lib::RADIUS_2,
                    kdtree_lib::RADIUS_2,
                ],
                transform,
                gl,
            );
            graphics::line(
                kdtree_lib::LIGHT_GRAY,
                kdtree_lib::LINE_WIDTH,
                line,
                transform,
                gl,
            );
        }
        graphics::ellipse(
            kdtree_lib::TEAL,
            [
                point.x - kdtree_lib::SEARCH_RADIUS,
                point.y - kdtree_lib::SEARCH_RADIUS,
                kdtree_lib::SEARCH_RADIUS_2,
                kdtree_lib::SEARCH_RADIUS_2,
            ],
            transform,
            gl,
        );
    })
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: Sdl2Window = WindowSettings::new(
        "ranim",
        [kdtree_lib::WINDOW_EDGE, kdtree_lib::WINDOW_EDGE],
    )
    .graphics_api(opengl)
    .exit_on_esc(true)
    .samples(kdtree_lib::ANTI_ALIAS)
    .vsync(true)
    .build()
    .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform_init: Uniform<f64> = Uniform::new_inclusive(
        kdtree_lib::POINT_RNG_LOWER,
        kdtree_lib::POINT_RNG_UPPER,
    );
    let uniform_walk: Uniform<f64> = Uniform::new_inclusive(
        kdtree_lib::WALK_RNG_LOWER,
        kdtree_lib::WALK_RNG_UPPER,
    );
    macro_rules! make_point {
        () => {
            Point {
                x: rng.sample(uniform_init),
                y: rng.sample(uniform_init),
            }
        };
    }
    let mut point: Point = make_point!();
    let mut points: ArrayVec<[Point; kdtree_lib::CAPACITY]> = ArrayVec::new();
    unsafe {
        for _ in 0..kdtree_lib::CAPACITY {
            points.push_unchecked(make_point!());
        }
    }
    let mut trees: ArrayVec<[Tree; kdtree_lib::CAPACITY]> = ArrayVec::new();
    let mut neighbors: ArrayVec<[*const Point; kdtree_lib::CAPACITY]> =
        ArrayVec::new();
    let mut counter: u16 = 0;
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if kdtree_lib::RELOAD_FRAME_INTERVAL < counter {
                point = make_point!();
                for i in 0..kdtree_lib::CAPACITY {
                    points[i].x = rng.sample(uniform_init);
                    points[i].y = rng.sample(uniform_init);
                }
                counter = 0;
            } else {
                point.x += rng.sample(uniform_walk);
                point.y += rng.sample(uniform_walk);
                for point in &mut points {
                    point.x += rng.sample(uniform_walk);
                    point.y += rng.sample(uniform_walk);
                }
                counter += 1;
            }
            unsafe {
                let tree: *mut Tree = kdtree_lib::make_tree(
                    &mut trees,
                    &mut points,
                    true,
                    kdtree_lib::BOUNDS,
                );
                kdtree_lib::search_tree(&point, tree, &mut neighbors);
                render(&mut gl, &args, &point, &trees, &mut neighbors);
            }
            trees.clear();
        }
    }
}
