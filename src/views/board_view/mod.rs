// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod layout;
mod visual_board;

use crate::board::{Board, SlideMove};
use crate::views::board_view::layout::Layout;
use crate::views::board_view::visual_board::{
    AnimatableCoordinates, Animation, AnimationRepeatBehavior, VisualBoard, VisualPiece, VisualSize,
};
use crate::views::frame_scheduler::FrameScheduler;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::get_canvas;
use futures::channel::oneshot;
use keyframe::{keyframes, AnimationSequence};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::console::error_1;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn create_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    Ok(canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?)
}

pub struct BoardView {
    _self_ref: Weak<RefCell<Self>>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    visual_board: VisualBoard,
    layout: Layout,
    canvas: HtmlCanvasElement,
    canvas_size: layout::Size,
    ctx: CanvasRenderingContext2d,
}
impl BoardView {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas = get_canvas(canvas_id)?;
        let ctx = create_context_2d(&canvas)?;
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<BoardView>>| {
            let self_ref_for_on_frame_cb = self_ref.clone();
            let self_ref_for_resize_observer_cb = self_ref.clone();

            let refcell_self = RefCell::new(Self {
                _self_ref: self_ref.clone(),
                frame_scheduler: FrameScheduler::new(Box::new(move |timestamp: Duration| {
                    self_ref_for_on_frame_cb
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .draw(timestamp);
                })),
                _resize_observer: ResizeObserver::new(
                    &canvas,
                    Box::new(move |width, height| {
                        self_ref_for_resize_observer_cb
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .resize(width, height);
                    }),
                ),
                visual_board: VisualBoard::empty(),
                layout: Layout::new(VisualSize::zero(), layout::Size::zero(), 0.0),
                canvas,
                canvas_size: layout::Size::zero(),
                ctx,
            });
            refcell_self
        }))
    }

    pub fn preview_move(&mut self, target_move: Option<&SlideMove>) {
        match target_move {
            None => self.visual_board.animate(None),
            Some(slide_move) => {
                let from = AnimatableCoordinates::zero();
                let to = AnimatableCoordinates::from_distance_and_direction(
                    slide_move.distance as f64,
                    slide_move.direction,
                );
                self.visual_board.animate(Some(Animation {
                    sequence: keyframes![
                        (from, 0.0),
                        (from, 1.0, keyframe::functions::EaseInOutCubic),
                        (to, 1.15),
                        (to, 2.15, keyframe::functions::EaseInOutCubic),
                        (from, 2.3)
                    ],
                    target: slide_move.start,
                    repeat: AnimationRepeatBehavior::Loop,
                    on_done_cb: None,
                }));
            }
        };

        self.frame_scheduler
            .schedule()
            .expect("Couldn't schedule frame");
    }

    pub fn do_move(&mut self, slide_move: &SlideMove, on_done_cb: Box<dyn FnOnce()>) {
        let from = AnimatableCoordinates::zero();
        let to = AnimatableCoordinates::from_distance_and_direction(
            slide_move.distance as f64,
            slide_move.direction,
        );

        self.visual_board.animate(Some(Animation {
            sequence: keyframes![(from, 0.0, keyframe::functions::EaseInOutCubic), (to, 0.15)],
            target: slide_move.start,
            repeat: AnimationRepeatBehavior::None,
            on_done_cb: Some(on_done_cb),
        }));

        self.frame_scheduler.schedule().unwrap();
    }

    pub fn transition_to(&mut self, board: &Board) {
        // TODO(Menno 30.06.2025) Animate this transition
        self.set_board(board);
    }

    fn set_board(&mut self, board: &Board) {
        self.visual_board = VisualBoard::new(board);
        self.frame_scheduler.schedule().unwrap();
    }

    /// Store the new size, application of the size is deferred to the draw function, to avoid flashes.
    fn resize(&mut self, width: f64, height: f64) {
        self.layout.invalidate();
        self.canvas_size = layout::Size::new(width, height);
        self.frame_scheduler.schedule().unwrap();
    }

    fn draw(&mut self, timestamp: Duration) {
        let request_new_frame = self.visual_board.update_to(timestamp).is_ok();

        if self.layout.is_valid() {
            // Clear the canvas
            self.ctx.clear_rect(
                0.0,
                0.0,
                self.canvas.width() as f64,
                self.canvas.height() as f64,
            );
        } else {
            // Resize the canvas, which also clears it
            self.canvas.set_width(self.canvas_size.width as u32);
            self.canvas.set_height(self.canvas_size.height as u32);
            self.layout = Layout::new(
                self.visual_board.size,
                layout::Size::new(self.canvas_size.width, self.canvas_size.height),
                web_sys::window().unwrap().device_pixel_ratio(),
            );
        }

        // Draw the game pieces
        self.visual_board
            .pieces
            .iter()
            .for_each(|(_, piece)| self.draw_piece(piece));

        // Draw the axes
        // TODO(Menno 22.06.2025) Draw the axes
        if request_new_frame {
            self.frame_scheduler.schedule().unwrap();
        }
    }

    /**
     * Draw a single game piece to the canvas
     * @param piece the piece to draw
     */
    fn draw_piece(&self, piece: &VisualPiece) {
        self.ctx.begin_path();

        let opacity: f64 = if piece.highlighted { 1.0 } else { 0.8 };
        self.ctx
            .set_fill_style_str(format!("rgba({},{})", piece.color, opacity).as_str());

        let (pos, size, corner_radius) = self.layout.apply_to_piece(piece);
        self.ctx
            .round_rect_with_f64(pos.x, pos.y, size.width, size.height, corner_radius)
            .expect("Failed to draw piece");
        self.ctx.fill();
    }
}
