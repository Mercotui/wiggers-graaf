// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::pointer_handler::{MouseHandler, PointerEvent};
use crate::views::utils::{Coordinates, Delta};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

const REST_TIME_THRESHOLD: Duration = Duration::from_millis(10);

pub struct Controls {
    on_event_cb: Box<OnPointerEventCb>,
    // TODO(Menno 04.09.2025) Track multiple pointers for gestures
    drag_pointer_index: Option<i32>,
    previous_drag_coordinates: Coordinates,
    previous_drag_timestamp: Duration,
    drag_velocity: Delta,
    _pointer_handler: Rc<RefCell<MouseHandler>>,
}

/// The callback type for the handler to call on a mouse event
pub type OnPointerEventCb = dyn FnMut(ControlEvent);

pub enum ControlEvent {
    /// Contains the contact point on the canvas
    Down(Coordinates),
    /// Contains the delta in pixels since the last event
    Move(Delta),
    /// Contains the current drag velocity in pixels per seconds
    Up(Delta),
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
                previous_drag_timestamp: Duration::from_secs(0),
                drag_velocity: Delta::zero(),
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
            PointerEvent::Down((index, timestamp, coordinates)) => {
                if self.drag_pointer_index.is_none() {
                    self.drag_pointer_index = Some(index);
                    self.previous_drag_coordinates = coordinates;
                    self.previous_drag_timestamp = timestamp;
                    self.drag_velocity = Delta::zero();
                    (self.on_event_cb)(ControlEvent::Down(coordinates));
                    handled = true;
                }
            }
            PointerEvent::Up((index, timestamp, _coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    self.drag_pointer_index = None;

                    // If the last move event was a while ago, then we ignore the velocity.
                    let time_delta = timestamp - self.previous_drag_timestamp;
                    if time_delta > REST_TIME_THRESHOLD {
                        self.drag_velocity = Delta::zero();
                    }

                    (self.on_event_cb)(ControlEvent::Up(self.drag_velocity));
                    handled = true;
                }
            }
            PointerEvent::Move((index, timestamp, coordinates)) => {
                if self.drag_pointer_index == Some(index) {
                    let delta = coordinates - self.previous_drag_coordinates;
                    let delta_time = timestamp - self.previous_drag_timestamp;
                    self.drag_velocity = delta / delta_time.as_secs_f64();
                    self.previous_drag_timestamp = timestamp;

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
