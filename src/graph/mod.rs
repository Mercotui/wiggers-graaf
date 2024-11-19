use crate::board::{Board, SlideMove};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

struct Node {
    board: Board,
    // TODO(Menno 19.11.2024) edges should record moves as well as neighbours
    neighbors: Vec<u64>,
}
pub struct Graph {
    map: HashMap<u64, Node>,
}

fn to_hash(board: &Board) -> u64 {
    let mut hasher = DefaultHasher::new();
    board.hash(&mut hasher);
    hasher.finish()
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, board: Board) {
        let hash = to_hash(&board);

        let entry = self.map.get(&hash);
        if entry.is_some() {
            let entry_unwrapped = entry.unwrap();
            if entry_unwrapped.board != board {
                panic!("Hash collision! These boards are not the same, but they produce the same hash.\
                 New Board: {:?}, Existing Board: {:?}, both reduced to hash: {:?}", board, entry_unwrapped.board, hash);
            }

            // We already found an identical entry, nothing to do
            return;
        }

        self.map.insert(
            hash,
            Node {
                board,
                neighbors: Vec::new(),
            },
        );
    }

    pub fn contains_node(&self, board: &Board) -> bool {
        let hash = to_hash(&board);
        self.map.contains_key(&hash)
    }

    pub fn node_count(&self) -> usize {
        self.map.len()
    }

    pub fn add_edge(&mut self, from: &Board, to: &Board, slide_move: &SlideMove) {
        let hash_a = to_hash(&from);
        let hash_b = to_hash(&to);
        self.map.get_mut(&hash_a).expect("Inserting edge from unknown node").neighbors.push(hash_b);
    }
}
