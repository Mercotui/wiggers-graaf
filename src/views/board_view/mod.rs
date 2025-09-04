// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT
mod controls;
mod layout;
mod renderer;
pub mod visual_board;

use crate::board::SlideMove;
use crate::graph;
use crate::views::board_view::controls::{ControlEvent, Controls};
use crate::views::board_view::layout::Layout;
use crate::views::board_view::renderer::Renderer;
use crate::views::board_view::visual_board::{
    AnimatableOffset, Animation, AnimationRepeatBehavior, DragEndResult, DragMove, VisualBoard,
};
use crate::views::frame_scheduler::FrameScheduler;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::{get_element_of_type, Size};
use futures::channel::oneshot;
use keyframe::{keyframes, AnimationSequence};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

pub type OnDragMoveCb = dyn FnMut(DragMove) -> graph::Node;

pub struct BoardView {
    on_drag_move_cb: Box<OnDragMoveCb>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    _pointer_controls: Rc<RefCell<Controls>>,
    visual_board: VisualBoard,
    layout: Layout,
    renderer: Renderer,
}
impl BoardView {
    pub fn new(
        canvas_id: &str,
        on_drag_move_cb: Box<OnDragMoveCb>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas: HtmlCanvasElement = get_element_of_type(canvas_id)?;
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<BoardView>>| {
            let self_ref_for_on_frame_cb = self_ref.clone();
            let self_ref_for_resize_observer_cb = self_ref.clone();
            let self_ref_for_mouse_event_cb = self_ref.clone();

            RefCell::new(Self {
                on_drag_move_cb,
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
                _pointer_controls: Controls::new(
                    &canvas,
                    Box::new(move |event: ControlEvent| {
                        self_ref_for_mouse_event_cb
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .handle_pointer_event(event)
                    }),
                )
                .expect("Could not create board MouseHandler"),
                visual_board: VisualBoard::empty(),
                layout: Layout::zero(),
                renderer: Renderer::new(canvas).expect("Could not initialize board renderer"),
            })
        }))
    }

    pub fn preview_move(&mut self, target_move: Option<SlideMove>) {
        let animation_done = match target_move {
            None => {
                self.visual_board.highlight(&None);
                self.visual_board.animate(None)
            }
            Some(slide_move) => {
                self.visual_board.highlight(&Some(slide_move.start));

                let from = AnimatableOffset::zero();
                let to = AnimatableOffset::from_distance_and_direction(
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
        let from = AnimatableOffset::zero();
        let to = AnimatableOffset::from_distance_and_direction(
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

    pub fn transition_to(&mut self, state: &graph::Node) {
        // TODO(Menno 30.06.2025) Animate this transition
        self.set_state(state);
    }

    fn handle_pointer_event(&mut self, event: ControlEvent) -> bool {
        let mut handled = false;
        // TODO(Menno 06.08.2025) Highlight pieces if we hover over them
        match event {
            ControlEvent::Down(coordinates) => {
                let coordinates = self.layout.apply_inverse_to_mouse(coordinates);
                handled = self.visual_board.start_drag(coordinates);
            }
            ControlEvent::Up() => {
                match self.visual_board.stop_drag() {
                    DragEndResult::Some(visual_move) => {
                        // TODO(Menno 16.08.2025) Animate this and the other views
                        let new_state = (self.on_drag_move_cb)(visual_move);
                        self.set_state(&new_state);
                        handled = true;
                    }
                    DragEndResult::None => {
                        handled = true;
                    }
                    DragEndResult::Invalid => {
                        // Ignore such an event
                    }
                }
            }
            ControlEvent::Move(coordinates) => {
                let coordinates = self.layout.apply_inverse_to_mouse(coordinates);
                handled = self.visual_board.drag(coordinates);
            }
        }
        if handled {
            self.frame_scheduler.schedule().unwrap();
        }
        handled
    }

    fn set_state(&mut self, state: &graph::Node) {
        self.visual_board = VisualBoard::new(state);
        self.layout = Layout::new(
            self.visual_board.size,
            self.layout.get_canvas_size(),
            web_sys::window().unwrap().device_pixel_ratio(),
        );
        self.frame_scheduler.schedule().unwrap();
    }

    /// Recalculate layout, application of the canvas size is deferred to the draw function, to avoid flashes.
    fn resize(&mut self, width: f64, height: f64) {
        let window = web_sys::window().unwrap();
        self.layout = Layout::new(
            self.visual_board.size,
            Size::new(width, height),
            window.device_pixel_ratio(),
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
