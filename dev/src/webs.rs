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

struct Point {
    x: f64,
    y: f64,
}

/* NOTE: `neighbors` are indices into `nodes: Vec<Node>`. */
struct Node {
    point: Point,
    neighbors: Vec<usize>,
}

/* NOTE: `a` and `b` are indices into `nodes: Vec<Node>`. */
struct Edge {
    a: usize,
    b: usize,
}

struct Candidate {
    a: Point,
    b: Point,
}

struct Intersection {
    point: Point,
    index: usize,
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
        for (i, neighbor) in $x.neighbors.iter().enumerate() {
            if *neighbor == $a {
                $x.neighbors[i] = $b;
                break;
            }
        }
    };
}

#[allow(clippy::comparison_chain)]
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
        for (i, edge) in edges.iter().enumerate() {
            if let Some(point) = point_of_intersection(
                &candidate.a,
                &candidate.b,
                &nodes[edge.a].point,
                &nodes[edge.b].point,
            ) {
                intersections.push(Intersection { point, index: i });
            }
        }
        let n: usize = intersections.len();
        if n == 1 {
            /* NOTE: a---b    a--p--b
             *             ->    |
             *                   q
             */
            let intersection: Intersection = intersections.pop().unwrap();
            let edge: &Edge = &edges[intersection.index];
            let a: usize = edge.a;
            let b: usize = edge.b;
            let p: usize = nodes.len();
            let q: usize = p + 1;
            nodes.push(Node {
                point: intersection.point,
                neighbors: vec![a, b, p],
            });
            nodes.push(Node {
                point: candidate.a,
                neighbors: vec![q],
            });
            replace_neighbor!(nodes[a], b, p);
            replace_neighbor!(nodes[b], a, p);
            edges[intersection.index] = Edge { a, b: p };
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
            let l_edge: &Edge = &edges[l_intersection.index];
            let r_edge: &Edge = &edges[r_intersection.index];
            let l_a: usize = l_edge.a;
            let l_b: usize = l_edge.b;
            let r_a: usize = r_edge.a;
            let r_b: usize = r_edge.b;
            let p: usize = nodes.len();
            let q: usize = p + 1;
            nodes.push(Node {
                point: l_intersection.point,
                neighbors: vec![l_a, l_b, q],
            });
            nodes.push(Node {
                point: r_intersection.point,
                neighbors: vec![r_a, r_b, p],
            });
            replace_neighbor!(nodes[l_a], l_b, p);
            replace_neighbor!(nodes[l_b], l_a, p);
            replace_neighbor!(nodes[r_a], r_b, q);
            replace_neighbor!(nodes[r_b], r_a, q);
            edges[l_intersection.index] = Edge { a: l_a, b: p };
            edges[r_intersection.index] = Edge { a: r_a, b: q };
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
    for i in 0..START {
        let j: usize = i * 2;
        let k: usize = j + 1;
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: vec![k],
        });
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: vec![j],
        });
        edges.push(Edge { a: j, b: k })
    }
}
