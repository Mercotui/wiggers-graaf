// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board::{BoardId, SlideMove};
use crate::graph::Graph;
use crate::views::utils::{get_document, get_element_of_type, get_window};
use crate::views::StatefulViews;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::{Rc, Weak};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlDivElement};

/**
 * Get the move indicator color for a given delta-distance
 * @param effectiveness How the move impacts resulting distance to the solution
 * @return The move-indicator color
 */
fn get_color(effectiveness: MoveEffectiveness) -> &'static str {
    match effectiveness {
        MoveEffectiveness::Positive => "#009d77",
        MoveEffectiveness::Neutral => "#4B7BFF",
        MoveEffectiveness::Negative => "#ff443a",
    }
}

fn collect_moves(graph: &Graph, state: BoardId) -> Vec<MoveInfo> {
    let state = graph
        .map
        .get(&state)
        .expect("Could not find state in graph");

    let current_distance = state
        .distance_to_solution
        .expect("Incomplete state, missing distance field");

    state
        .edges
        .iter()
        .filter_map(|edge| {
            let neighbor = graph.map.get(&edge.neighbor).expect("Invalid neighbor ID");
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

/// Create a div that acts as a button that executes the corresponding move
fn create_move_button(
    move_info: MoveInfo,
    document: &Document,
    parent_ref: Rc<RefCell<StatefulViews>>,
) -> Result<HtmlDivElement, JsValue> {
    let move_div: HtmlDivElement = document.create_element("div")?.dyn_into()?;
    let indicator_div: HtmlDivElement = document.create_element("div")?.dyn_into()?;
    let coordinates_span = document.create_element("span")?;
    let description_span = document.create_element("span")?;

    indicator_div.class_list().add_1("game-move-indicator")?;
    indicator_div
        .style()
        .set_property("background-color", get_color(move_info.effectiveness))?;

    coordinates_span.append_with_str_1(move_info.slide_move.to_string().as_str())?;
    coordinates_span.class_list().add_1("game-coordinates")?;

    description_span
        .class_list()
        .add_1("game-move-description")?;
    description_span.append_with_node_1(&coordinates_span)?;
    description_span
        .append_with_str_1(format!("{} steps left", move_info.resulting_distance).as_str())?;

    move_div.append_with_node_2(&indicator_div, &description_span)?;
    move_div.class_list().add_1("game-move")?;

    let move_info_copy = move_info;
    let move_div_clone = move_div.clone();
    let parent_ref_clone = parent_ref.clone();
    move_div.set_onclick(Some(
        Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |_event| {
            move_div_clone
                .class_list()
                .add_1("clicked")
                .expect("Failed to add clicked class to move div");

            // TODO(Menno 24.08.2025) I don't quite understand why this second clone is needed,
            //  but the compiler says so. And I'm too deep into this to question it. Please refactor.
            let parent_ref_clone = parent_ref_clone.clone();
            spawn_local(
                async move { StatefulViews::do_move(&parent_ref_clone, &move_info_copy).await },
            );
        })
        .into_js_value()
        .unchecked_ref(),
    ));

    let move_info_copy = move_info;
    let parent_ref_clone = parent_ref.clone();
    move_div.set_onmouseenter(Some(
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_event| {
            parent_ref_clone
                .borrow_mut()
                .preview_move(Some(move_info_copy));
        })
        .into_js_value()
        .unchecked_ref(),
    ));

    let parent_ref_clone = parent_ref.clone();
    move_div.set_onmouseleave(Some(
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_event| {
            parent_ref_clone.borrow_mut().preview_move(None);
        })
        .into_js_value()
        .unchecked_ref(),
    ));

    Ok(move_div)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MoveEffectiveness {
    Positive,
    Neutral,
    Negative,
}

#[derive(Clone, Copy)]
pub struct MoveInfo {
    pub slide_move: SlideMove,
    pub resulting_id: BoardId,
    pub resulting_distance: u32,
    pub effectiveness: MoveEffectiveness,
}

struct MoveButton {
    move_info: MoveInfo,
    div: HtmlDivElement,
}

pub struct MovesView {
    // TODO(Menno 17.08.2025) At this point I'm convinced my architecture sucks, and I know how to fix it,
    //  but that's gonna be a fair bit of work. Dear future me, please remove this ref.
    parent_ref: Weak<RefCell<StatefulViews>>,
    self_ref: Weak<RefCell<MovesView>>,
    best_move: Option<MoveButton>,
    auto_solve_enabled: bool,
    auto_solve_toggle_div: HtmlDivElement,
    auto_solve_timeout_id: Option<i32>,
    restart_button_div: HtmlDivElement,
    moves_div: HtmlDivElement,
}

impl MovesView {
    pub fn new(
        moves_div_id: &str,
        restart_div_id: &str,
        solve_div_id: &str,
        parent_ref: Weak<RefCell<StatefulViews>>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<Self>>| {
            let self_ref_clone = self_ref.clone();
            let restart_div = get_element_of_type::<HtmlDivElement>(restart_div_id)
                .expect("Failed to find restart button div");
            restart_div.set_onclick(Some(
                Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |_event| {
                    MovesView::restart(&self_ref_clone.upgrade().unwrap());
                })
                .into_js_value()
                .unchecked_ref(),
            ));

            let self_ref_clone = self_ref.clone();
            let solve_div = get_element_of_type::<HtmlDivElement>(solve_div_id)
                .expect("Failed to find auto solve button div");
            solve_div.set_onclick(Some(
                Closure::<dyn FnMut(web_sys::PointerEvent)>::new(move |_event| {
                    self_ref_clone
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .toggle_auto_solve();
                })
                .into_js_value()
                .unchecked_ref(),
            ));

            RefCell::new(Self {
                parent_ref,
                self_ref: self_ref.clone(),
                best_move: None,
                auto_solve_enabled: false,
                auto_solve_toggle_div: solve_div,
                auto_solve_timeout_id: None,
                restart_button_div: restart_div,
                moves_div: get_element_of_type(moves_div_id).expect("Failed to find moves div"),
            })
        }))
    }

    pub fn set_data(&mut self, graph: &Graph, active_state: BoardId) {
        // Clear the contents
        self.moves_div.set_inner_html("");
        self.best_move = None;

        let moves = collect_moves(graph, active_state);

        let document = get_document().expect("Failed to get document");
        for move_info in moves {
            let parent_ref = self.parent_ref.upgrade().unwrap().clone();
            let move_button = create_move_button(move_info, &document, parent_ref)
                .expect("Failed to create move button");
            self.moves_div
                .append_child(&move_button)
                .expect("Failed to append move button to list");

            // Store the first move, we consider this to be the best move
            if self.best_move.is_none() {
                self.best_move = Some(MoveButton {
                    move_info,
                    div: move_button,
                });
            }
        }

        // If we are in auto-solve mode, we soon start the next move.
        if self.auto_solve_enabled {
            if self.best_move.is_some()
                && self.best_move.as_ref().unwrap().move_info.effectiveness
                    == MoveEffectiveness::Positive
            {
                // Keep going until we run out of good moves to make
                self.queue_best_move().expect("Failed to queue best move");
            } else {
                self.set_auto_solve(false)
                    .expect("Failed to disable auto_solve");
            }
        }
    }

    fn restart(self_ref: &Rc<RefCell<Self>>) {
        let parent_ref = {
            let mut self_deref = self_ref.borrow_mut();
            self_deref
                .set_auto_solve(false)
                .expect("Failed to disable auto_solve");
            // Highlight the restart button
            self_deref
                .restart_button_div
                .class_list()
                .add_1("clicked")
                .expect("Couldn't add clicked class to restart button");
            // Remove highlight from button after 200 ms
            let self_ref_clone = self_ref.clone();
            get_window()
                .expect("Failed to access DOM")
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    Closure::<dyn FnMut()>::new(move || {
                        self_ref_clone
                            .borrow()
                            .restart_button_div
                            .class_list()
                            .remove_1("clicked")
                            .expect("Failed to remove clicked class from restart button");
                    })
                    .into_js_value()
                    .unchecked_ref(),
                    200,
                )
                .expect_throw("Failed to remove clicked class from restart button");

            self_deref.parent_ref.upgrade().unwrap()
        };
        StatefulViews::restart(&parent_ref);
    }

    fn toggle_auto_solve(&mut self) {
        self.set_auto_solve(!self.auto_solve_enabled)
            .expect("Failed to toggle auto-solve");
    }

    fn set_auto_solve(&mut self, enable: bool) -> Result<(), JsValue> {
        if self.auto_solve_enabled == enable {
            // do nothing
            return Ok(());
        }

        self.auto_solve_enabled = enable;

        if self.auto_solve_enabled {
            self.auto_solve_toggle_div.class_list().add_1("clicked")?;
            // Start chain of moves
            self.queue_best_move()?;
        } else {
            self.auto_solve_toggle_div
                .class_list()
                .remove_1("clicked")?;
            self.cancel_best_move()?;
        }
        Ok(())
    }

    fn queue_best_move(&mut self) -> Result<(), JsValue> {
        if let Some(best_move) = &self.best_move {
            best_move.div.class_list().add_1("highlight")?;
            let self_ref_clone = self.self_ref.upgrade().unwrap().clone();
            let id = get_window()?.set_timeout_with_callback_and_timeout_and_arguments_0(
                Closure::<dyn FnMut()>::new(move || {
                    let self_ref_clone = self_ref_clone.clone();
                    spawn_local(async move { MovesView::do_best_move(&self_ref_clone).await });
                })
                .into_js_value()
                .unchecked_ref(),
                200,
            )?;
            self.auto_solve_timeout_id = Some(id);
        }
        Ok(())
    }

    fn cancel_best_move(&mut self) -> Result<(), JsValue> {
        if let Some(timeout_id) = self.auto_solve_timeout_id.take() {
            get_window()?.clear_timeout_with_handle(timeout_id);
        }
        if let Some(best_move) = &self.best_move {
            best_move.div.class_list().remove_1("highlight")?
        }
        Ok(())
    }

    async fn do_best_move(self_ref: &Rc<RefCell<Self>>) {
        let (parent_ref, move_info) = {
            let self_ref = self_ref.borrow();
            let Some(best_move) = &self_ref.best_move else {
                // Nothing to do
                return;
            };
            best_move
                .div
                .class_list()
                .add_1("clicked")
                .expect("Couldn't add clicked class to move");
            (self_ref.parent_ref.upgrade().unwrap(), best_move.move_info)
        };

        StatefulViews::do_move(&parent_ref, &move_info).await;
    }
}
