mod point;

use glutin_window::GlutinWindow;
use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use point::Point;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::f64;
use std::ptr;

const WINDOW_WIDTH: f64 = 600.0;
const WINDOW_HEIGHT: f64 = 600.0;
const WINDOW_HALF_WIDTH: f64 = WINDOW_WIDTH / 2.0;
const WINDOW_HALF_HEIGHT: f64 = WINDOW_HEIGHT / 2.0;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];

const LINE_WIDTH: f64 = 0.7;

const PI_2: f64 = f64::consts::PI * 2.0;

const INIT: usize = 5;
const SPREAD: f64 = 100.0;

const SEARCH_RADIUS: f64 = 10.0;
const SEARCH_RADIUS_SQUARED: f64 = SEARCH_RADIUS * SEARCH_RADIUS;

const GATE: f32 = 0.1;
const THRESHOLD: f64 = 10.0;
const DRAG_ATTRACT: f64 = 10.0;
const DRAG_REJECT: f64 = 10.0;

#[derive(Debug)]
struct Node<'a> {
    point: Point,
    next: Point,
    left: *mut Node<'a>,
    right: *mut Node<'a>,
    neighbors: Vec<&'a Point>,
}

struct Bounds {
    lower: Point,
    upper: Point,
}

struct Tree<'a> {
    point: &'a Point,
    bounds: Bounds,
    left: Option<Box<Tree<'a>>>,
    right: Option<Box<Tree<'a>>>,
}

fn build_tree<'a>(
    mut points_by_x: Vec<&'a Point>,
    mut points_by_y: Vec<&'a Point>,
    horizontal: bool,
    bounds: Bounds,
) -> Option<Box<Tree<'a>>> {
    let n: usize = points_by_x.len();
    if n == 0 {
        return None;
    }
    let median: usize = n / 2;
    let (point, left_bounds, right_bounds): (&Point, Bounds, Bounds) =
        if horizontal {
            let point: &Point = points_by_x.remove(median);
            let _: &Point = points_by_y.remove(median);
            let left_bounds: Bounds = Bounds {
                lower: Point {
                    x: bounds.lower.x,
                    y: bounds.lower.y,
                },
                upper: Point {
                    x: point.x,
                    y: bounds.upper.y,
                },
            };
            let right_bounds: Bounds = Bounds {
                lower: Point {
                    x: point.x,
                    y: bounds.lower.y,
                },
                upper: Point {
                    x: bounds.upper.x,
                    y: bounds.upper.y,
                },
            };
            (point, left_bounds, right_bounds)
        } else {
            let _: &Point = points_by_x.remove(median);
            let point: &Point = points_by_y.remove(median);
            let left_bounds: Bounds = Bounds {
                lower: Point {
                    x: bounds.lower.x,
                    y: bounds.lower.y,
                },
                upper: Point {
                    x: bounds.upper.x,
                    y: point.y,
                },
            };
            let right_bounds: Bounds = Bounds {
                lower: Point {
                    x: bounds.lower.x,
                    y: point.y,
                },
                upper: Point {
                    x: bounds.upper.x,
                    y: bounds.upper.y,
                },
            };
            (point, left_bounds, right_bounds)
        };
    let mut left_points_by_x: Vec<&Point> = points_by_x;
    let mut left_points_by_y: Vec<&Point> = points_by_y;
    let right_points_by_x: Vec<&Point> = left_points_by_x.split_off(median);
    let right_points_by_y: Vec<&Point> = left_points_by_y.split_off(median);
    Some(Box::new(Tree {
        point,
        bounds,
        left: build_tree(
            left_points_by_x,
            left_points_by_y,
            !horizontal,
            left_bounds,
        ),
        right: build_tree(
            right_points_by_x,
            right_points_by_y,
            !horizontal,
            right_bounds,
        ),
    }))
}

fn transform<'a>(nodes: &'a [Node<'a>]) -> Option<Box<Tree<'a>>> {
    let n: usize = nodes.len();
    let mut points: Vec<&Point> = Vec::with_capacity(n);
    for node in nodes.iter() {
        points.push(&node.point)
    }
    let mut points_by_x: Vec<&Point> = points.clone();
    let mut points_by_y: Vec<&Point> = points;
    points_by_x.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    points_by_y.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    build_tree(
        points_by_x,
        points_by_y,
        true,
        Bounds {
            lower: Point { x: 0.0, y: 0.0 },
            upper: Point {
                x: WINDOW_WIDTH,
                y: WINDOW_HEIGHT,
            },
        },
    )
}

fn neighbors_within_radius<'a, 'b, 'c>(
    point: &'a Point,
    points: &'b mut Vec<&'a Point>,
    tree: &'c Option<Box<Tree<'a>>>,
) {
    if let Some(tree) = tree {
        let x: f64 = point.x
            - tree.bounds.lower.x.max(point.x.min(tree.bounds.upper.x));
        let y: f64 = point.y
            - tree.bounds.lower.y.max(point.y.min(tree.bounds.upper.y));
        if ((x * x) + (y * y)) < SEARCH_RADIUS_SQUARED {
            points.push(tree.point);
            neighbors_within_radius(point, points, &tree.left);
            neighbors_within_radius(point, points, &tree.right);
        }
    }
}

fn update_neighbors<'a>(
    nodes: &'a mut [Node<'a>],
    tree: &Option<Box<Tree<'a>>>,
) {
    for node in nodes.iter_mut() {
        neighbors_within_radius(&node.point, &mut node.neighbors, tree)
    }
}

#[allow(clippy::cast_precision_loss)]
fn update_positions(nodes: &mut [Node]) {
    unsafe {
        for node in nodes.iter_mut() {
            let x: f64 = (*node.left).point.x + (*node.right).point.x;
            let y: f64 = (*node.left).point.y + (*node.right).point.y;
            node.next.x =
                node.point.x + (((x / 2.0) - node.point.x) / DRAG_ATTRACT);
            node.next.y =
                node.point.y + (((y / 2.0) - node.point.y) / DRAG_ATTRACT);
            let n: usize = node.neighbors.len();
            if 0 < n {
                let n: f64 = n as f64;
                let mut x: f64 = 0.0;
                let mut y: f64 = 0.0;
                for neighbor in &mut node.neighbors {
                    x += node.point.x - neighbor.x;
                    y += node.point.y - neighbor.y;
                }
                node.next.x += (x / n) / DRAG_REJECT;
                node.next.y += (y / n) / DRAG_REJECT;
            }
        }
        for node in nodes.iter_mut() {
            node.point.x = node.next.x;
            node.point.y = node.next.y;
        }
    }
}

fn insert(rng: &mut ThreadRng, nodes: &mut Vec<Node>) {
    let mut selection: Option<*mut Node> = None;
    unsafe {
        for node in nodes.iter_mut() {
            if (rng.gen::<f32>() < GATE)
                && (THRESHOLD
                    < point::squared_distance(
                        &node.point,
                        &(*node.left).point,
                    ))
            {
                selection = Some(node);
                break;
            }
        }
        if let Some(node) = selection {
            nodes.push(Node {
                point: Point {
                    x: ((*(*node).left).point.x + (*node).point.x) / 2.0,
                    y: ((*(*node).left).point.y + (*node).point.y) / 2.0,
                },
                next: empty_point!(),
                left: (*node).left,
                right: node,
                neighbors: Vec::with_capacity(INIT),
            });
            let new_node: *mut Node = nodes.last_mut().unwrap();
            (*(*node).left).right = new_node;
            (*node).left = new_node;
        }
    }
}

fn render(gl: &mut GlGraphics, args: &RenderArgs, nodes: &[Node]) {
    unsafe {
        gl.draw(args.viewport(), |context, gl| {
            let transform: Matrix2d = context
                .transform
                .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
            graphics::clear(DARK_GRAY, gl);
            for node in nodes {
                graphics::line(
                    LIGHT_GRAY,
                    LINE_WIDTH,
                    [
                        (*node.left).point.x,
                        (*node.left).point.y,
                        node.point.x,
                        node.point.y
                    ],
                    transform,
                    gl,
                )
            }
        })
    }
}

fn main() {
    let opengl: OpenGL = OpenGL::V3_2;
    let mut window: GlutinWindow =
        WindowSettings::new("ranim", [WINDOW_WIDTH, WINDOW_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let mut nodes: Vec<Node> = Vec::with_capacity(INIT);
    let mut angles: Vec<f64> = Vec::with_capacity(INIT);
    for _ in 0..INIT {
        angles.push(rng.gen_range(0.0, PI_2));
    }
    angles.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    for i in 0..INIT {
        nodes.push(Node {
            point: Point {
                x: (angles[i].cos() * SPREAD),
                y: (angles[i].sin() * SPREAD),
            },
            next: empty_point!(),
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            neighbors: Vec::with_capacity(INIT),
        });
    }
    let n: usize = INIT - 1;
    for i in 0..INIT {
        let (l, r): (usize, usize) = {
            if i == 0 {
                (n, 1)
            } else if i == n {
                (i - 1, 0)
            } else {
                (i - 1, i + 1)
            }
        };
        nodes[i].left = &mut nodes[l];
        nodes[i].right = &mut nodes[r];
    }
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            {
                render(&mut gl, &args, &nodes);
            }
            {
                let tree: Option<Box<Tree>> = transform(&nodes);
                update_neighbors(&mut nodes, &tree);
            }
        }
    }
}
