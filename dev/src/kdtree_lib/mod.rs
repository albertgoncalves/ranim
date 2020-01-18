use arrayvec::ArrayVec;
use std::ptr;

pub const CAPACITY: usize = 100;

pub const SEARCH_RADIUS: f64 = 150.0;
pub const SEARCH_RADIUS_2: f64 = SEARCH_RADIUS * 2.0;
const SEARCH_RADIUS_SQUARED: f64 = SEARCH_RADIUS * SEARCH_RADIUS;

#[derive(Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub struct Bounds {
    pub lower: Point,
    pub upper: Point,
}

pub struct Tree {
    pub point: Point,
    pub bounds: Bounds,
    pub horizontal: bool,
    pub left: *const Tree,
    pub right: *const Tree,
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

pub fn construct_tree(
    trees: &mut ArrayVec<[Tree; CAPACITY]>,
    points: &mut [Point],
    horizontal: bool,
    bounds: Bounds,
) -> *const Tree {
    let n: usize = points.len();
    if n == 0 {
        return ptr::null();
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
                bounds!(lower_x, lower_y, x, upper_y),
                bounds!(x, lower_y, upper_x, upper_y),
            )
        } else {
            points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            let point: Point = points[median].clone();
            let y: f64 = point.y;
            (
                point,
                bounds!(lower_x, lower_y, upper_x, y),
                bounds!(lower_x, y, upper_x, upper_y),
            )
        }
    };
    let left: *const Tree =
        construct_tree(trees, &mut points[..median], !horizontal, left_bounds);
    let right: *const Tree = construct_tree(
        trees,
        &mut points[(median + 1)..],
        !horizontal,
        right_bounds,
    );
    unsafe {
        trees.push_unchecked(Tree {
            point,
            bounds,
            horizontal,
            left,
            right,
        });
    }
    trees.last().unwrap()
}

fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

pub unsafe fn search_tree(
    point: &Point,
    tree: *const Tree,
    neighbors: &mut ArrayVec<[*const Point; CAPACITY]>,
) {
    let bounds: &Bounds = &(*tree).bounds;
    let x: f64 = point.x - bounds.lower.x.max(point.x.min(bounds.upper.x));
    let y: f64 = point.y - bounds.lower.y.max(point.y.min(bounds.upper.y));
    if ((x * x) + (y * y)) < SEARCH_RADIUS_SQUARED {
        let neighbor: &Point = &(*tree).point;
        if (point != neighbor)
            && (squared_distance(point, neighbor) < SEARCH_RADIUS_SQUARED)
        {
            neighbors.push_unchecked(neighbor);
        }
        let left: *const Tree = (*tree).left;
        if !left.is_null() {
            search_tree(point, left, neighbors);
        }
        let right: *const Tree = (*tree).right;
        if !right.is_null() {
            search_tree(point, right, neighbors);
        }
    }
}
