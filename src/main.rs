mod board;
mod solver;

use crate::solver::Solver;

fn main() {
    env_logger::init();
    let solver = Solver::new();
    // solver::export_vis_dot();
}
