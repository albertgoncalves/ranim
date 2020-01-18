#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::{Edge, Node};
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;

const POINT_RNG_UPPER: f64 = 1.0;
const POINT_RNG_LOWER: f64 = 0.0;

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
            r#mod::update(&mut nodes);
        };
    })
}

benchmark_group!(benches, init_insert_update);
benchmark_main!(benches);
