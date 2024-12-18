mod arrangement;
mod board;
mod graph;
mod renderer;
mod solver;
mod utils;

use crate::arrangement::Arrangement;
use crate::graph::{Graph, Node};
use crate::solver::Solver;
use crate::utils::set_panic_hook;
use std::hash::Hash;
use std::time::Instant;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;

#[wasm_bindgen]
pub fn generate() -> Solver {
    env_logger::init();

    // let start = Instant::now();
    let solver = Solver::new();
    // let duration = start.elapsed();
    // print!(
    //     "Found {:?} states in {:?}",
    //     solver.graph.node_count(),
    //     duration
    // );

    solver
}

#[wasm_bindgen]
pub fn draw(canvas_id: &str, solver: &Solver) -> Result<WebGlRenderingContext, JsValue> {
    set_panic_hook();

    let arrangement = Arrangement::new(&solver.graph);
    renderer::draw(canvas_id, &arrangement)
}

#[wasm_bindgen]
pub fn get_start_id() -> u64 {
    board::to_hash(&board::get_start_board())
}

#[wasm_bindgen]
pub fn get_state(solver: &Solver, id: u64) -> Node {
    solver.graph.map.get(&id).expect("Invalid ID").clone()
}
