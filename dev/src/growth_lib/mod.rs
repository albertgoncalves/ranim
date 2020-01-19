use arrayvec::ArrayVec;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

pub const CAPACITY: usize = 512;
pub const NODES_INIT: usize = 3;
pub const NODES_CAP_LIMIT: usize = CAPACITY - 1;
const NODES_INIT_LIMIT: usize = NODES_INIT - 1;

pub const NEIGHBOR_DISTANCE_SQUARED: f64 = 1000.0;
const SEARCH_RADIUS_SQUARED: f64 = 2000.0;

const DRAG_ATTRACT: f64 = 35.0;
const DRAG_REJECT: f64 = 25.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

type NodeIndex = usize;

pub struct Node {
    pub point: Point,
    pub left_index: NodeIndex,
    pub right_index: NodeIndex,
}

pub struct Bounds {
    pub lower: Point,
    pub upper: Point,
}

macro_rules! make_bounds {
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

type TreeIndex = usize;

struct Tree {
    point: Point,
    bounds: Bounds,
    left_index: Option<TreeIndex>,
    right_index: Option<TreeIndex>,
}

pub fn init_nodes(
    rng: &mut ThreadRng,
    uniform: &Uniform<f64>,
    nodes: &mut ArrayVec<[Node; CAPACITY]>,
) {
    for i in 0..NODES_INIT {
        let (left_index, right_index): (NodeIndex, NodeIndex) = {
            if i == 0 {
                (NODES_INIT_LIMIT, i + 1)
            } else if i == NODES_INIT_LIMIT {
                (i - 1, 0)
            } else {
                (i - 1, i + 1)
            }
        };
        nodes.push(Node {
            point: Point {
                x: rng.sample(uniform),
                y: rng.sample(uniform),
            },
            left_index,
            right_index,
        });
    }
}

pub fn insert_node(
    nodes: &mut ArrayVec<[Node; CAPACITY]>,
    left_index: NodeIndex,
) {
    let index: usize = nodes.len();
    let right_index: NodeIndex = nodes[left_index].right_index;
    let left_point: &Point = &nodes[left_index].point;
    let right_point: &Point = &nodes[right_index].point;
    let left_x: f64 = left_point.x;
    let left_y: f64 = left_point.y;
    let right_x: f64 = right_point.x;
    let right_y: f64 = right_point.y;
    nodes.push(Node {
        point: Point {
            x: (left_x + right_x) / 2.0,
            y: (left_y + right_y) / 2.0,
        },
        left_index,
        right_index,
    });
    nodes[left_index].right_index = index;
    nodes[right_index].left_index = index;
}

fn make_tree(
    trees: &mut ArrayVec<[Tree; CAPACITY]>,
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
                make_bounds!(lower_x, lower_y, x, upper_y),
                make_bounds!(x, lower_y, upper_x, upper_y),
            )
        } else {
            points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            let point: Point = points[median].clone();
            let y: f64 = point.y;
            (
                point,
                make_bounds!(lower_x, lower_y, upper_x, y),
                make_bounds!(lower_x, y, upper_x, upper_y),
            )
        }
    };
    let left_index: Option<TreeIndex> =
        make_tree(trees, &mut points[..median], !horizontal, left_bounds);
    let right_index: Option<TreeIndex> = make_tree(
        trees,
        &mut points[(median + 1)..],
        !horizontal,
        right_bounds,
    );
    trees.push(Tree {
        point,
        bounds,
        left_index,
        right_index,
    });
    Some(trees.len() - 1)
}

pub fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

fn bounds_to_point_squared_distance(bounds: &Bounds, point: &Point) -> f64 {
    let x: f64 = point.x - bounds.lower.x.max(point.x.min(bounds.upper.x));
    let y: f64 = point.y - bounds.lower.y.max(point.y.min(bounds.upper.y));
    (x * x) + (y * y)
}

fn search_tree(
    point: &Point,
    trees: &ArrayVec<[Tree; CAPACITY]>,
    index: TreeIndex,
    neighbors: &mut ArrayVec<[TreeIndex; CAPACITY]>,
) {
    let tree: &Tree = &trees[index];
    if bounds_to_point_squared_distance(&tree.bounds, point)
        < SEARCH_RADIUS_SQUARED
    {
        let neighbor: &Point = &tree.point;
        if (point != neighbor)
            && (squared_distance(point, neighbor) < SEARCH_RADIUS_SQUARED)
        {
            neighbors.push(index);
        }
        if let Some(left_index) = tree.left_index {
            search_tree(point, trees, left_index, neighbors);
        }
        if let Some(right_index) = tree.right_index {
            search_tree(point, trees, right_index, neighbors);
        }
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn update_nodes(nodes: &mut ArrayVec<[Node; CAPACITY]>, bounds: Bounds) {
    let mut points: ArrayVec<[Point; CAPACITY]> = ArrayVec::new();
    for node in nodes.iter() {
        points.push(node.point.clone());
    }
    let mut trees: ArrayVec<[Tree; CAPACITY]> = ArrayVec::new();
    if let Some(index) = make_tree(&mut trees, &mut points, true, bounds) {
        let mut next_points: ArrayVec<[(usize, Point); CAPACITY]> =
            ArrayVec::new();
        for (i, node) in nodes.iter().enumerate() {
            let point: &Point = &node.point;
            let left_point: &Point = &nodes[node.left_index].point;
            let right_point: &Point = &nodes[node.right_index].point;
            let mut next_point: Point = Point {
                x: point.x
                    + ((((left_point.x + right_point.x) / 2.0) - point.x)
                        / DRAG_ATTRACT),
                y: point.y
                    + ((((left_point.y + right_point.y) / 2.0) - point.y)
                        / DRAG_ATTRACT),
            };
            let mut neighbors: ArrayVec<[TreeIndex; CAPACITY]> =
                ArrayVec::new();
            search_tree(point, &trees, index, &mut neighbors);
            let n: usize = neighbors.len();
            if 0 < n {
                let mut x: f64 = 0.0;
                let mut y: f64 = 0.0;
                for neighbor_index in neighbors.iter() {
                    let neighbor_point: &Point = &trees[*neighbor_index].point;
                    x += point.x - neighbor_point.x;
                    y += point.y - neighbor_point.y;
                }
                let n: f64 = n as f64;
                next_point.x += (x / n) / DRAG_REJECT;
                next_point.y += (y / n) / DRAG_REJECT;
            }
            next_points.push((i, next_point));
        }
        for (i, next_point) in next_points {
            nodes[i].point.x = next_point.x;
            nodes[i].point.y = next_point.y;
        }
    }
}
