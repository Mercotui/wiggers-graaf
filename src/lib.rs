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
    pub fn draw(&mut self) {
        self.renderer.draw();
    }

    pub fn resize_meta_canvas(&mut self) {
        self.renderer.resize();
    }

    pub fn accumulate_translation(&mut self, delta_x: f32, delta_y: f32) {
        self.renderer.accumulate_translation(delta_x, delta_y);
    }

    pub fn accumulate_scale(&mut self, delta_scale: f32) {
        self.renderer.accumulate_scale(delta_scale);
    }

    pub fn get_start_id() -> BoardId {
        board::to_id(&board::get_start_board())
    }

    pub fn get_state(&mut self, id: BoardId) -> Node {
        self.solver.graph.map.get(&id).expect("Invalid ID").clone()
    }

    pub fn set_active_state(&mut self, active_state: BoardId) {
        self.renderer
            .set_data(&Arrangement::new(&self.solver.graph, active_state));
    }
}
