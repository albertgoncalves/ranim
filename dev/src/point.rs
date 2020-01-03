#[derive(PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[macro_export]
macro_rules! empty_point {
    () => {
        Point { x: 0.0, y: 0.0 }
    };
}

pub fn squared_distance(a: &Point, b: &Point) -> f64 {
    let x: f64 = a.x - b.x;
    let y: f64 = a.y - b.y;
    (x * x) + (y * y)
}

#[allow(clippy::many_single_char_names)]
pub fn intersection(
    a: &Point,
    b: &Point,
    c: &Point,
    d: &Point,
) -> Option<Point> {
    /* NOTE: `a` `c`
     *         \ /
     *          X
     *         / \
     *       `d` `b`
     */
    let x1: f64 = a.x;
    let x2: f64 = b.x;
    let x3: f64 = c.x;
    let x4: f64 = d.x;
    let y1: f64 = a.y;
    let y2: f64 = b.y;
    let y3: f64 = c.y;
    let y4: f64 = d.y;
    let denominator: f64 = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));
    if denominator != 0.0 {
        let t: f64 =
            (((x1 - x3) * (y3 - y4)) - ((y1 - y3) * (x3 - x4))) / denominator;
        let u: f64 =
            -(((x1 - x2) * (y1 - y3)) - ((y1 - y2) * (x1 - x3))) / denominator;
        if (0.0 <= t) && (t <= 1.0) && (0.0 <= u) && (u <= 1.0) {
            return Some(Point {
                x: x1 + (t * (x2 - x1)),
                y: y1 + (t * (y2 - y1)),
            });
        }
    }
    None
}
