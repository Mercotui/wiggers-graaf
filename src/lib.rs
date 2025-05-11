// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board;
mod graph;
mod graph_view;
mod solver;

use crate::board::BoardId;
use crate::graph::Node;
use crate::graph_view::GraphView;
use crate::solver::Solver;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WiggersGraaf {
    solver: Solver,
    graph_view: GraphView,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        env_logger::init();
        Ok(Self {
            solver: Solver::new(),
            graph_view: GraphView::new(canvas_id)?,
        })
    }
    pub fn draw(&mut self) {
        self.graph_view.draw();
    }

    pub fn resize_meta_canvas(&mut self) {
        self.graph_view.resize();
    }

    pub fn accumulate_translation(&mut self, delta_x: f32, delta_y: f32) {
        self.graph_view.accumulate_translation(delta_x, delta_y);
    }

    pub fn accumulate_zoom(&mut self, zoom_movement: f32, target_x: f32, target_y: f32) {
        self.graph_view
            .accumulate_zoom(zoom_movement, target_x, target_y);
    }

    pub fn get_start_id() -> BoardId {
        board::to_id(&board::get_start_board())
    }

    pub fn get_state(&mut self, id: BoardId) -> Node {
        self.solver.graph.map.get(&id).expect("Invalid ID").clone()
    }

    pub fn set_active_state(&mut self, active_state: BoardId) {
        self.graph_view.set_data(&self.solver.graph, active_state);
    }
}
