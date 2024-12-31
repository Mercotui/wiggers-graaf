mod arrangement;
mod board;
mod graph;
mod renderer;
mod solver;
mod utils;

use crate::arrangement::Arrangement;
use crate::graph::Node;
use crate::solver::Solver;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext;
use crate::board::BoardId;

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
pub fn draw(canvas_id: &str, active_state: BoardId, solver: &Solver) -> Result<WebGlRenderingContext, JsValue> {
    set_panic_hook();

    let arrangement = Arrangement::new(&solver.graph, active_state);
    renderer::draw(canvas_id, &arrangement)
}

#[wasm_bindgen]
pub fn get_start_id() -> BoardId {
    board::to_id(&board::get_start_board())
}

#[wasm_bindgen]
pub fn get_state(solver: &Solver, id: BoardId) -> Node {
    solver.graph.map.get(&id).expect("Invalid ID").clone()
}
