#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::{Edge, Node};
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;

const WINDOW_EDGE: f64 = 800.0;
const WINDOW_EDGE_HALF: f64 = WINDOW_EDGE / 2.0;
const WINDOW_EDGE_HALF_MINUS: f64 = -WINDOW_EDGE_HALF;

const POINT_RNG_UPPER: f64 = WINDOW_EDGE_HALF;
const POINT_RNG_LOWER: f64 = WINDOW_EDGE_HALF_MINUS;

const POINT_DRAG: f64 = 0.0025;
const NEIGHBOR_DISTANCE_SQUARED: f64 = 100.0;

fn init_insert_update(b: &mut Bencher) {
    let mut rng: ThreadRng = rand::thread_rng();
    let uniform: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    b.iter(|| {
        let mut nodes: ArrayVec<[Node; r#mod::NODES_CAP]> = ArrayVec::new();
        let mut edges: ArrayVec<[Edge; r#mod::EDGES_CAP]> = ArrayVec::new();
        unsafe {
            r#mod::init(&mut rng, &uniform, &mut nodes, &mut edges);
            r#mod::insert(&mut rng, &uniform, &mut nodes, &mut edges);
            r#mod::update(&mut nodes, NEIGHBOR_DISTANCE_SQUARED, POINT_DRAG);
        };
    })
}

benchmark_group!(benches, init_insert_update);
benchmark_main!(benches);
