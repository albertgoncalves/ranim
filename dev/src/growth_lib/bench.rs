#![allow(dead_code)]

#[macro_use]
extern crate bencher;

mod r#mod;

use arrayvec::ArrayVec;
use bencher::Bencher;
use r#mod::Node;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;

fn init_update_nodes(b: &mut Bencher) {
    b.iter(|| {
        let mut rng: ThreadRng = rand::thread_rng();
        let uniform_init: Uniform<f64> = Uniform::new_inclusive(
            r#mod::POINT_RNG_LOWER,
            r#mod::POINT_RNG_UPPER,
        );
        let uniform_walk: Uniform<f64> = Uniform::new_inclusive(
            r#mod::WALK_RNG_LOWER,
            r#mod::WALK_RNG_UPPER,
        );
        let mut nodes: ArrayVec<[Node; r#mod::CAPACITY]> = ArrayVec::new();
        r#mod::init_nodes(&mut rng, &uniform_init, &mut nodes);
        for _ in 0..r#mod::CAPACITY {
            r#mod::update_nodes(
                &mut rng,
                &uniform_walk,
                &mut nodes,
                r#mod::BOUNDS,
                r#mod::NEIGHBOR_RADIUS_SQUARED,
                r#mod::SEARCH_RADIUS_SQUARED,
                r#mod::DRAG_ATTRACT,
                r#mod::DRAG_REJECT,
            );
        }
    })
}

benchmark_group!(benches, init_update_nodes);
benchmark_main!(benches);
