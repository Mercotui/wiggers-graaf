mod board;
mod solver;
mod graph;

use std::time::Instant;
use crate::solver::Solver;

fn main() {
    env_logger::init();

    let start = Instant::now();
    let solver = Solver::new();
    let duration = start.elapsed();
    print!("Found {:?} states in {:?}", solver.graph.node_count(), duration);
}
