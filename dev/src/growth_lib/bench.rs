#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::{Node, Point};
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;

const WINDOW_EDGE: f64 = 800.0;
const WINDOW_EDGE_HALF: f64 = WINDOW_EDGE / 2.0;
const WINDOW_EDGE_HALF_MINUS: f64 = -WINDOW_EDGE_HALF;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF / 3.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;
const WALK_RNG_UPPER: f64 = 0.15;
const WALK_RNG_LOWER: f64 = -WALK_RNG_UPPER;

const NEIGHBOR_RADIUS_SQUARED: f64 = 1000.0;
const SEARCH_RADIUS_SQUARED: f64 = 2000.0;

const DRAG_ATTRACT: f64 = 35.0;
const DRAG_REJECT: f64 = 25.0;

const BOUNDS: r#mod::Bounds = r#mod::Bounds {
    lower: Point {
        x: WINDOW_EDGE_HALF_MINUS,
        y: WINDOW_EDGE_HALF_MINUS,
    },
    upper: Point {
        x: WINDOW_EDGE_HALF,
        y: WINDOW_EDGE_HALF,
    },
};

fn init_update_nodes(b: &mut Bencher) {
    b.iter(|| {
        let mut rng: ThreadRng = rand::thread_rng();
        let uniform_init: Uniform<f64> =
            Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
        let uniform_walk: Uniform<f64> =
            Uniform::new_inclusive(WALK_RNG_LOWER, WALK_RNG_UPPER);
        let mut nodes: ArrayVec<[Node; r#mod::CAPACITY]> = ArrayVec::new();
        r#mod::init_nodes(&mut rng, &uniform_init, &mut nodes);
        for _ in 0..r#mod::CAPACITY {
            r#mod::update_nodes(
                &mut rng,
                &uniform_walk,
                &mut nodes,
                BOUNDS,
                NEIGHBOR_RADIUS_SQUARED,
                SEARCH_RADIUS_SQUARED,
                DRAG_ATTRACT,
                DRAG_REJECT,
            );
        }
    })
}

benchmark_group!(benches, init_update_nodes);
benchmark_main!(benches);
