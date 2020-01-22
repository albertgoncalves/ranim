#![allow(clippy::cast_possible_truncation)]

use arrayvec::ArrayVec;
use std::ptr;
use std::slice;

pub const WINDOW_EDGE: f64 = 800.0;
pub const WINDOW_EDGE_HALF: f32 = (WINDOW_EDGE as f32) / 2.0;
pub const WINDOW_EDGE_HALF_MINUS: f32 = -WINDOW_EDGE_HALF;

pub const ANTI_ALIAS: u8 = 4;

pub const LIGHT_GRAY: [f32; 4] = [0.95, 0.95, 0.95, 1.0];
pub const DARK_GRAY: [f32; 4] = [0.15, 0.15, 0.15, 1.0];
pub const RED: [f32; 4] = [0.92, 0.47, 0.47, 0.75];
pub const TEAL: [f32; 4] = [0.17, 0.82, 0.76, 0.15];

pub const LINE_WIDTH: f64 = 1.15;
pub const RADIUS: f64 = 6.0;
pub const RADIUS_2: f64 = RADIUS * 2.0;
pub const RADIUS_4: f64 = RADIUS * 4.0;

pub const RELOAD_FRAME_INTERVAL: u16 = 60 * 8;

pub const CAPACITY: usize = 100;

pub const SEARCH_RADIUS: f32 = 150.0;
pub const SEARCH_RADIUS_2: f32 = SEARCH_RADIUS * 2.0;
const SEARCH_RADIUS_SQUARED: f32 = SEARCH_RADIUS * SEARCH_RADIUS;

pub const POINT_RNG_UPPER: f32 = WINDOW_EDGE_HALF - 50.0;
pub const POINT_RNG_LOWER: f32 = -POINT_RNG_UPPER;
pub const WALK_RNG_UPPER: f32 = 0.35;
pub const WALK_RNG_LOWER: f32 = -WALK_RNG_UPPER;

pub const BOUNDS: Bounds = Bounds {
    lower: Point {
        x: WINDOW_EDGE_HALF_MINUS,
        y: WINDOW_EDGE_HALF_MINUS,
    },
    upper: Point {
        x: WINDOW_EDGE_HALF,
        y: WINDOW_EDGE_HALF,
    },
};

#[derive(Clone, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
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

pub struct Tree {
    pub point: Point,
    pub bounds: Bounds,
    pub horizontal: bool,
    pub left: *mut Tree,
    pub right: *mut Tree,
}

macro_rules! get_median {
    ($points:expr, $n:expr, $horizontal:expr $(,)?) => {{
        let median: usize = $n / 2;
        if $horizontal {
            $points.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        } else {
            $points.sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        }
        $points[median].clone()
    }};
}

pub unsafe fn make_tree(
    trees: &mut ArrayVec<[Tree; CAPACITY]>,
    points: &mut [Point],
    horizontal: bool,
    bounds: Bounds,
) -> *mut Tree {
    let mut stack: ArrayVec<[(*mut Tree, &mut [Point]); CAPACITY]> =
        ArrayVec::new();
    let n: usize = points.len();
    if 0 < n {
        let point: Point = get_median!(points, n, horizontal);
        trees.push(Tree {
            point,
            bounds,
            horizontal,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
        });
        stack.push((trees.last_mut().unwrap(), points));
    }
    while 0 < stack.len() {
        let (tree, points): (*mut Tree, &mut [Point]) = stack.pop().unwrap();
        let point: &Point = &(*tree).point;
        let horizontal: bool = (*tree).horizontal;
        let bounds: &Bounds = &(*tree).bounds;
        let lower_x: f32 = bounds.lower.x;
        let lower_y: f32 = bounds.lower.y;
        let upper_x: f32 = bounds.upper.x;
        let upper_y: f32 = bounds.upper.y;
        let (left_bounds, right_bounds): (Bounds, Bounds) = {
            if horizontal {
                let x: f32 = point.x;
                (
                    make_bounds!(lower_x, lower_y, x, upper_y),
                    make_bounds!(x, lower_y, upper_x, upper_y),
                )
            } else {
                let y: f32 = point.y;
                (
                    make_bounds!(lower_x, lower_y, upper_x, y),
                    make_bounds!(lower_x, y, upper_x, upper_y),
                )
            }
        };
        let (left_points, right_points): (&mut [Point], &mut [Point]) = {
            let n: usize = points.len();
            let median: usize = points.len() / 2;
            let points: *mut Point = points.as_mut_ptr();
            let offset: usize = median + 1;
            (
                slice::from_raw_parts_mut(points, median),
                slice::from_raw_parts_mut(points.add(offset), n - offset),
            )
        };
        if !left_points.is_empty() {
            let left_horizontal: bool = !horizontal;
            let left_point: Point =
                get_median!(left_points, left_points.len(), left_horizontal);
            trees.push(Tree {
                point: left_point,
                bounds: left_bounds,
                horizontal: left_horizontal,
                left: ptr::null_mut(),
                right: ptr::null_mut(),
            });
            let left_tree: *mut Tree = trees.last_mut().unwrap();
            (*tree).left = left_tree;
            stack.push((left_tree, left_points));
        }
        if !right_points.is_empty() {
            let right_horizontal: bool = !horizontal;
            let right_point: Point = get_median!(
                right_points,
                right_points.len(),
                right_horizontal,
            );
            trees.push(Tree {
                point: right_point,
                bounds: right_bounds,
                horizontal: right_horizontal,
                left: ptr::null_mut(),
                right: ptr::null_mut(),
            });
            let right_tree: *mut Tree = trees.last_mut().unwrap();
            (*tree).right = right_tree;
            stack.push((right_tree, right_points));
        }
    }
    if 0 < trees.len() {
        &mut trees[0]
    } else {
        ptr::null_mut()
    }
}

fn squared_distance(a: &Point, b: &Point) -> f32 {
    let x: f32 = a.x - b.x;
    let y: f32 = a.y - b.y;
    (x * x) + (y * y)
}

fn bounds_to_point_squared_distance(bounds: &Bounds, point: &Point) -> f32 {
    let x: f32 = point.x - bounds.lower.x.max(point.x.min(bounds.upper.x));
    let y: f32 = point.y - bounds.lower.y.max(point.y.min(bounds.upper.y));
    (x * x) + (y * y)
}

pub unsafe fn search_trees(
    point: &Point,
    tree: *mut Tree,
    neighbors: &mut ArrayVec<[*const Point; CAPACITY]>,
) {
    let mut stack: ArrayVec<[*mut Tree; CAPACITY]> = ArrayVec::new();
    stack.push(tree);
    while 0 < stack.len() {
        let tree: *mut Tree = stack.pop().unwrap();
        let bounds: &Bounds = &(*tree).bounds;
        if bounds_to_point_squared_distance(bounds, point)
            < SEARCH_RADIUS_SQUARED
        {
            let neighbor: &Point = &(*tree).point;
            if (point != neighbor)
                && (squared_distance(point, neighbor) < SEARCH_RADIUS_SQUARED)
            {
                neighbors.push_unchecked(neighbor);
            }
            let left: *mut Tree = (*tree).left;
            if !left.is_null() {
                stack.push(left);
            }
            let right: *mut Tree = (*tree).right;
            if !right.is_null() {
                stack.push(right);
            }
        }
    }
}
