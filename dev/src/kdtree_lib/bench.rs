#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::{Bounds, Point, Tree};
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

const POINT_RNG_UPPER: f64 = 1.0;
const POINT_RNG_LOWER: f64 = 0.0;

const BOUNDS: Bounds = Bounds {
    lower: Point {
        x: POINT_RNG_LOWER,
        y: POINT_RNG_LOWER,
    },
    upper: Point {
        x: POINT_RNG_UPPER,
        y: POINT_RNG_UPPER,
    },
};

macro_rules! make_point {
    ($rng:expr, $uniform:expr $(,)?) => {
        Point {
            x: $rng.sample($uniform),
            y: $rng.sample($uniform),
        }
    };
}

macro_rules! make_points {
    ($rng:expr, $uniform:expr $(,)?) => {{
        let mut points: ArrayVec<[Point; r#mod::CAPACITY]> = ArrayVec::new();
        unsafe {
            for _ in 0..r#mod::CAPACITY {
                points.push_unchecked(make_point!($rng, $uniform))
            }
        }
        points
    }};
}

fn make_tree(b: &mut Bencher) {
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut points: ArrayVec<[Point; r#mod::CAPACITY]> =
        make_points!(rng, uniform);
    b.iter(|| {
        let mut trees: ArrayVec<[Tree; r#mod::CAPACITY]> = ArrayVec::new();
        r#mod::make_tree(&mut trees, &mut points, true, BOUNDS);
    })
}

fn search_tree(b: &mut Bencher) {
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    let mut points: ArrayVec<[Point; r#mod::CAPACITY]> =
        make_points!(rng, uniform);
    let point: Point = make_point!(rng, uniform);
    let mut trees: ArrayVec<[Tree; r#mod::CAPACITY]> = ArrayVec::new();
    let tree: *const Tree =
        r#mod::make_tree(&mut trees, &mut points, true, BOUNDS);
    b.iter(|| {
        let mut neighbors: ArrayVec<[*const Point; r#mod::CAPACITY]> =
            ArrayVec::new();
        unsafe { r#mod::search_tree(&point, tree, &mut neighbors) }
    })
}

benchmark_group!(benches, make_tree, search_tree);
benchmark_main!(benches);
