use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

const START: usize = 2;
const STOP: usize = 20;
const CAPACITY: usize = 800;
const THRESHOLD: usize = CAPACITY - 3;

const WINDOW_WIDTH: f64 = 400.0;
const WINDOW_HEIGHT: f64 = 400.0;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.35];

const POINT_LOWER: f64 = -300.0;
const POINT_UPPER: f64 = 300.0;

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Node {
    point: Point,
    neighbors: Vec<*mut Node>,
}

#[derive(Debug)]
struct Edge {
    a: *mut Node,
    b: *mut Node,
}

struct Candidate {
    a: Point,
    b: Point,
}

struct Intersection<'a> {
    point: Point,
    edge: &'a mut Edge,
}

fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

#[allow(clippy::many_single_char_names)]
fn point_of_intersection(
    a: &Point,
    b: &Point,
    c: &Point,
    d: &Point,
) -> Option<Point> {
    let x1 = a.x;
    let x2 = b.x;
    let x3 = c.x;
    let x4 = d.x;
    let y1 = a.y;
    let y2 = b.y;
    let y3 = c.y;
    let y4 = d.y;
    let denominator = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));
    if denominator != 0.0 {
        let t =
            (((x1 - x3) * (y3 - y4)) - ((y1 - y3) * (x3 - x4))) / denominator;
        let u =
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
    ($x:expr, $a:expr, $b:expr $(,)?) => {
        for neighbor in &mut $x.neighbors {
            if *neighbor == $a {
                *neighbor = $b;
                break;
            }
        }
    };
}

#[allow(clippy::comparison_chain, clippy::many_single_char_names)]
fn insert(
    rng: &mut ThreadRng,
    range: &Uniform<f64>,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    loop {
        let candidate: Candidate = Candidate {
            a: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            b: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
        };
        let mut intersections: Vec<Intersection> = Vec::new();
        unsafe {
            for edge in edges.iter_mut() {
                if let Some(point) = point_of_intersection(
                    &candidate.a,
                    &candidate.b,
                    &(*edge.a).point,
                    &(*edge.b).point,
                ) {
                    intersections.push(Intersection { point, edge });
                }
            }
        }
        let n: usize = intersections.len();
        if n == 1 {
            /* NOTE: a---b    a--p--b
             *             ->    |
             *                   q
             */
            let intersection: Intersection = intersections.pop().unwrap();
            let edge: &mut Edge = intersection.edge;
            nodes.push(Node {
                point: intersection.point,
                neighbors: vec![edge.a, edge.b],
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            nodes.push(Node {
                point: candidate.a,
                neighbors: vec![p],
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            edge.b = p;
            unsafe {
                replace_neighbor!(*edge.a, edge.b, p);
                replace_neighbor!(*edge.b, edge.a, q);
                (*p).neighbors.push(q);
            }
            let b: *mut Node = edge.b;
            edges.push(Edge { a: p, b });
            edges.push(Edge { a: p, b: q });
            return;
        } else if 1 < n {
            /* NOTE: l.a---l.b    l.a--p--l.b
             *                 ->      |
             *       r.a---r.b    r.a--q--r.b
             */
            intersections
                .sort_by(|a, b| a.point.x.partial_cmp(&b.point.x).unwrap());
            let l_intersection: Intersection = intersections.pop().unwrap();
            let r_intersection: Intersection = intersections.pop().unwrap();
            let l_edge: &mut Edge = l_intersection.edge;
            let r_edge: &mut Edge = r_intersection.edge;
            nodes.push(Node {
                point: l_intersection.point,
                neighbors: vec![l_edge.a, l_edge.b],
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            nodes.push(Node {
                point: r_intersection.point,
                neighbors: vec![r_edge.a, r_edge.b, p],
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*l_edge.a, l_edge.b, p);
                replace_neighbor!(*l_edge.b, l_edge.a, p);
                replace_neighbor!(*r_edge.a, r_edge.b, q);
                replace_neighbor!(*r_edge.b, r_edge.a, q);
                (*p).neighbors.push(q);
            }
            l_edge.b = p;
            r_edge.b = q;
            let l_b: *mut Node = l_edge.b;
            let r_b: *mut Node = r_edge.b;
            edges.push(Edge { a: p, b: l_b });
            edges.push(Edge { a: q, b: r_b });
            edges.push(Edge { a: p, b: q });
            return;
        }
    }
}

fn main() {
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> = Uniform::new_inclusive(POINT_LOWER, POINT_UPPER);
    let mut nodes: Vec<Node> = Vec::with_capacity(CAPACITY);
    let mut edges: Vec<Edge> = Vec::with_capacity(CAPACITY);
    for _ in 0..START {
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: Vec::with_capacity(1),
        });
        let a: *mut Node = nodes.last_mut().unwrap();
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: Vec::with_capacity(1),
        });
        let b: *mut Node = nodes.last_mut().unwrap();
        unsafe {
            (*a).neighbors.push(b);
            (*b).neighbors.push(a);
        }
        edges.push(Edge { a, b })
    }
    insert(&mut rng, &range, &mut nodes, &mut edges);
}
