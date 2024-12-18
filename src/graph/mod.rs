#[cfg(test)]
mod unittest;

use crate::board::{to_hash, Board, SlideMove};
use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Node {
    pub board: Board,
    // TODO(Menno 19.11.2024) edges should record moves as well as neighbours
    #[wasm_bindgen(getter_with_clone)]
    pub neighbors: Vec<u64>,
    pub distance_to_start: Option<u32>,
    pub distance_to_solution: Option<u32>,
    pub on_shortest_path: bool,
}

pub struct Graph {
    pub map: HashMap<u64, Node>,
    pub max_distance_to_start: u32,
    pub max_distance_to_solution: u32,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            map: HashMap::new(),
            max_distance_to_start: 0,
            max_distance_to_solution: 0,
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
                distance_to_start: None,
                distance_to_solution: None,
                on_shortest_path: false,
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
        self.map
            .get_mut(&hash_a)
            .expect("Inserting edge from unknown node")
            .neighbors
            .push(hash_b);
    }

    pub fn analyze(&mut self, start: &Board, solution: &Board) {
        // Find distances from start board
        self.max_distance_to_start = self.distance_from(to_hash(start), |node, distance| {
            if node.distance_to_start.is_some() {
                // This node was already visited
                return false;
            }
            node.distance_to_start = Some(*distance);
            return true;
        });

        // Find distances from solution board
        self.max_distance_to_solution = self.distance_from(to_hash(solution), |node, distance| {
            if node.distance_to_solution.is_some() {
                // This node was already visited
                return false;
            }
            node.distance_to_solution = Some(*distance);
            return true;
        });

        // TODO (Menno 10.12.2024) annotate boards that are part of fasted solution
        println!(
            "Minimum moves from start to solution is {:?}",
            self.map
                .get(&to_hash(start))
                .expect("Huh how did that happen")
                .distance_to_solution
        );
        println!(
            "Minimum moves from solution to start is {:?}",
            self.map
                .get(&to_hash(solution))
                .expect("Huh how did that happen")
                .distance_to_start
        );
    }

    /// Do a breadth first traversal, counting distance from a starting point. Returns max distance
    fn distance_from<Pred>(&mut self, from: u64, pred: Pred) -> u32
    where
        Pred: Fn(&mut Node, &u32) -> bool,
    {
        struct QueueEntry {
            key: u64,
            distance_from: u32,
        }
        // Create a queue with board keys and their corresponding distance to the start staring point.
        // We initialize the queue with the starting board, which has 0 distance to itself.
        let mut inspection_queue: VecDeque<QueueEntry> = VecDeque::from([QueueEntry {
            key: from,
            distance_from: 0,
        }]);
        let mut max_distance_from = 0;

        while !inspection_queue.is_empty() {
            let entry = inspection_queue
                .pop_front()
                .expect("Failed to pop from queue, is it empty?");
            let node = self
                .map
                .get_mut(&entry.key)
                .expect("Graph does not contain this board.");

            // Run predicate
            if !pred(node, &entry.distance_from) {
                // Predicate claims this node was already visited, ignore it.
                continue;
            }

            if max_distance_from < entry.distance_from {
                max_distance_from = entry.distance_from;
            }

            let neighbors_distance_from = entry.distance_from + 1;
            for neighbor in node.neighbors.iter() {
                inspection_queue.push_back(QueueEntry {
                    key: *neighbor,
                    distance_from: neighbors_distance_from,
                });
            }
        }
        max_distance_from
    }

    /// Do a breadth first traversal on only the shortest paths between from and to
    fn shortest_path(&mut self, from: u64, to: u64) {
        struct QueueEntry {
            key: u64,
            distance_from: u32,
        }
        // Create a queue with board keys and their corresponding distance to the start staring point.
        // We initialize the queue with the starting board, which has 0 distance to itself.
        let mut inspection_queue: VecDeque<QueueEntry> = VecDeque::from([QueueEntry {
            key: from,
            distance_from: 0,
        }]);

        while !inspection_queue.is_empty() {
            let entry = inspection_queue
                .pop_front()
                .expect("Failed to pop from queue, is it empty?");
            let node = self
                .map
                .get_mut(&entry.key)
                .expect("Graph does not contain this board.");

            // TODO breadth first shortest paths

            let neighbors_distance_from = entry.distance_from + 1;
            for neighbor in node.neighbors.iter() {
                inspection_queue.push_back(QueueEntry {
                    key: *neighbor,
                    distance_from: neighbors_distance_from,
                });
            }
        }
    }
}
