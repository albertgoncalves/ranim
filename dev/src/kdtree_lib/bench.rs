#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::{Point, Tree};
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

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
    let uniform: Uniform<f32> =
        Uniform::new_inclusive(r#mod::POINT_RNG_LOWER, r#mod::POINT_RNG_UPPER);
    let mut points: ArrayVec<[Point; r#mod::CAPACITY]> =
        make_points!(rng, uniform);
    b.iter(|| {
        let mut trees: ArrayVec<[Tree; r#mod::CAPACITY]> = ArrayVec::new();
        unsafe {
            r#mod::make_tree(&mut trees, &mut points, true, r#mod::BOUNDS);
        }
    })
}

fn search_trees(b: &mut Bencher) {
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f32> =
        Uniform::new_inclusive(r#mod::POINT_RNG_LOWER, r#mod::POINT_RNG_UPPER);
    let mut points: ArrayVec<[Point; r#mod::CAPACITY]> =
        make_points!(rng, uniform);
    let point: Point = make_point!(rng, uniform);
    let mut trees: ArrayVec<[Tree; r#mod::CAPACITY]> = ArrayVec::new();
    unsafe {
        let tree: *mut Tree =
            r#mod::make_tree(&mut trees, &mut points, true, r#mod::BOUNDS);
        b.iter(|| {
            let mut neighbors: ArrayVec<[*const Point; r#mod::CAPACITY]> =
                ArrayVec::new();
            r#mod::search_trees(&point, tree, &mut neighbors)
        })
    }
}

benchmark_group!(benches, make_tree, search_trees);
benchmark_main!(benches);
