// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board;
mod graph;
mod solver;
mod views;

use crate::board::{BoardId, SlideMove};
use crate::solver::Solver;
use crate::views::StatefulViews;
use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum MoveEffectiveness {
    Positive,
    Neutral,
    Negative,
}

#[wasm_bindgen]
pub struct MoveInfo {
    pub slide_move: SlideMove,
    pub resulting_id: BoardId,
    pub resulting_distance: u32,
    pub effectiveness: MoveEffectiveness,
}

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
        on_highlight: Function,
    ) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        env_logger::init();

        let solver = Solver::new();

        Ok(Self {
            stateful_views: StatefulViews::new(
                solver.graph,
                meta_canvas_id,
                board_canvas_id,
                on_highlight,
            )?,
        })
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

    pub fn preview_move(&self, move_info: &MoveInfo) {
        self.stateful_views.borrow().preview_move(move_info);
    }

    pub fn cancel_preview(&self) {
        self.stateful_views.borrow().cancel_preview();
    }

    pub async fn do_move(&self, move_info: &MoveInfo) -> Option<Vec<MoveInfo>> {
        // TODO(Menno 11.08.2025) I'm getting really annoyed at all this refcell trickery and struggling with async.
        //  Dear future me, please review this code and clean it up?
        StatefulViews::do_move(&self.stateful_views, move_info).await
    }

    pub fn restart(&self) -> Option<Vec<MoveInfo>> {
        self.stateful_views.borrow().restart()
    }
}
