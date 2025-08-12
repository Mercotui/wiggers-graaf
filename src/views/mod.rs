// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT
mod board_view;
mod frame_scheduler;
pub mod graph_view;
mod mouse_handler;
mod resize_observer;
mod utils;

use crate::board::BoardId;
use crate::graph::Graph;
use crate::{board, graph, MoveEffectiveness, MoveInfo};
pub(crate) use board_view::BoardView;
pub(crate) use graph_view::GraphView;
use itertools::Itertools;
use js_sys::Function;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::{Rc, Weak};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use wasm_bindgen::JsValue;

/**
 * TODO(Menno 12.08.2025) I'm adding this just to be done with async borrow checking stuff,
 *  please revisit this if I ever rewrite this app to be less... naive C++ brained.
 *
 * A RAII guard to "lock" a bool, a poor man's work around to not have to use a mutex.
 * Using a mutex would cause borrowing across an await, and by using a bool I have the freedom to
 * drop the borrow after setting it to "locked".
 */
struct BoolGuard<'a> {
    target: &'a AtomicBool,
}
impl<'a> BoolGuard<'a> {
    fn lock(target: &'a AtomicBool) -> Option<Self> {
        if target.load(Relaxed) {
            // Already "locked"
            return None;
        }

        // Lock
        target.store(true, Relaxed);
        Some(Self { target })
    }
}

impl<'a> Drop for BoolGuard<'a> {
    fn drop(&mut self) {
        // Unlock
        self.target.store(false, Relaxed);
    }
}

pub struct StatefulViews {
    graph: Graph,
    graph_view: Rc<RefCell<GraphView>>,
    board_view: Rc<RefCell<BoardView>>,
    _on_highlight: Function,
    move_lock: AtomicBool,
}

impl StatefulViews {
    pub fn new(
        graph: Graph,
        meta_canvas_id: &str,
        board_canvas_id: &str,
        on_highlight: Function,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        Ok(Rc::new_cyclic(|_self_ref: &Weak<RefCell<Self>>| {
            RefCell::new(Self {
                graph,
                graph_view: GraphView::new(meta_canvas_id).expect("Couldn't create GraphView"),
                board_view: BoardView::new(board_canvas_id).expect("Couldn't create BoardView"),
                move_lock: AtomicBool::new(false),
                _on_highlight: on_highlight,
            })
        }))
    }

    pub fn accumulate_translation(&self, delta_x: f32, delta_y: f32) {
        self.graph_view
            .borrow_mut()
            .accumulate_translation(delta_x, delta_y);
    }

    pub fn accumulate_zoom(&self, zoom_movement: f32, target_x: f32, target_y: f32) {
        self.graph_view
            .borrow_mut()
            .accumulate_zoom(zoom_movement, target_x, target_y);
    }

    pub fn preview_move(&self, move_info: &MoveInfo) {
        let Some(_lock) = BoolGuard::lock(&self.move_lock) else {
            // No preview to cancel, a move is ongoing
            return;
        };
        self.board_view
            .borrow_mut()
            .preview_move(Some(&move_info.slide_move.clone()));
    }

    pub fn cancel_preview(&self) {
        let Some(_lock) = BoolGuard::lock(&self.move_lock) else {
            // No preview to cancel, a move is ongoing
            return;
        };
        self.board_view.borrow_mut().preview_move(None);
    }

    pub async fn do_move(
        self_ref: &Rc<RefCell<Self>>,
        move_info: &MoveInfo,
    ) -> Option<Vec<MoveInfo>> {
        // Set lock to true, and check if it was already set to true
        if self_ref.borrow().move_lock.swap(true, Relaxed) {
            // Ignore further moves until previous move has finished
            return None;
        };

        let move_done = self_ref
            .borrow()
            .board_view
            .borrow_mut()
            .do_move(&move_info.slide_move);
        move_done.await.expect("Unable to finish move");

        let result = Some(self_ref.borrow().set_state(move_info.resulting_id));
        self_ref.borrow().move_lock.store(false, Relaxed);
        result
    }

    pub fn restart(&self) -> Option<Vec<MoveInfo>> {
        let Some(_lock) = BoolGuard::lock(&self.move_lock) else {
            // No preview to cancel, a move is ongoing
            return None;
        };
        Some(self.set_state(board::to_id(&board::get_start_board())))
    }

    fn set_state(&self, new_state: BoardId) -> Vec<MoveInfo> {
        self.graph_view
            .borrow_mut()
            .set_data(&self.graph, new_state);

        let node = self.graph.map.get(&new_state).expect("Invalid ID");
        self.board_view.borrow_mut().transition_to(node);
        self.collect_moves(node)
    }

    fn collect_moves(&self, state: &graph::Node) -> Vec<MoveInfo> {
        let current_distance = state
            .distance_to_solution
            .expect("Incomplete state, missing distance field");

        state
            .edges
            .iter()
            .filter_map(|edge| {
                let neighbor = self
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
            .collect()
    }
}
