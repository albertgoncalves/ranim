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
const GREEN: [f32; 4] = [0.5, 1.0, 0.87, 1.0];
const CYAN: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

const LINE_WIDTH: f64 = 0.8;
const RADIUS: f64 = 3.5;
const RADIUS_2: f64 = RADIUS * 2.0;
const PAD: f64 = 17.5;
const PAD_2: f64 = PAD * 2.0;

const UPPER_BOUND: f64 = 400.0;
const LOWER_BOUND: f64 = -UPPER_BOUND;
const CUTOFF: f64 = 100.0;
const DRAG: f64 = 0.0025;

const INIT_EDGES: usize = 1;
const INIT_NODES: usize = INIT_EDGES * 2;
const CAPACITY: usize = 512;
const THRESHOLD: usize = CAPACITY - 3;
const NEIGHBORS: usize = 3;
const INTERSECTIONS: usize = 16;

const INTERVAL: u16 = 10;

#[derive(PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

struct Node {
    point: Point,
    next: Point,
    neighbors: ArrayVec<[*mut Node; NEIGHBORS]>,
}

struct Edge {
    a: *mut Node,
    b: *mut Node,
}

struct Intersection<'a> {
    point: Point,
    edge: &'a mut Edge,
}

struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

macro_rules! empty_point {
    () => {
        Point { x: 0.0, y: 0.0 }
    };
}

fn init(
    rng: &mut ThreadRng,
    range: &Uniform<f64>,
    nodes: &mut ArrayVec<[Node; CAPACITY]>,
    edges: &mut ArrayVec<[Edge; CAPACITY]>,
) {
    for _ in 0..INIT_EDGES {
        unsafe {
            nodes.push_unchecked(Node {
                point: Point {
                    x: rng.sample(range),
                    y: rng.sample(range),
                },
                neighbors: ArrayVec::new(),
                next: empty_point!(),
            });
        }
        let a: *mut Node = nodes.last_mut().unwrap();
        unsafe {
            nodes.push_unchecked(Node {
                point: Point {
                    x: rng.sample(range),
                    y: rng.sample(range),
                },
                neighbors: ArrayVec::new(),
                next: empty_point!(),
            });
        }
        let b: *mut Node = nodes.last_mut().unwrap();
        unsafe {
            (*b).neighbors.push_unchecked(a);
            (*a).neighbors.push_unchecked(b);
        }
        unsafe { edges.push_unchecked(Edge { a, b }) }
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

#[allow(clippy::many_single_char_names)]
fn intersection(a: &Point, b: &Point, c: &Point, d: &Point) -> Option<Point> {
    /* NOTE:     `a`
     *            |
     *       `c`--+--`d`
     *            |
     *           `b`
     */
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

#[allow(clippy::comparison_chain)]
fn insert(
    rng: &mut ThreadRng,
    range: &Uniform<f64>,
    nodes: &mut ArrayVec<[Node; CAPACITY]>,
    edges: &mut ArrayVec<[Edge; CAPACITY]>,
) {
    loop {
        let candidate_a: Point = Point {
            x: rng.sample(range),
            y: rng.sample(range),
        };
        let candidate_b: Point = Point {
            x: rng.sample(range),
            y: rng.sample(range),
        };
        let mut intersections: Vec<Intersection> =
            Vec::with_capacity(INTERSECTIONS);
        unsafe {
            for edge in edges.iter_mut() {
                if let Some(point) = intersection(
                    &candidate_a,
                    &candidate_b,
                    &(*edge.a).point,
                    &(*edge.b).point,
                ) {
                    intersections.push(Intersection { point, edge });
                }
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
            unsafe {
                nodes.push_unchecked(Node {
                    point: candidate_a,
                    neighbors: ArrayVec::new(),
                    next: empty_point!(),
                });
            }
            let q: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                nodes.push_unchecked(Node {
                    point: intersection.point,
                    neighbors: ArrayVec::from([edge.a, edge.b, q]),
                    next: empty_point!(),
                });
            }
            let p: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*edge.a, edge.b, p);
                replace_neighbor!(*edge.b, edge.a, p);
                (*q).neighbors.push_unchecked(p);
            }
            let b: *mut Node = edge.b;
            edge.b = p;
            unsafe {
                edges.push_unchecked(Edge { a: p, b });
                edges.push_unchecked(Edge { a: p, b: q });
            }
            return;
        } else if 1 < n {
            /* NOTE: `l.a`---`l.b`    `l.a`--`p`--`l.b`
             *                     ->         |
             *       `r.a`---`r.b`    `r.a`--`q`--`r.b`
             */
            intersections
                .sort_by(|a, b| a.point.x.partial_cmp(&b.point.x).unwrap());
            let i: usize = rng.gen_range(0, n - 1);
            let l_intersection: Intersection = intersections.remove(i);
            let r_intersection: Intersection = intersections.remove(i);
            let l_edge: &mut Edge = l_intersection.edge;
            let r_edge: &mut Edge = r_intersection.edge;
            unsafe {
                nodes.push_unchecked(Node {
                    point: r_intersection.point,
                    neighbors: ArrayVec::new(),
                    next: empty_point!(),
                });
            }
            let q: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                nodes.push_unchecked(Node {
                    point: l_intersection.point,
                    neighbors: ArrayVec::from([l_edge.a, l_edge.b, q]),
                    next: empty_point!(),
                });
            }
            let p: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*l_edge.a, l_edge.b, p);
                replace_neighbor!(*l_edge.b, l_edge.a, p);
                replace_neighbor!(*r_edge.a, r_edge.b, q);
                replace_neighbor!(*r_edge.b, r_edge.a, q);
                (*q).neighbors.push_unchecked(p);
            }
            let l_b: *mut Node = l_edge.b;
            let r_b: *mut Node = r_edge.b;
            l_edge.b = p;
            r_edge.b = q;
            unsafe {
                edges.push_unchecked(Edge { a: p, b: l_b });
                edges.push_unchecked(Edge { a: q, b: r_b });
                edges.push_unchecked(Edge { a: p, b: q });
            }
            return;
        }
    }
}

fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

fn update(nodes: &mut ArrayVec<[Node; CAPACITY]>) {
    for node in nodes[INIT_NODES..].iter_mut() {
        let node_point: &mut Point = &mut node.point;
        let node_x: f64 = node_point.x;
        let node_y: f64 = node_point.y;
        let mut n: f64 = 0.0;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        unsafe {
            for neighbor in &node.neighbors {
                let neighbor_point: &Point = &(**neighbor).point;
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
    for node in nodes[INIT_NODES..].iter_mut() {
        let node_point: &mut Point = &mut node.point;
        let next_point: &Point = &node.next;
        node_point.x = next_point.x;
        node_point.y = next_point.y;
    }
}

fn bounds(a: &Point, b: &Point) -> Rect {
    let x1: f64 = a.x;
    let x2: f64 = b.x;
    let y1: f64 = a.y;
    let y2: f64 = b.y;
    let (x, width): (f64, f64) = {
        if x1 < x2 {
            (x1, x2 - x1)
        } else {
            (x2, a.x - x2)
        }
    };
    let (y, height): (f64, f64) = {
        if y1 < y2 {
            (y1, y2 - y1)
        } else {
            (y2, y1 - y2)
        }
    };
    Rect {
        x,
        y,
        width,
        height,
    }
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
                let rect: Rect = bounds(a, b);
                graphics::rectangle(
                    CYAN,
                    [
                        rect.x - PAD,
                        rect.y - PAD,
                        rect.width + PAD_2,
                        rect.height + PAD_2,
                    ],
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
    let mut nodes: ArrayVec<[Node; CAPACITY]> = ArrayVec::new();
    let mut edges: ArrayVec<[Edge; CAPACITY]> = ArrayVec::new();
    let mut counter: u16 = 0;
    init(&mut rng, &range, &mut nodes, &mut edges);
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
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
