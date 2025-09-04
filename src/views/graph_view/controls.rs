// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::pointer_handler::{MouseHandler, PointerEvent};
use crate::views::utils::{Coordinates, Delta};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

pub struct Controls {
    on_event_cb: Box<OnPointerEventCb>,
    // TODO(Menno 04.09.2025) Track multiple pointers for gestures
    drag_pointer_index: Option<i32>,
    previous_drag_coordinates: Coordinates,
    _pointer_handler: Rc<RefCell<MouseHandler>>,
}

/// The callback type for the handler to call on a mouse event
pub type OnPointerEventCb = dyn FnMut(ControlEvent);

pub enum ControlEvent {
    Down(Coordinates),
    Move(Delta),
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
                previous_drag_coordinates: Coordinates::zero(),
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
                if self.drag_pointer_index.is_none() {
                    self.drag_pointer_index = Some(index);
                    self.previous_drag_coordinates = coordinates;
                    (self.on_event_cb)(ControlEvent::Down(coordinates));
                    handled = true;
                }
            }
            PointerEvent::Up((index, _timestamp, _coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    self.drag_pointer_index = None;
                    (self.on_event_cb)(ControlEvent::Up());
                    handled = true;
                }
            }
            PointerEvent::Move((index, _timestamp, coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    let delta = coordinates - self.previous_drag_coordinates;
                    (self.on_event_cb)(ControlEvent::Move(delta));
                    self.previous_drag_coordinates = coordinates;
                    handled = true;
                }
            }
            PointerEvent::TouchMove() => {
                handled = true;
            }
            PointerEvent::Wheel(_) => {}
        }
        handled
    }
}
