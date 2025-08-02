// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::utils::Coordinates;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;

/// The callback type for the handler to call on a mouse event
pub type OnMouseEventCb = dyn FnMut(MouseEvent);

pub struct MouseWheel {}

pub enum MouseEvent {
    Down(Coordinates),
    Up(),
    Move(Coordinates),
    Wheel(MouseWheel),
}

pub struct MouseHandler {
    is_down: bool,
    on_event_cb: Box<OnMouseEventCb>,
}

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

impl MouseEvent {
    pub fn from_mousedown(event: web_sys::MouseEvent) -> Self {
        Self::Down(Coordinates::new(
            event.offset_x() as f64,
            event.offset_y() as f64,
        ))
    }

    pub fn from_mouseup(_event: web_sys::MouseEvent) -> Self {
        Self::Up()
    }

    pub fn from_mousemove(event: web_sys::MouseEvent) -> Self {
        Self::Move(Coordinates::new(
            event.offset_x() as f64,
            event.offset_y() as f64,
        ))
    }

    pub fn from_wheel(_event: web_sys::MouseScrollEvent) -> Self {
        // TODO(Menno 02.08.2025) Implement scroll handling
        Self::Wheel(MouseWheel {})
    }
}

impl MouseHandler {
    pub fn new(
        target: &HtmlElement,
        on_event_cb: Box<OnMouseEventCb>,
    ) -> Result<Rc<RefCell<MouseHandler>>, JsValue> {
        let self_ref = Rc::new(RefCell::new(Self {
            is_down: false,
            on_event_cb,
        }));

        let self_ref_clone = self_ref.clone();
        add_listener(
            target,
            "wheel",
            Box::new(move |event: web_sys::MouseScrollEvent| {
                (self_ref_clone.borrow_mut().on_event_cb)(MouseEvent::from_wheel(event));
            }),
        );

        let self_ref_clone = self_ref.clone();
        add_listener(
            target,
            "mousedown",
            Box::new(move |event: web_sys::MouseEvent| {
                let mut self_mut = self_ref_clone.borrow_mut();
                self_mut.is_down = true;
                (self_mut.on_event_cb)(MouseEvent::from_mousedown(event));
            }),
        );
        // The next event listeners listen to the entire window, so that the user can generously
        // move the mouse around, dragging and release anywhere.
        let window = web_sys::window().expect("Unable to access the window");
        let self_ref_clone = self_ref.clone();
        add_listener(
            &window,
            "mouseup",
            Box::new(move |event: web_sys::MouseEvent| {
                let mut self_mut = self_ref_clone.borrow_mut();
                self_mut.is_down = false;
                (self_mut.on_event_cb)(MouseEvent::from_mouseup(event));
            }),
        );

        let self_ref_clone = self_ref.clone();
        add_listener(
            &window,
            "mousemove",
            Box::new(move |event: web_sys::MouseEvent| {
                let mut self_mut = self_ref_clone.borrow_mut();
                if self_mut.is_down {
                    (self_mut.on_event_cb)(MouseEvent::from_mousemove(event));
                }
            }),
        );

        Ok(self_ref)
    }
}
