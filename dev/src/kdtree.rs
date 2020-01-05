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

const WINDOW_WIDTH: f64 = 500.0;
const WINDOW_HEIGHT: f64 = 500.0;
const ANTI_ALIAS: u8 = 4;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

const LINE_WIDTH: f64 = 1.15;
const RADIUS: f64 = 6.0;
const RADIUS_2: f64 = RADIUS * 2.0;

const POINT_RNG_UPPER: f64 = 400.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;
const WALK_RNG_UPPER: f64 = 0.35;
const WALK_RNG_LOWER: f64 = -WALK_RNG_UPPER;

const CAPACITY: usize = 50;

#[derive(Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
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

const BOUNDS_RECT: Rect = Rect {
    x: BOUNDS.lower.x,
    y: BOUNDS.lower.y,
    width: BOUNDS.upper.x - BOUNDS.lower.x,
    height: BOUNDS.upper.y - BOUNDS.lower.y,
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

fn construct(
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
    let (point, left_bounds, right_bounds): (Point, Bounds, Bounds) = {
        if horizontal {
            points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            let point: Point = points[median].clone();
            let left_bounds: Bounds = bounds!(
                bounds.lower.x, //
                bounds.lower.y, //
                point.x,        //
                bounds.upper.y,
            );
            let right_bounds: Bounds = bounds!(
                point.x,        //
                bounds.lower.y, //
                bounds.upper.x, //
                bounds.upper.y,
            );
            (point, left_bounds, right_bounds)
        } else {
            points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            let point: Point = points[median].clone();
            let left_bounds: Bounds = bounds!(
                bounds.lower.x, //
                bounds.lower.y, //
                bounds.upper.x, //
                point.y,
            );
            let right_bounds: Bounds = bounds!(
                bounds.lower.x, //
                point.y,        //
                bounds.upper.x, //
                bounds.upper.y,
            );
            (point, left_bounds, right_bounds)
        }
    };
    let left: Option<TreeIndex> =
        construct(tree_stack, &mut points[..median], !horizontal, left_bounds);
    let right: Option<TreeIndex> = construct(
        tree_stack,
        &mut points[(median + 1)..],
        !horizontal,
        right_bounds,
    );
    let tree_index: TreeIndex = tree_stack.len();
    tree_stack.push(Tree {
        point,
        bounds,
        horizontal,
        left,
        right,
    });
    Some(tree_index)
}

fn draw_tree(
    gl: &mut GlGraphics,
    transform: Matrix2d,
    tree_stack: &ArrayVec<[Tree; CAPACITY]>,
    tree_index: TreeIndex,
) {
    let tree: &Tree = &tree_stack[tree_index];
    let line: [f64; 4] = if tree.horizontal {
        [
            tree.point.x,
            tree.bounds.lower.y,
            tree.point.x,
            tree.bounds.upper.y,
        ]
    } else {
        [
            tree.bounds.lower.x,
            tree.point.y,
            tree.bounds.upper.x,
            tree.point.y,
        ]
    };
    graphics::ellipse(
        LIGHT_GRAY,
        [
            tree.point.x - RADIUS,
            tree.point.y - RADIUS,
            RADIUS_2,
            RADIUS_2,
        ],
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
    tree_stack: &ArrayVec<[Tree; CAPACITY]>,
    tree_index: TreeIndex,
) {
    gl.draw(args.viewport(), |context, gl| {
        let transform: Matrix2d = context
            .transform
            .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(
            DARK_GRAY,
            [
                BOUNDS_RECT.x,
                BOUNDS_RECT.y,
                BOUNDS_RECT.width,
                BOUNDS_RECT.height,
            ],
            transform,
            gl,
        );
        draw_tree(gl, transform, tree_stack, tree_index);
    })
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut settings: WindowSettings =
        WindowSettings::new("ranim", [WINDOW_WIDTH, WINDOW_HEIGHT])
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
    let mut points: ArrayVec<[Point; CAPACITY]> = ArrayVec::new();
    for _ in 0..CAPACITY {
        points.push(point!());
    }
    let mut tree_stack: ArrayVec<[Tree; CAPACITY]> = ArrayVec::new();
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if let Some(tree_index) =
                construct(&mut tree_stack, &mut points, true, BOUNDS)
            {
                render(&mut gl, &args, &tree_stack, tree_index);
                tree_stack.clear();
                for point in &mut points {
                    point.x += rng.sample(range_walk);
                    point.y += rng.sample(range_walk);
                }
            }
        }
    }
}
