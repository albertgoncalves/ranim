use arrayvec::ArrayVec;
use graphics::math::Matrix2d;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent};
use piston::window::WindowSettings;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2_window::Sdl2Window;

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
const CYAN: [f32; 4] = [0.5, 1.0, 0.87, 1.0];
const TEAL: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

const LINE_WIDTH: f64 = 0.8;
const RADIUS: f64 = 3.5;
const RADIUS_2: f64 = RADIUS * 2.0;
const RECT_PAD: f64 = 17.5;
const RECT_PAD_2: f64 = RECT_PAD * 2.0;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF;
const POINT_RNG_LOWER: f64 = WINDOW_EDGE_HALF_MINUS;
const POINT_DRAG: f64 = 0.0025;
const NEIGHBOR_DISTANCE_SQUARED: f64 = 100.0;

const EDGES_CAP: usize = 512;
const EDGES_LIMIT: usize = EDGES_CAP - 3;
const EDGES_INIT: usize = 1;
const NODES_CAP: usize = 512;
const NODES_LIMIT: usize = NODES_CAP - 2;
const NODES_INIT: usize = EDGES_INIT * 2;
const NEIGHBORS_CAP: usize = 3;
const INTERSECTIONS_CAP: usize = 16;

const INSERT_FRAME_INTERVAL: u16 = 10;

#[derive(PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

struct Node {
    point: Point,
    next: Point,
    neighbors: ArrayVec<[*mut Node; NEIGHBORS_CAP]>,
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
    nodes: &mut ArrayVec<[Node; NODES_CAP]>,
    edges: &mut ArrayVec<[Edge; EDGES_CAP]>,
) {
    for _ in 0..EDGES_INIT {
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
            edges.push_unchecked(Edge { a, b });
        }
    }
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

#[allow(
    clippy::comparison_chain,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]
fn insert(
    rng: &mut ThreadRng,
    range: &Uniform<f64>,
    nodes: &mut ArrayVec<[Node; NODES_CAP]>,
    edges: &mut ArrayVec<[Edge; EDGES_CAP]>,
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
            Vec::with_capacity(INTERSECTIONS_CAP);
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
            let a: *mut Node = edge.a;
            let b: *mut Node = edge.b;
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
                    neighbors: ArrayVec::from([a, b, q]),
                    next: empty_point!(),
                });
            }
            let p: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*a, b, p);
                replace_neighbor!(*b, a, p);
                (*q).neighbors.push_unchecked(p);
            }
            (*edge).b = p;
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
            let l_a: *mut Node = (*l_edge).a;
            let l_b: *mut Node = (*l_edge).b;
            let r_a: *mut Node = (*r_edge).a;
            let r_b: *mut Node = (*r_edge).b;
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
                    neighbors: ArrayVec::from([l_a, l_b, q]),
                    next: empty_point!(),
                });
            }
            let p: *mut Node = nodes.last_mut().unwrap();
            unsafe {
                replace_neighbor!(*l_a, l_b, p);
                replace_neighbor!(*l_b, l_a, p);
                replace_neighbor!(*r_a, r_b, q);
                replace_neighbor!(*r_b, r_a, q);
                (*q).neighbors.push_unchecked(r_a);
                (*q).neighbors.push_unchecked(r_b);
                (*q).neighbors.push_unchecked(p);
            }
            (*l_edge).b = p;
            (*r_edge).b = q;
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

fn update(nodes: &mut ArrayVec<[Node; NODES_CAP]>) {
    for node in nodes[NODES_INIT..].iter_mut() {
        let node_point: &mut Point = &mut node.point;
        let node_x: f64 = node_point.x;
        let node_y: f64 = node_point.y;
        let mut n: f64 = 0.0;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        unsafe {
            for neighbor in &node.neighbors {
                let neighbor_point: &Point = &(**neighbor).point;
                if NEIGHBOR_DISTANCE_SQUARED
                    < squared_distance(node_point, neighbor_point)
                {
                    n += 1.0;
                    x += node_x - neighbor_point.x;
                    y += node_y - neighbor_point.y;
                }
            }
        }
        let next_point: &mut Point = &mut node.next;
        if 0.0 < n {
            next_point.x = node_x - ((x / n) * POINT_DRAG);
            next_point.y = node_y - ((y / n) * POINT_DRAG);
        } else {
            next_point.x = node_x;
            next_point.y = node_y;
        }
    }
    for node in nodes[NODES_INIT..].iter_mut() {
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
            (x2, x1 - x2)
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
    let n: usize = edges.len() - 1;
    gl.draw(args.viewport(), |context, gl| {
        let [width, height]: [f64; 2] = args.window_size;
        let transform: Matrix2d =
            context.transform.trans(width / 2.0, height / 2.0);
        graphics::clear(LIGHT_GRAY, gl);
        graphics::rectangle(DARK_GRAY, WINDOW_RECT, transform, gl);
        unsafe {
            {
                let edge: &Edge = &edges[n];
                let a: &Point = &(*edge.a).point;
                let b: &Point = &(*edge.b).point;
                let rect: Rect = bounds(a, b);
                graphics::rectangle(
                    TEAL,
                    [
                        rect.x - RECT_PAD,
                        rect.y - RECT_PAD,
                        rect.width + RECT_PAD_2,
                        rect.height + RECT_PAD_2,
                    ],
                    transform,
                    gl,
                );
                graphics::line(
                    CYAN,
                    LINE_WIDTH,
                    [a.x, a.y, b.x, b.y],
                    transform,
                    gl,
                );
                graphics::ellipse(
                    CYAN,
                    [a.x - RADIUS, a.y - RADIUS, RADIUS_2, RADIUS_2],
                    transform,
                    gl,
                );
                graphics::ellipse(
                    CYAN,
                    [b.x - RADIUS, b.y - RADIUS, RADIUS_2, RADIUS_2],
                    transform,
                    gl,
                );
            }
            for edge in edges.iter().take(n) {
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
        WindowSettings::new("ranim", [WINDOW_EDGE, WINDOW_EDGE])
            .graphics_api(opengl)
            .exit_on_esc(true);
    settings.set_samples(ANTI_ALIAS);
    let mut window: Sdl2Window = settings.build().unwrap();
    let mut events: Events = Events::new(EventSettings::new());
    let mut gl: GlGraphics = GlGraphics::new(opengl);
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut nodes: ArrayVec<[Node; NODES_CAP]> = ArrayVec::new();
    let mut edges: ArrayVec<[Edge; EDGES_CAP]> = ArrayVec::new();
    let mut counter: u16 = 0;
    init(&mut rng, &range, &mut nodes, &mut edges);
    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            if (NODES_LIMIT < nodes.len()) || (EDGES_LIMIT < edges.len()) {
                nodes.clear();
                edges.clear();
                init(&mut rng, &range, &mut nodes, &mut edges);
            } else if INSERT_FRAME_INTERVAL < counter {
                insert(&mut rng, &range, &mut nodes, &mut edges);
                counter = 0;
            }
            update(&mut nodes);
            render(&mut gl, &args, &edges);
            counter += 1;
        }
    }
}
