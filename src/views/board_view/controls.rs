// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::pointer_handler::{MouseHandler, PointerEvent};
use crate::views::utils::Coordinates;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

pub struct Controls {
    on_event_cb: Box<OnPointerEventCb>,
    drag_pointer_index: Option<i32>,
    _pointer_handler: Rc<RefCell<MouseHandler>>,
}

/// The callback type for the handler to call on a mouse event
pub type OnPointerEventCb = dyn FnMut(ControlEvent) -> bool;

pub enum ControlEvent {
    Down(Coordinates),
    Move(Coordinates),
    Up(),
}

impl Controls {
    pub fn new(
        target: &HtmlElement,
        on_event_cb: Box<OnPointerEventCb>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<Self>>| {
            let self_ref = self_ref.clone();
            RefCell::new(Self {
                on_event_cb,
                drag_pointer_index: None,
                _pointer_handler: MouseHandler::new(
                    target,
                    Box::new(move |event| -> bool {
                        self_ref.upgrade().unwrap().borrow_mut().handle_event(event)
                    }),
                )
                .expect("Couldn't register mouse handler"),
            })
        }))
    }

    fn handle_event(&mut self, event: PointerEvent) -> bool {
        let mut handled = false;
        match event {
            PointerEvent::Down((index, _timestamp, coordinates)) => {
                // If we were not dragging yet, then try to initiate a drag
                if self.drag_pointer_index.is_none()
                    && (self.on_event_cb)(ControlEvent::Down(coordinates))
                {
                    self.drag_pointer_index = Some(index);
                    handled = true;
                }
            }
            PointerEvent::Up((index, _timestamp, _coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    self.drag_pointer_index = None;
                    handled = (self.on_event_cb)(ControlEvent::Up());
                }
            }
            PointerEvent::Move((index, _timestamp, coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    handled = (self.on_event_cb)(ControlEvent::Move(coordinates));
                }
            }
            PointerEvent::TouchMove() => {
                // Prevent default behavior of touchmove if pointer is down
                handled = self.drag_pointer_index.is_some()
            }
            PointerEvent::Wheel(_) => {}
        }
        handled
    }
}
