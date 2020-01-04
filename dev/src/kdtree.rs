use arrayvec::ArrayVec;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

const POINT_RNG_UPPER: f64 = 200.0;
const POINT_RNG_LOWER: f64 = -POINT_RNG_UPPER;

const CAPACITY: usize = 5;

#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Bounds {
    lower: Point,
    upper: Point,
}

#[derive(Debug)]
struct Tree {
    point: Point,
    bounds: Bounds,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
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
    }
}

fn construct(
    points: &mut [Point],
    horizontal: bool,
    bounds: Bounds,
) -> Option<Box<Tree>> {
    let n: usize = points.len();
    if n == 0 {
        return None;
    }
    if horizontal {
        points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        let median: usize = n / 2;
        let point: Point = points[median].clone();
        let left_bounds: Bounds = bounds!(
            bounds.lower.x,
            bounds.lower.y,
            point.x,
            bounds.upper.y,
        );
        let right_bounds: Bounds = bounds!(
            point.x,
            bounds.lower.y,
            bounds.upper.x,
            bounds.upper.y,
        );
        Some(Box::new(Tree {
            point,
            bounds,
            left: construct(&mut points[..median], !horizontal, left_bounds),
            right: construct(
                &mut points[(median + 1)..],
                !horizontal,
                right_bounds,
            ),
        }))
    } else {
        points.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
        let median: usize = n / 2;
        let point: Point = points[median].clone();
        let left_bounds: Bounds = bounds!(
            bounds.lower.x,
            bounds.lower.y,
            bounds.upper.x,
            point.y,
        );
        let right_bounds: Bounds = bounds!(
            bounds.lower.x,
            point.y,
            bounds.upper.x,
            bounds.upper.y,
        );
        Some(Box::new(Tree {
            point,
            bounds,
            left: construct(&mut points[..median], !horizontal, left_bounds),
            right: construct(
                &mut points[(median + 1)..],
                !horizontal,
                right_bounds,
            ),
        }))
    }
}

fn main() {
    let mut rng: ThreadRng = rand::thread_rng();
    let range: Uniform<f64> =
        Uniform::new_inclusive(POINT_RNG_LOWER, POINT_RNG_UPPER);
    macro_rules! point {
        () => {
            Point {
                x: rng.sample(range),
                y: rng.sample(range),
            }
        };
    }
    let mut points: ArrayVec<[Point; CAPACITY]> = ArrayVec::new();
    for _ in 0..CAPACITY {
        points.push(point!());
    }
    let bounds: Bounds = Bounds {
        lower: Point {
            x: POINT_RNG_LOWER,
            y: POINT_RNG_LOWER,
        },
        upper: Point {
            x: POINT_RNG_UPPER,
            y: POINT_RNG_UPPER,
        },
    };
    let tree: Option<Box<Tree>> = construct(&mut points, true, bounds);
    println!("{:#?}\n{:#?}", points, tree);
}
