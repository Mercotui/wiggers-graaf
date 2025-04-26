mod arrangement;
mod board;
mod graph;
mod renderer;
mod solver;
mod utils;

use crate::arrangement::Arrangement;
use crate::board::BoardId;
use crate::graph::Node;
use crate::renderer::Renderer;
use crate::solver::Solver;
use crate::utils::set_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WiggersGraaf {
    solver: Solver,
    renderer: Renderer,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        set_panic_hook();
        env_logger::init();
        Ok(Self {
            solver: Solver::new(),
            renderer: Renderer::new(canvas_id)?,
        })
    }
    pub fn draw(&mut self, active_state: BoardId) {
        set_panic_hook();

        self.renderer
            .set_data(&Arrangement::new(&self.solver.graph, active_state));
        self.renderer.draw();
    }

    pub fn resize_meta_canvas(&mut self) {
        self.renderer.resize();
    }

    pub fn get_start_id() -> BoardId {
        board::to_id(&board::get_start_board())
    }

    pub fn get_state(&mut self, id: BoardId) -> Node {
        self.solver.graph.map.get(&id).expect("Invalid ID").clone()
    }
}
