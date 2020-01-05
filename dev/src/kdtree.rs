use arrayvec::ArrayVec;
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
const RED: [f32; 4] = [0.92, 0.47, 0.47, 0.75];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

const LINE_WIDTH: f64 = 1.15;
const RADIUS: f64 = 6.0;
const RADIUS_2: f64 = RADIUS * 2.0;
const RADIUS_4: f64 = RADIUS * 4.0;

const POINT_RNG_LOWER: f64 = WINDOW_EDGE_HALF_MINUS;
const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF;
const WALK_RNG_UPPER: f64 = 0.35;
const WALK_RNG_LOWER: f64 = -WALK_RNG_UPPER;

const SEARCH_RADIUS: f64 = 150.0;
const SEARCH_RADIUS_2: f64 = SEARCH_RADIUS * 2.0;
const SEARCH_RADIUS_SQUARED: f64 = SEARCH_RADIUS * SEARCH_RADIUS;

const CAPACITY: usize = 100;

const RELOAD_FRAME_INTERVAL: u16 = 60 * 8;

#[derive(Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

struct Bounds {
    lower: Point,
    upper: Point,
}

const BOUNDS: Bounds = Bounds {
    lower: Point {
        x: POINT_RNG_LOWER,
        y: POINT_RNG_LOWER,
    },
    upper: Point {
        x: POINT_RNG_UPPER,
        y: POINT_RNG_UPPER,
    },
};

type TreeIndex = usize;

struct Tree {
    point: Point,
    bounds: Bounds,
    horizontal: bool,
    left: Option<TreeIndex>,
    right: Option<TreeIndex>,
}

macro_rules! bounds {
    ($lower_x:expr, $lower_y:expr, $upper_x:expr, $upper_y: expr $(,)?) => {
        Bounds {
            lower: Point {
                x: $lower_x,
                y: $lower_y,
            },
            upper: Point {
                x: $upper_x,
                y: $upper_y,
            },
        }
    };
}

fn construct_tree(
    tree_stack: &mut ArrayVec<[Tree; CAPACITY]>,
    points: &mut [Point],
    horizontal: bool,
    bounds: Bounds,
) -> Option<TreeIndex> {
    let n: usize = points.len();
    if n == 0 {
        return None;
    }
    let median: usize = n / 2;
    let lower_x: f64 = bounds.lower.x;
    let lower_y: f64 = bounds.lower.y;
    let upper_x: f64 = bounds.upper.x;
    let upper_y: f64 = bounds.upper.y;
    let (point, left_bounds, right_bounds): (Point, Bounds, Bounds) = {
        if horizontal {
            points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            let point: Point = points[median].clone();
            let x: f64 = point.x;
            (
                point,
                bounds!(lower_x, lower_y, x, upper_y),
                bounds!(x, lower_y, upper_x, upper_y),
            )
        } else {
            points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            let point: Point = points[median].clone();
            let y: f64 = point.y;
            (
                point,
                bounds!(lower_x, lower_y, upper_x, y),
                bounds!(lower_x, y, upper_x, upper_y),
            )
        }
    };
    let left: Option<TreeIndex> = construct_tree(
        tree_stack,
        &mut points[..median],
        !horizontal,
        left_bounds,
    );
    let right: Option<TreeIndex> = construct_tree(
        tree_stack,
        &mut points[(median + 1)..],
        !horizontal,
        right_bounds,
    );
    let tree_index: TreeIndex = tree_stack.len();
    unsafe {
        tree_stack.push_unchecked(Tree {
            point,
            bounds,
            horizontal,
            left,
            right,
        });
    }
    Some(tree_index)
}

fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

unsafe fn search_tree(
    point: &Point,
    neighbors: &mut ArrayVec<[&Point; CAPACITY]>,
    tree_stack: *const ArrayVec<[Tree; CAPACITY]>,
    tree_index: TreeIndex,
) {
    let tree: &Tree = &(*tree_stack)[tree_index];
    let bounds: &Bounds = &tree.bounds;
    let x: f64 = point.x - bounds.lower.x.max(point.x.min(bounds.upper.x));
    let y: f64 = point.y - bounds.lower.y.max(point.y.min(bounds.upper.y));
    if ((x * x) + (y * y)) < SEARCH_RADIUS_SQUARED {
        let neighbor: &Point = &tree.point;
        if (point != neighbor)
            && (squared_distance(point, neighbor) < SEARCH_RADIUS_SQUARED)
        {
            neighbors.push(&tree.point);
        }
        if let Some(left) = tree.left {
            search_tree(point, neighbors, tree_stack, left);
        }
        if let Some(right) = tree.right {
            search_tree(point, neighbors, tree_stack, right);
        }
    }
}

fn draw_tree(
    gl: &mut GlGraphics,
    transform: Matrix2d,
    tree_stack: &ArrayVec<[Tree; CAPACITY]>,
    tree_index: TreeIndex,
) {
    let tree: &Tree = &tree_stack[tree_index];
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
        LIGHT_GRAY,
        [x - RADIUS, y - RADIUS, RADIUS_2, RADIUS_2],
        transform,
        gl,
    );
    graphics::line(LIGHT_GRAY, LINE_WIDTH, line, transform, gl);
    if let Some(left) = tree.left {
        draw_tree(gl, transform, tree_stack, left);
    }
    if let Some(right) = tree.right {
        draw_tree(gl, transform, tree_stack, right);
    }
}

fn render(
    gl: &mut GlGraphics,
    args: &RenderArgs,
    point: &Point,
    neighbors: &[&Point],
    tree_stack: &ArrayVec<[Tree; CAPACITY]>,
    tree_index: TreeIndex,
) {
    gl.draw(args.viewport(), |context, gl| {
        let transform: Matrix2d = context
            .transform
            .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(DARK_GRAY, WINDOW_RECT, transform, gl);
        for neighbor in neighbors {
            graphics::ellipse(
                RED,
                [
                    neighbor.x - RADIUS_2,
                    neighbor.y - RADIUS_2,
                    RADIUS_4,
                    RADIUS_4,
                ],
                transform,
                gl,
            );
        }
        draw_tree(gl, transform, tree_stack, tree_index);
        graphics::ellipse(
            CYAN,
            [
                point.x - SEARCH_RADIUS,
                point.y - SEARCH_RADIUS,
                SEARCH_RADIUS_2,
                SEARCH_RADIUS_2,
            ],
            transform,
            gl,
        );
    })
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut settings: WindowSettings =
        WindowSettings::new("ranim", [WINDOW_EDGE, WINDOW_EDGE])
            .graphics_api(opengl)
            .exit_on_esc(true);
    settings.set_samples(ANTI_ALIAS);
    let mut window: GlutinWindow = settings.build().unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let range_init: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let range_walk: Uniform<f64> =
        Uniform::new_inclusive(WALK_RNG_LOWER, WALK_RNG_UPPER);
    macro_rules! point {
        () => {
            Point {
                x: rng.sample(range_init),
                y: rng.sample(range_init),
            }
        };
    }
    let mut point: Point = point!();
    let mut points: ArrayVec<[Point; CAPACITY]> = ArrayVec::new();
    for _ in 0..CAPACITY {
        unsafe {
            points.push_unchecked(point!());
        }
    }
    let mut tree_stack: ArrayVec<[Tree; CAPACITY]> = ArrayVec::new();
    let mut neighbors: ArrayVec<[&Point; CAPACITY]> = ArrayVec::new();
    let mut counter: u16 = 0;
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if RELOAD_FRAME_INTERVAL < counter {
                point = point!();
                for i in 0..CAPACITY {
                    points[i].x = rng.sample(range_init);
                    points[i].y = rng.sample(range_init);
                }
                counter = 0;
            }
            if let Some(tree_index) =
                construct_tree(&mut tree_stack, &mut points, true, BOUNDS)
            {
                unsafe {
                    search_tree(
                        &point,
                        &mut neighbors,
                        &tree_stack,
                        tree_index,
                    );
                }
                render(
                    &mut gl,
                    &args,
                    &point,
                    &neighbors,
                    &tree_stack,
                    tree_index,
                );
                point.x += rng.sample(range_walk);
                point.y += rng.sample(range_walk);
                for point in &mut points {
                    point.x += rng.sample(range_walk);
                    point.y += rng.sample(range_walk);
                }
            }
            neighbors.clear();
            tree_stack.clear();
            counter += 1;
        }
    }
}
