// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use crate::board::Board;
use crate::frame_scheduler::FrameScheduler;
use crate::utils::get_canvas;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn create_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    Ok(canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?)
}

pub struct BoardView {
    self_ref: Option<Rc<RefCell<Self>>>,
    frame_scheduler: FrameScheduler,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}
impl BoardView {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas = get_canvas(canvas_id)?;
        let ctx = create_context_2d(&canvas)?;
        let board_view = Rc::new(RefCell::new(BoardView {
            self_ref: None,
            frame_scheduler: FrameScheduler::noop(),
            canvas,
            ctx,
        }));

        board_view.borrow_mut().self_ref = Some(board_view.clone());
        let on_animation_frame = Closure::wrap(Box::new(move |timestamp: f64| {
            self_rc.borrow_mut().draw(timestamp);
        }));


        Ok(board_view)
    }

    pub fn set_board(&mut self, board: &Board) {}

    fn draw(&mut self, timestamp: Duration) {

    }
}
