use arrayvec::ArrayVec;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

pub const POINTS_CAP: usize = 100;
pub const POINTS_INIT: usize = 10;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn init(
    rng: &mut ThreadRng,
    uniform: &Uniform<f64>,
    points: &mut ArrayVec<[Point; POINTS_CAP]>,
) {
    for _ in 0..POINTS_INIT {
        points.push(Point {
            x: rng.sample(uniform),
            y: rng.sample(uniform),
        });
    }
}
