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

const WINDOW_WIDTH: f64 = 600.0;
const WINDOW_HEIGHT: f64 = 600.0;
const ANTI_ALIAS: u8 = 4;

const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
const GREEN: [f32; 4] = [0.5, 1.0, 0.87, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

const LINE_WIDTH: f64 = 0.7;
const RADIUS: f64 = 3.5;
const RADIUS_2: f64 = RADIUS * 2.0;
const PAD: f64 = 15.0;
const PAD_2: f64 = PAD * 2.0;

const UPPER_BOUND: f64 = 350.0;
const LOWER_BOUND: f64 = -UPPER_BOUND;
const CUTOFF: f64 = 400.0;
const DRAG: f64 = 0.00175;

const INIT: usize = 2;
const CAPACITY: usize = 1000;
const THRESHOLD: usize = CAPACITY - 3;

const INTERVAL: u16 = 5;

struct Point {
    x: f64,
    y: f64,
}

struct Node {
    point: Point,
    next: Point,
    neighbors: Vec<*mut Node>,
}

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
    let x1: f64 = a.x;
    let x2: f64 = b.x;
    let x3: f64 = c.x;
    let x4: f64 = d.x;
    let y1: f64 = a.y;
    let y2: f64 = b.y;
    let y3: f64 = c.y;
    let y4: f64 = d.y;
    let denominator: f64 = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));
    if denominator != 0.0 {
        let t: f64 =
            (((x1 - x3) * (y3 - y4)) - ((y1 - y3) * (x3 - x4))) / denominator;
        let u: f64 =
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

macro_rules! empty_point {
    () => {
        Point { x: 0.0, y: 0.0 }
    };
}

fn init(
    rng: &mut ThreadRng,
    range: &Uniform<f64>,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) {
    for _ in 0..INIT {
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: Vec::with_capacity(1),
            next: empty_point!(),
        });
        let a: *mut Node = nodes.last_mut().unwrap();
        nodes.push(Node {
            point: Point {
                x: rng.sample(range),
                y: rng.sample(range),
            },
            neighbors: vec![a],
            next: empty_point!(),
        });
        let b: *mut Node = nodes.last_mut().unwrap();
        unsafe {
            (*a).neighbors.push(b);
        }
        edges.push(Edge { a, b })
    }
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
                next: empty_point!(),
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            nodes.push(Node {
                point: candidate.a,
                neighbors: vec![p],
                next: empty_point!(),
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*(edge.a), edge.b, p);
                replace_neighbor!(*(edge.b), edge.a, p);
                (*p).neighbors.push(q);
            }
            let b: *mut Node = edge.b;
            edge.b = p;
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
            let i: usize = rng.gen_range(0, n - 1);
            let l_intersection: Intersection = intersections.remove(i);
            let r_intersection: Intersection = intersections.remove(i);
            let l_edge: &mut Edge = l_intersection.edge;
            let r_edge: &mut Edge = r_intersection.edge;
            nodes.push(Node {
                point: l_intersection.point,
                neighbors: vec![l_edge.a, l_edge.b],
                next: empty_point!(),
            });
            let p: *mut Node = nodes.last_mut().unwrap();
            nodes.push(Node {
                point: r_intersection.point,
                neighbors: vec![r_edge.a, r_edge.b, p],
                next: empty_point!(),
            });
            let q: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*(l_edge.a), l_edge.b, p);
                replace_neighbor!(*(l_edge.b), l_edge.a, p);
                replace_neighbor!(*(r_edge.a), r_edge.b, q);
                replace_neighbor!(*(r_edge.b), r_edge.a, q);
                (*p).neighbors.push(q);
            }
            let l_b: *mut Node = l_edge.b;
            let r_b: *mut Node = r_edge.b;
            l_edge.b = p;
            r_edge.b = q;
            edges.push(Edge { a: p, b: l_b });
            edges.push(Edge { a: q, b: r_b });
            edges.push(Edge { a: p, b: q });
            return;
        }
    }
}

fn update(nodes: &mut Vec<Node>) {
    for node in nodes[INIT..].iter_mut() {
        let node_point: &mut Point = &mut node.point;
        let node_x: f64 = node_point.x;
        let node_y: f64 = node_point.y;
        let mut n: f64 = 0.0;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        unsafe {
            for neighbor in &node.neighbors {
                let neighbor_point: &Point = &(*(*neighbor)).point;
                if CUTOFF < squared_distance(node_point, neighbor_point) {
                    n += 1.0;
                    x += node_x - neighbor_point.x;
                    y += node_y - neighbor_point.y;
                }
            }
        }
        let next_point: &mut Point = &mut node.next;
        if 0.0 < n {
            next_point.x = node_x - ((x / n) * DRAG);
            next_point.y = node_y - ((y / n) * DRAG);
        } else {
            next_point.x = node_x;
            next_point.y = node_y;
        }
    }
    for node in nodes.iter_mut() {
        let node_point: &mut Point = &mut node.point;
        let next_point: &Point = &node.next;
        node_point.x = next_point.x;
        node_point.y = next_point.y;
    }
}

fn bounding_box(a: &Point, b: &Point) -> (f64, f64, f64, f64) {
    let x1: f64 = a.x;
    let x2: f64 = b.x;
    let y1: f64 = a.y;
    let y2: f64 = b.y;
    let (min_x, width): (f64, f64) = {
        if x1 < x2 {
            (x1, x2 - x1)
        } else {
            (x2, a.x - x2)
        }
    };
    let (min_y, height): (f64, f64) = {
        if y1 < y2 {
            (y1, y2 - y1)
        } else {
            (y2, y1 - y2)
        }
    };
    (min_x, min_y, width, height)
}

fn render(gl: &mut GlGraphics, args: &RenderArgs, edges: &[Edge]) {
    let n: usize = edges.len();
    gl.draw(args.viewport(), |context, gl| {
        let transform: Matrix2d = context
            .transform
            .trans(args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        graphics::clear(DARK_GRAY, gl);
        unsafe {
            {
                let edge: &Edge = &edges[n - 1];
                let a: &Point = &(*edge.a).point;
                let b: &Point = &(*edge.b).point;
                let (min_x, min_y, width, height): (f64, f64, f64, f64) =
                    bounding_box(a, b);
                graphics::rectangle(
                    CYAN,
                    [min_x - PAD, min_y - PAD, width + PAD_2, height + PAD_2],
                    transform,
                    gl,
                );
                graphics::line(
                    GREEN,
                    LINE_WIDTH,
                    [a.x, a.y, b.x, b.y],
                    transform,
                    gl,
                );
                graphics::ellipse(
                    GREEN,
                    [a.x - RADIUS, a.y - RADIUS, RADIUS_2, RADIUS_2],
                    transform,
                    gl,
                );
                graphics::ellipse(
                    GREEN,
                    [b.x - RADIUS, b.y - RADIUS, RADIUS_2, RADIUS_2],
                    transform,
                    gl,
                );
            }
            for edge in edges.iter().take(n - 1) {
                let a: &Point = &(*edge.a).point;
                let b: &Point = &(*edge.b).point;
                graphics::line(
                    LIGHT_GRAY,
                    LINE_WIDTH,
                    [a.x, a.y, b.x, b.y],
                    transform,
                    gl,
                );
            }
        }
    });
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
    let range: Uniform<f64> = Uniform::new_inclusive(LOWER_BOUND, UPPER_BOUND);
    let mut nodes: Vec<Node> = Vec::with_capacity(CAPACITY);
    let mut edges: Vec<Edge> = Vec::with_capacity(CAPACITY);
    let mut counter: u16 = 0;
    init(&mut rng, &range, &mut nodes, &mut edges);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            if (THRESHOLD < nodes.len()) || (THRESHOLD < edges.len()) {
                nodes.clear();
                edges.clear();
                init(&mut rng, &range, &mut nodes, &mut edges);
            } else if INTERVAL < counter {
                insert(&mut rng, &range, &mut nodes, &mut edges);
                counter = 0;
            }
            update(&mut nodes);
            render(&mut gl, &args, &edges);
            counter += 1;
        }
    }
}
