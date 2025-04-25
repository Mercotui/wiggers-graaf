mod arrangement;
mod board;
mod graph;
mod renderer;
mod solver;
mod utils;

use crate::arrangement::Arrangement;
use crate::board::BoardId;
use crate::graph::Node;
use crate::solver::Solver;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

#[wasm_bindgen]
struct WiggersGraaf {
    solver: Solver,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        set_panic_hook();
        env_logger::init();
        Self {
            solver: Solver::new(),
        }
    }
    pub fn draw(
        &mut self,
        canvas_id: &str,
        active_state: BoardId,
    ) -> Result<WebGl2RenderingContext, JsValue> {
        set_panic_hook();

        let arrangement = Arrangement::new(&self.solver.graph, active_state);
        renderer::draw(canvas_id, &arrangement)
    }

    pub fn get_start_id() -> BoardId {
        board::to_id(&board::get_start_board())
    }

    pub fn get_state(&mut self, id: BoardId) -> Node {
        self.solver.graph.map.get(&id).expect("Invalid ID").clone()
    }
}
