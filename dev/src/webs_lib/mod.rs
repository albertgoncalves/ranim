#![allow(clippy::cast_possible_truncation)]

use arrayvec::ArrayVec;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

pub const WINDOW_EDGE: f64 = 800.0;
const WINDOW_EDGE_HALF: f32 = (WINDOW_EDGE as f32) / 2.0;
const WINDOW_EDGE_HALF_MINUS: f32 = -WINDOW_EDGE_HALF;

pub const ANTI_ALIAS: u8 = 4;

pub const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
pub const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
pub const CYAN: [f32; 4] = [0.5, 1.0, 0.87, 1.0];
pub const TEAL: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

pub const LINE_WIDTH: f64 = 0.8;
pub const RADIUS: f64 = 3.5;
pub const RADIUS_2: f64 = RADIUS * 2.0;
pub const RECT_PAD: f64 = 17.5;
pub const RECT_PAD_2: f64 = RECT_PAD * 2.0;

pub const INSERT_FRAME_INTERVAL: u16 = 10;

pub const NODES_CAP: usize = 1024;
pub const EDGES_CAP: usize = 1024;

const NODES_INIT: usize = EDGES_INIT * 2;
const EDGES_INIT: usize = 1;

const NEIGHBORS_CAP: usize = 3;
const INTERSECTIONS_CAP: usize = 16;

pub const NODES_LIMIT: usize = NODES_CAP - 2;
pub const EDGES_LIMIT: usize = EDGES_CAP - 3;

pub const POINT_RNG_UPPER: f32 = WINDOW_EDGE_HALF;
pub const POINT_RNG_LOWER: f32 = WINDOW_EDGE_HALF_MINUS;

const POINT_DRAG: f32 = 0.0025;
const NEIGHBOR_DISTANCE_SQUARED: f32 = 100.0;

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct Node {
    pub point: Point,
    pub neighbors: ArrayVec<[*mut Node; NEIGHBORS_CAP]>,
}

pub struct Edge {
    pub a: *mut Node,
    pub b: *mut Node,
}

struct Intersection<'a> {
    point: Point,
    edge: &'a mut Edge,
}

pub unsafe fn init(
    rng: &mut ThreadRng,
    uniform: Uniform<f32>,
    nodes: &mut ArrayVec<[Node; NODES_CAP]>,
    edges: &mut ArrayVec<[Edge; EDGES_CAP]>,
) {
    for _ in 0..EDGES_INIT {
        nodes.push_unchecked(Node {
            point: Point {
                x: rng.sample(uniform),
                y: rng.sample(uniform),
            },
            neighbors: ArrayVec::new(),
        });
        let a: *mut Node = nodes.last_mut().unwrap();
        nodes.push_unchecked(Node {
            point: Point {
                x: rng.sample(uniform),
                y: rng.sample(uniform),
            },
            neighbors: ArrayVec::new(),
        });
        let b: *mut Node = nodes.last_mut().unwrap();
        (*b).neighbors.push_unchecked(a);
        (*a).neighbors.push_unchecked(b);
        edges.push_unchecked(Edge { a, b });
    }
}

#[allow(clippy::many_single_char_names)]
fn get_intersection(
    a: &Point,
    b: &Point,
    c: &Point,
    d: &Point,
) -> Option<Point> {
    /* NOTE:     `a`
     *            |
     *       `c`--+--`d`
     *            |
     *           `b`
     */
    let x1: f32 = a.x;
    let x2: f32 = b.x;
    let x3: f32 = c.x;
    let x4: f32 = d.x;
    let y1: f32 = a.y;
    let y2: f32 = b.y;
    let y3: f32 = c.y;
    let y4: f32 = d.y;
    let denominator: f32 = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));
    if denominator != 0.0 {
        let t: f32 =
            (((x1 - x3) * (y3 - y4)) - ((y1 - y3) * (x3 - x4))) / denominator;
        let u: f32 =
            -(((x1 - x2) * (y1 - y3)) - ((y1 - y2) * (x1 - x3))) / denominator;
        if (0.0 <= t) && (t <= 1.0) && (0.0 <= u) && (u <= 1.0) {
            return Some(Point {
                x: x1 + (t * (x2 - x1)),
                y: y1 + (t * (y2 - y1)),
            });
        }
    }
    None
}

macro_rules! replace_neighbor {
    ($node:expr, $old:expr, $new:expr $(,)?) => {
        for neighbor in &mut $node.neighbors {
            if *neighbor == $old {
                *neighbor = $new;
                break;
            }
        }
    };
}

#[allow(clippy::comparison_chain, clippy::many_single_char_names)]
pub unsafe fn insert(
    rng: &mut ThreadRng,
    uniform: Uniform<f32>,
    nodes: &mut ArrayVec<[Node; NODES_CAP]>,
    edges: &mut ArrayVec<[Edge; EDGES_CAP]>,
) {
    loop {
        let candidate_a: Point = Point {
            x: rng.sample(uniform),
            y: rng.sample(uniform),
        };
        let candidate_b: Point = Point {
            x: rng.sample(uniform),
            y: rng.sample(uniform),
        };
        let mut intersections: Vec<Intersection> =
            Vec::with_capacity(INTERSECTIONS_CAP);
        for edge in edges.iter_mut() {
            if let Some(point) = get_intersection(
                &candidate_a,
                &candidate_b,
                &(*edge.a).point,
                &(*edge.b).point,
            ) {
                intersections.push(Intersection { point, edge });
            }
        }
        let n: usize = intersections.len();
        if n == 1 {
            /* NOTE: `a`---`b`    `a`--`p`--`b`
             *                 ->       |
             *                         `q`
             */
            let intersection: Intersection = intersections.pop().unwrap();
            let edge: &mut Edge = intersection.edge;
            let a: *mut Node = edge.a;
            let b: *mut Node = edge.b;
            nodes.push_unchecked(Node {
                point: candidate_a,
                neighbors: ArrayVec::new(),
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            nodes.push_unchecked(Node {
                point: intersection.point,
                neighbors: ArrayVec::from([a, b, q]),
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            replace_neighbor!(*a, b, p);
            replace_neighbor!(*b, a, p);
            (*q).neighbors.push_unchecked(p);
            (*edge).b = p;
            edges.push_unchecked(Edge { a: p, b });
            edges.push_unchecked(Edge { a: p, b: q });
            return;
        } else if 1 < n {
            /* NOTE: `l.a`---`l.b`    `l.a`--`p`--`l.b`
             *                     ->         |
             *       `r.a`---`r.b`    `r.a`--`q`--`r.b`
             */
            intersections.sort_unstable_by(|a, b| {
                a.point.x.partial_cmp(&b.point.x).unwrap()
            });
            let i: usize = rng.gen_range(0, n - 1);
            let l_intersection: Intersection = intersections.remove(i);
            let r_intersection: Intersection = intersections.remove(i);
            let l_edge: &mut Edge = l_intersection.edge;
            let r_edge: &mut Edge = r_intersection.edge;
            let l_a: *mut Node = (*l_edge).a;
            let l_b: *mut Node = (*l_edge).b;
            let r_a: *mut Node = (*r_edge).a;
            let r_b: *mut Node = (*r_edge).b;
            nodes.push_unchecked(Node {
                point: r_intersection.point,
                neighbors: ArrayVec::new(),
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            nodes.push_unchecked(Node {
                point: l_intersection.point,
                neighbors: ArrayVec::from([l_a, l_b, q]),
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            replace_neighbor!(*l_a, l_b, p);
            replace_neighbor!(*l_b, l_a, p);
            replace_neighbor!(*r_a, r_b, q);
            replace_neighbor!(*r_b, r_a, q);
            (*q).neighbors.push_unchecked(r_a);
            (*q).neighbors.push_unchecked(r_b);
            (*q).neighbors.push_unchecked(p);
            (*l_edge).b = p;
            (*r_edge).b = q;
            edges.push_unchecked(Edge { a: p, b: l_b });
            edges.push_unchecked(Edge { a: q, b: r_b });
            edges.push_unchecked(Edge { a: p, b: q });
            return;
        }
    }
}

fn squared_distance(a: &Point, b: &Point) -> f32 {
    let x: f32 = a.x - b.x;
    let y: f32 = a.y - b.y;
    (x * x) + (y * y)
}

pub unsafe fn update(nodes: &mut ArrayVec<[Node; NODES_CAP]>) {
    let mut updates: ArrayVec<[(usize, Point); NODES_CAP]> = ArrayVec::new();
    for i in NODES_INIT..nodes.len() {
        let node: &Node = nodes.get_unchecked(i);
        let node_point: &Point = &node.point;
        let node_x: f32 = node_point.x;
        let node_y: f32 = node_point.y;
        let mut n: f32 = 0.0;
        let mut update_x: f32 = 0.0;
        let mut update_y: f32 = 0.0;
        for neighbor in &node.neighbors {
            let neighbor_point: &Point = &(**neighbor).point;
            if NEIGHBOR_DISTANCE_SQUARED
                < squared_distance(node_point, neighbor_point)
            {
                n += 1.0;
                update_x += node_x - neighbor_point.x;
                update_y += node_y - neighbor_point.y;
            }
        }
        if 0.0 < n {
            updates.push_unchecked((
                i,
                Point {
                    x: node_x - ((update_x / n) * POINT_DRAG),
                    y: node_y - ((update_y / n) * POINT_DRAG),
                },
            ));
        }
    }
    for (i, update_point) in updates {
        let node_point: &mut Point = &mut nodes.get_unchecked_mut(i).point;
        node_point.x = update_point.x;
        node_point.y = update_point.y;
    }
}
