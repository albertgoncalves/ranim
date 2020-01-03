mod point;

use point::Point;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::f64;
use std::ptr;

const WINDOW_WIDTH: f64 = 600.0;
const WINDOW_HEIGHT: f64 = 600.0;
const WINDOW_HALF_WIDTH: f64 = WINDOW_WIDTH / 2.0;
const WINDOW_HALF_HEIGHT: f64 = WINDOW_HEIGHT / 2.0;

const PI_2: f64 = f64::consts::PI * 2.0;

const INIT: usize = 5;
const SPREAD: f64 = 10.0;

const SEARCH_RADIUS: f64 = 10.0;
const SEARCH_RADIUS_SQUARED: f64 = SEARCH_RADIUS * SEARCH_RADIUS;

const GATE: f32 = 0.1;
const THRESHOLD: f64 = 10.0;
const DRAG_ATTRACT: f64 = 5.0;
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

fn init(rng: &mut ThreadRng) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::with_capacity(INIT);
    for _ in 0..INIT {
        let angle: f64 = rng.gen_range(0.0, PI_2);
        nodes.push(Node {
            point: Point {
                x: (angle.cos() * SPREAD) + WINDOW_HALF_WIDTH,
                y: (angle.sin() * SPREAD) + WINDOW_HALF_HEIGHT,
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
    nodes
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

fn main() {
    let mut rng: ThreadRng = rand::thread_rng();
    let nodes: Vec<Node> = init(&mut rng);
    println!("{:#?}", nodes);
}
