// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board;
mod frame_scheduler;
mod graph;
mod graph_view;
mod solver;
mod utils;

use crate::board::BoardId;
use crate::graph::Node;
use crate::graph_view::GraphView;
use crate::solver::Solver;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WiggersGraaf {
    solver: Solver,
    graph_view: Rc<RefCell<GraphView>>,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new(meta_canvas_id: &str) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        env_logger::init();
        Ok(Self {
            solver: Solver::new(),
            graph_view: GraphView::new(meta_canvas_id)?,
        })
    }
    pub fn draw(&mut self) -> Result<(), JsValue> {
        self.graph_view.borrow_mut().schedule_draw()
    }

    pub fn resize_meta_canvas(&mut self) {
        self.graph_view.borrow_mut().resize();
    }

    pub fn accumulate_translation(&mut self, delta_x: f32, delta_y: f32) {
        self.graph_view
            .borrow_mut()
            .accumulate_translation(delta_x, delta_y);
    }

    pub fn accumulate_zoom(&mut self, zoom_movement: f32, target_x: f32, target_y: f32) {
        self.graph_view
            .borrow_mut()
            .accumulate_zoom(zoom_movement, target_x, target_y);
    }

    pub fn get_start_id() -> BoardId {
        board::to_id(&board::get_start_board())
    }

    pub fn get_state(&mut self, id: BoardId) -> Node {
        self.solver.graph.map.get(&id).expect("Invalid ID").clone()
    }

    pub fn set_active_state(&mut self, active_state: BoardId) {
        self.graph_view
            .borrow_mut()
            .set_data(&self.solver.graph, active_state);
    }
}
