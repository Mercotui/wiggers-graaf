// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board;
mod graph;
mod solver;
mod views;

use crate::solver::Solver;
use crate::views::StatefulViews;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WiggersGraaf {
    stateful_views: Rc<RefCell<StatefulViews>>,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new(
        meta_canvas_id: &str,
        board_canvas_id: &str,
        moves_div_id: &str,
        restart_div_id: &str,
        solve_div_id: &str,
    ) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        env_logger::init();

        let solver = Solver::new();

        let instance = Self {
            stateful_views: StatefulViews::new(
                solver.graph,
                meta_canvas_id,
                board_canvas_id,
                moves_div_id,
                restart_div_id,
                solve_div_id,
            )?,
        };
        StatefulViews::restart(&instance.stateful_views);
        Ok(instance)
    }

    pub fn accumulate_translation(&self, delta_x: f32, delta_y: f32) {
        self.stateful_views
            .borrow()
            .accumulate_translation(delta_x, delta_y);
    }

    pub fn accumulate_zoom(&self, zoom_movement: f32, target_x: f32, target_y: f32) {
        self.stateful_views
            .borrow()
            .accumulate_zoom(zoom_movement, target_x, target_y);
    }
}
