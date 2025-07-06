// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board;
mod graph;
mod solver;
mod views;

use crate::board::{BoardId, SlideMove};
use crate::solver::Solver;
use itertools::Itertools;
use js_sys::Function;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use views::{BoardView, GraphView};
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
    solver: Solver,
    graph_view: Rc<RefCell<GraphView>>,
    board_view: Rc<RefCell<BoardView>>,
    on_state_changed: Function,
    on_highlight: Function,
    move_pending: bool,
}

#[wasm_bindgen]
impl WiggersGraaf {
    #[wasm_bindgen(constructor)]
    pub fn new(
        meta_canvas_id: &str,
        board_canvas_id: &str,
        on_state_changed: Function,
        on_highlight: Function,
    ) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        env_logger::init();

        let mut instance = Self {
            solver: Solver::new(),
            graph_view: GraphView::new(meta_canvas_id)?,
            board_view: BoardView::new(board_canvas_id)?,
            on_state_changed,
            on_highlight,
            move_pending: false,
        };
        instance.restart();

        Ok(instance)
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

    pub fn preview_move(&mut self, move_info: &MoveInfo) {
        self.board_view
            .borrow_mut()
            .preview_move(Some(&move_info.slide_move));
    }

    pub fn do_move(&mut self, move_info: &MoveInfo) {
        self.board_view.borrow_mut().do_move(
            &move_info.slide_move,
            Box::new(move || {
                // TODO(Menno 29.06.2025) set state after animation completes
                // self.set_state(move_info.resulting_id);
            }),
        );
    }

    pub fn cancel_preview(&mut self) {
        self.board_view.borrow_mut().preview_move(None);
    }

    pub fn restart(&mut self) {
        self.set_state(board::to_id(&board::get_start_board()));
    }

    fn set_state(&self, new_state: BoardId) {
        self.graph_view
            .borrow_mut()
            .set_data(&self.solver.graph, new_state);

        let node = self.solver.graph.map.get(&new_state).expect("Invalid ID");
        self.board_view.borrow_mut().transition_to(&node.board);
        self.emit_moves(node);
    }

    fn emit_moves(&self, state: &graph::Node) {
        let current_distance = state
            .distance_to_solution
            .expect("Incomplete state, missing distance field");

        let moves_list: Vec<MoveInfo> = state
            .edges
            .iter()
            .filter_map(|edge| {
                let neighbor = self
                    .solver
                    .graph
                    .map
                    .get(&edge.neighbor)
                    .expect("Invalid neighbor ID");
                let resulting_distance = neighbor
                    .distance_to_solution
                    .expect("Incomplete neighbour, missing distance field");
                let effectiveness = match resulting_distance.cmp(&current_distance) {
                    Ordering::Less => MoveEffectiveness::Positive,
                    Ordering::Equal => MoveEffectiveness::Neutral,
                    Ordering::Greater => MoveEffectiveness::Negative,
                };

                // Hide our "fake" solution moves
                // TODO(Menno 28.06.2025) We could get rid of these fake moves by altering the solver
                if resulting_distance == 0 {
                    return None;
                }

                Some(MoveInfo {
                    slide_move: edge.slide_move,
                    resulting_id: edge.neighbor,
                    resulting_distance,
                    effectiveness,
                })
            })
            .sorted_by(|a, b| a.resulting_distance.cmp(&b.resulting_distance))
            .collect();
        // Emit the list of possible moves to JS
        self.on_state_changed
            .call1(&JsValue::NULL, &moves_list.into())
            .unwrap();
    }
}
