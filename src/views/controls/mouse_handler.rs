// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::controls::add_listener;
use crate::views::utils::{Coordinates, Delta};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

/// The callback type for the handler to call on a mouse event
pub type OnMouseEventCb = dyn FnMut(MouseEvent);

pub struct MouseWheel {}

pub enum MouseEvent {
    Down(Coordinates),
    Up(),
    Move(Delta),
    Wheel(MouseWheel),
}

pub struct MouseHandler {
    device_pixel_ratio: f64,
    on_event_cb: Box<OnMouseEventCb>,
}

impl MouseEvent {
    pub fn from_mousedown(event: web_sys::MouseEvent, device_pixel_ratio: f64) -> Self {
        Self::Down(
            Coordinates::new(event.offset_x() as f64, event.offset_y() as f64) * device_pixel_ratio,
        )
    }

    pub fn from_mouseup(_event: web_sys::MouseEvent, _device_pixel_ratio: f64) -> Self {
        Self::Up()
    }

    pub fn from_mousemove(event: web_sys::MouseEvent, device_pixel_ratio: f64) -> Self {
        Self::Move(
            Delta::new(event.movement_x() as f64, event.movement_y() as f64) * device_pixel_ratio,
        )
    }

    pub fn from_wheel(_event: web_sys::MouseScrollEvent, _device_pixel_ratio: f64) -> Self {
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
            device_pixel_ratio: web_sys::window().unwrap().device_pixel_ratio(),
            on_event_cb,
        }));

        Self::add_mouse_event_listener(target, "wheel", self_ref.clone(), MouseEvent::from_wheel);
        Self::add_mouse_event_listener(
            target,
            "mousedown",
            self_ref.clone(),
            MouseEvent::from_mousedown,
        );

        // The next event listeners listen to the entire window, so that the user can generously
        // move the mouse around, dragging and release anywhere.
        let window = web_sys::window().expect("Unable to access the window");
        Self::add_mouse_event_listener(
            &window,
            "mouseup",
            self_ref.clone(),
            MouseEvent::from_mouseup,
        );
        Self::add_mouse_event_listener(
            &window,
            "mousemove",
            self_ref.clone(),
            MouseEvent::from_mousemove,
        );

        Ok(self_ref)
    }

    fn add_mouse_event_listener<TEvent: wasm_bindgen::convert::FromWasmAbi + 'static>(
        target: &web_sys::EventTarget,
        event_name: &str,
        self_ref: Rc<RefCell<Self>>,
        event_mapping_fn: fn(TEvent, f64) -> MouseEvent,
    ) {
        add_listener(
            target,
            event_name,
            Box::new(move |event: TEvent| {
                let mut self_mut = self_ref.borrow_mut();
                let event = event_mapping_fn(event, self_mut.device_pixel_ratio);
                (self_mut.on_event_cb)(event);
            }),
        );
    }
}
