// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::controls::mouse_handler::{MouseEvent, MouseHandler};
use crate::views::controls::touch_handler::{TouchEvent, TouchHandler};
use crate::views::utils::Coordinates;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

pub mod mouse_handler;
pub mod touch_handler;

fn add_listener<TEvent: wasm_bindgen::convert::FromWasmAbi + 'static>(
    target: &web_sys::EventTarget,
    event_type: &str,
    cb: Box<dyn FnMut(TEvent)>,
) {
    target
        .add_event_listener_with_callback(
            event_type,
            Closure::<dyn FnMut(TEvent)>::new(cb)
                .into_js_value()
                .unchecked_ref(),
        )
        .expect("Couldn't register event listener");
}

pub struct PointerControls {
    on_event_cb: Box<OnPointerEventCb>,
    drag_coordinates: Option<Coordinates>,
    // TODO(Menno 29.08.2025) Replace mouse handler and touch handler with  pointer event handler
    _mouse_handler: Rc<RefCell<MouseHandler>>,
    _touch_handler: Rc<RefCell<TouchHandler>>,
}

/// The callback type for the handler to call on a mouse event
pub type OnPointerEventCb = dyn FnMut(PointerEvent) -> bool;

pub struct Zoom {}

pub enum PointerEvent {
    Down(Coordinates),
    Up(),
    Move(Coordinates),
    Zoom(Zoom),
}

impl PointerControls {
    pub fn new(
        target: &HtmlElement,
        on_event_cb: Box<OnPointerEventCb>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<Self>>| {
            let self_ref_clone_mouse = self_ref.clone();
            let self_ref_clone_touch = self_ref.clone();
            RefCell::new(Self {
                on_event_cb,
                drag_coordinates: None,
                _mouse_handler: MouseHandler::new(
                    target,
                    Box::new(move |event| {
                        self_ref_clone_mouse
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .handle_mouse(event);
                    }),
                )
                .expect("Couldn't register mouse handler"),
                _touch_handler: TouchHandler::new(
                    target,
                    Box::new(move |event| {
                        self_ref_clone_touch
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .handle_touch(event)
                    }),
                )
                .expect("Couldn't register touch handler"),
            })
        }))
    }

    fn handle_mouse(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::Down(coordinates) => {
                self.drag_coordinates.get_or_insert(coordinates);
                (self.on_event_cb)(PointerEvent::Down(coordinates));
            }
            MouseEvent::Up() => {
                self.drag_coordinates.take();
                (self.on_event_cb)(PointerEvent::Up());
            }
            MouseEvent::Move(delta) => {
                if let Some(drag_coordinates) = &mut self.drag_coordinates {
                    *drag_coordinates += delta;
                    (self.on_event_cb)(PointerEvent::Move(*drag_coordinates));
                }
            }
            MouseEvent::Wheel(_) => {}
        }
    }

    /// Handle a touch event, returns true if this event is accepted, or false if it is ignored.
    fn handle_touch(&mut self, event: TouchEvent) -> bool {
        match event {
            TouchEvent::Start(coordinates) => {
                self.drag_coordinates.get_or_insert(coordinates);
                (self.on_event_cb)(PointerEvent::Down(coordinates))
            }
            TouchEvent::End => {
                self.drag_coordinates.take();
                (self.on_event_cb)(PointerEvent::Up())
            }
            TouchEvent::Move(delta) => {
                if let Some(drag_coordinates) = &mut self.drag_coordinates {
                    *drag_coordinates += delta;
                    (self.on_event_cb)(PointerEvent::Move(*drag_coordinates))
                } else {
                    false
                }
            }
        }
    }
}
