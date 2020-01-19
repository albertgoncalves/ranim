use arrayvec::ArrayVec;
use rand::distributions::Uniform;
use rand::rngs::ThreadRng;
use rand::Rng;

pub const NODES_CAP: usize = 100;
pub const NODES_INIT: usize = 10;

const NODES_INIT_LIMIT: usize = NODES_INIT - 1;
const NODES_CAP_LIMIT: usize = NODES_CAP - 1;

const NEIGHBORS_CAP: usize = 10;

pub struct Point {
    pub x: f64,
    pub y: f64,
}

type NodeIndex = usize;

pub struct Node {
    pub point: Point,
    index: NodeIndex,
    pub left_index: NodeIndex,
    pub right_index: NodeIndex,
    neighbors: Vec<NodeIndex>,
}

pub fn init(
    rng: &mut ThreadRng,
    uniform: &Uniform<f64>,
    nodes: &mut ArrayVec<[Node; NODES_CAP]>,
) {
    for i in 0..NODES_INIT {
        let (left_index, index, right_index): (
            NodeIndex,
            NodeIndex,
            NodeIndex,
        ) = {
            if i == 0 {
                (NODES_INIT_LIMIT, i, i + 1)
            } else if i == NODES_INIT_LIMIT {
                (i - 1, i, 0)
            } else {
                (i - 1, i, i + 1)
            }
        };
        nodes.push(Node {
            point: Point {
                x: rng.sample(uniform),
                y: rng.sample(uniform),
            },
            index,
            left_index,
            right_index,
            neighbors: Vec::with_capacity(NEIGHBORS_CAP),
        });
    }
}

pub fn insert(nodes: &mut ArrayVec<[Node; NODES_CAP]>, left_index: NodeIndex) {
    let index: usize = nodes.len();
    if index < NODES_CAP_LIMIT {
        let right_index: NodeIndex = nodes[left_index].right_index;
        let left_point: &Point = &nodes[left_index].point;
        let right_point: &Point = &nodes[right_index].point;
        let left_x: f64 = left_point.x;
        let left_y: f64 = left_point.y;
        let right_x: f64 = right_point.x;
        let right_y: f64 = right_point.y;
        nodes.push(Node {
            point: Point {
                x: (left_x + right_x) / 2.0,
                y: (left_y + right_y) / 2.0,
            },
            index,
            left_index,
            right_index,
            neighbors: Vec::with_capacity(NEIGHBORS_CAP),
        });
        nodes[left_index].right_index = index;
        nodes[right_index].left_index = index;
    }
}
