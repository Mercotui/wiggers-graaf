// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod board_view;
mod frame_scheduler;
pub mod graph_view;
mod mouse_handler;
mod moves_view;
mod resize_observer;
mod utils;

use crate::board::BoardId;
use crate::graph::Graph;
use crate::views::board_view::visual_board::DragMove;
use crate::views::moves_view::{MoveInfo, MovesView};
use crate::{board, graph};
pub(crate) use board_view::BoardView;
pub(crate) use graph_view::GraphView;
use std::cell::RefCell;
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
    moves_view: Rc<RefCell<MovesView>>,
    move_lock: AtomicBool,
}

impl StatefulViews {
    pub fn new(
        graph: Graph,
        meta_canvas_id: &str,
        board_canvas_id: &str,
        moves_div_id: &str,
        restart_div_id: &str,
        solve_div_id: &str,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<Self>>| {
            let self_ref_clone_for_board_view = self_ref.clone();
            let self_ref_clone_for_moves_view = self_ref.clone();
            RefCell::new(Self {
                graph,
                graph_view: GraphView::new(meta_canvas_id).expect("Couldn't create GraphView"),
                board_view: BoardView::new(
                    board_canvas_id,
                    Box::new(move |drag_move| {
                        self_ref_clone_for_board_view
                            .upgrade()
                            .expect("Could not reference StatefulViews")
                            .borrow_mut()
                            .do_drag_move(&drag_move)
                    }),
                )
                .expect("Couldn't create BoardView"),
                moves_view: MovesView::new(
                    moves_div_id,
                    restart_div_id,
                    solve_div_id,
                    self_ref_clone_for_moves_view,
                )
                .expect("Couldn't create MovesView"),
                move_lock: AtomicBool::new(false),
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

    pub fn preview_move(&self, move_info: Option<MoveInfo>) {
        let Some(_lock) = BoolGuard::lock(&self.move_lock) else {
            // No preview, a move is ongoing
            return;
        };
        self.board_view
            .borrow_mut()
            .preview_move(move_info.map(|move_info| move_info.slide_move));
    }

    fn do_drag_move(&self, drag_move: &DragMove) -> graph::Node {
        let new_state = drag_move.resulting_id;

        // TODO(Menno 16.08.2025) This duplicates code from set_state
        self.graph_view
            .borrow_mut()
            .set_data(&self.graph, new_state);
        self.moves_view
            .borrow_mut()
            .set_data(&self.graph, new_state);

        // Return the new node to the BoardView
        self.graph.map.get(&new_state).expect("Invalid ID").clone()
    }

    pub async fn do_move(self_ref: &Rc<RefCell<Self>>, move_info: &MoveInfo) {
        // Set lock to true, and check if it was already set to true
        if self_ref.borrow().move_lock.swap(true, Relaxed) {
            // Ignore further moves until previous move has finished
            return;
        };

        let move_done = self_ref
            .borrow()
            .board_view
            .borrow_mut()
            .do_move(&move_info.slide_move);
        move_done.await.expect("Unable to finish move");

        // Steps to take after move finished
        let self_ref = self_ref.borrow();
        self_ref.set_state(move_info.resulting_id);
        self_ref.move_lock.store(false, Relaxed);
    }

    pub fn restart(self_ref: &Rc<RefCell<Self>>) {
        let self_ref = self_ref.borrow();
        // TODO(Menno 24.08.2025) Restart should cancel ongoing moves
        let Some(_lock) = BoolGuard::lock(&self_ref.move_lock) else {
            // Refuse to restart, a move is ongoing
            return;
        };
        self_ref.set_state(board::to_id(&board::get_start_board()));
    }

    fn set_state(&self, new_state: BoardId) {
        self.graph_view
            .borrow_mut()
            .set_data(&self.graph, new_state);
        self.moves_view
            .borrow_mut()
            .set_data(&self.graph, new_state);

        let node = self.graph.map.get(&new_state).expect("Invalid ID");
        self.board_view.borrow_mut().transition_to(node);
    }
}
