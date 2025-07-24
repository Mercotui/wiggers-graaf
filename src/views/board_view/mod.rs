// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod layout;
mod renderer;
mod visual_board;

use crate::board::{Board, SlideMove};
use crate::views::board_view::layout::Layout;
use crate::views::board_view::renderer::Renderer;
use crate::views::board_view::visual_board::{
    AnimatableCoordinates, Animation, AnimationRepeatBehavior, VisualBoard,
};
use crate::views::frame_scheduler::FrameScheduler;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::get_canvas;
use futures::channel::oneshot;
use keyframe::{keyframes, AnimationSequence};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::JsValue;

pub struct BoardView {
    _self_ref: Weak<RefCell<Self>>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    visual_board: VisualBoard,
    layout: Layout,
    renderer: Renderer,
}
impl BoardView {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas = get_canvas(canvas_id)?;
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
                layout: Layout::zero(),
                renderer: Renderer::new(canvas).expect("Could not initialize board renderer"),
            });
            refcell_self
        }))
    }

    pub fn preview_move(&mut self, target_move: Option<&SlideMove>) {
        let animation_done = match target_move {
            None => {
                self.visual_board.highlight(None);
                self.visual_board.animate(None)
            }
            Some(slide_move) => {
                self.visual_board.highlight(Some(&slide_move.start));

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
                }))
            }
        };
        // Ignore the future, this animation never finishes anyway.
        drop(animation_done);

        self.frame_scheduler
            .schedule()
            .expect("Couldn't schedule frame");
    }

    pub fn do_move(&mut self, slide_move: &SlideMove) -> oneshot::Receiver<()> {
        let from = AnimatableCoordinates::zero();
        let to = AnimatableCoordinates::from_distance_and_direction(
            slide_move.distance as f64,
            slide_move.direction,
        );

        let animation_done = self.visual_board.animate(Some(Animation {
            sequence: keyframes![(from, 0.0, keyframe::functions::EaseInOutCubic), (to, 0.15)],
            target: slide_move.start,
            repeat: AnimationRepeatBehavior::None,
        }));

        self.frame_scheduler.schedule().unwrap();

        animation_done
    }

    pub fn transition_to(&mut self, board: &Board) {
        // TODO(Menno 30.06.2025) Animate this transition
        self.set_board(board);
    }

    fn set_board(&mut self, board: &Board) {
        self.visual_board = VisualBoard::new(board);
        self.layout = Layout::new(
            self.visual_board.size,
            self.layout.get_canvas_size(),
            web_sys::window().unwrap().device_pixel_ratio(),
        );
        self.frame_scheduler.schedule().unwrap();
    }

    /// Recalculate layout, application of the canvas size is deferred to the draw function, to avoid flashes.
    fn resize(&mut self, width: f64, height: f64) {
        self.layout = Layout::new(
            self.visual_board.size,
            layout::Size::new(width, height),
            web_sys::window().unwrap().device_pixel_ratio(),
        );
        self.frame_scheduler.schedule().unwrap();
    }

    fn draw(&mut self, timestamp: Duration) {
        // Update board and draw it
        let request_new_frame = self.visual_board.update_to(timestamp).is_ok();
        self.renderer.draw(&self.visual_board, &self.layout);

        // Schedule next frame if board is still animating
        if request_new_frame {
            self.frame_scheduler.schedule().unwrap();
        }
    }
}
