// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::utils;
use crate::views::utils::Coordinates;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AddEventListenerOptions, Element, HtmlElement};

/// The callback type for the handler to call on a pointer event
pub type OnMouseEventCb = dyn FnMut(PointerEvent) -> bool;

#[derive(Debug)]
pub struct MouseWheel {}

#[derive(Debug)]
pub enum PointerEvent {
    Down((i32, Duration, Coordinates)),
    Up((i32, Duration, Coordinates)),
    Move((i32, Duration, Coordinates)),
    TouchMove(),
    Wheel(MouseWheel),
}

pub struct MouseHandler {
    target: Element,
    device_pixel_ratio: f64,
    on_event_cb: Box<OnMouseEventCb>,
}

impl MouseHandler {
    pub fn new(
        target: &HtmlElement,
        on_event_cb: Box<OnMouseEventCb>,
    ) -> Result<Rc<RefCell<MouseHandler>>, JsValue> {
        let self_ref = Rc::new(RefCell::new(Self {
            target: target.clone().into(),
            device_pixel_ratio: web_sys::window().unwrap().device_pixel_ratio(),
            on_event_cb,
        }));

        Self::add_listener(target, "wheel", self_ref.clone(), Self::handle_wheel);
        Self::add_listener(
            target,
            "pointerdown",
            self_ref.clone(),
            Self::handle_pointerdown,
        );
        Self::add_listener(
            target,
            "pointerup",
            self_ref.clone(),
            Self::handle_pointerup,
        );
        Self::add_listener(
            target,
            "pointercancel",
            self_ref.clone(),
            Self::handle_pointerup,
        );
        Self::add_listener(
            target,
            "pointermove",
            self_ref.clone(),
            Self::handle_pointermove,
        );
        Self::add_listener(
            target,
            "touchmove",
            self_ref.clone(),
            Self::handle_touchmove,
        );

        Ok(self_ref)
    }

    fn add_listener<
        TEvent: AsRef<web_sys::Event> + wasm_bindgen::convert::FromWasmAbi + 'static,
    >(
        target: &web_sys::EventTarget,
        event_name: &str,
        self_ref: Rc<RefCell<Self>>,
        event_mapping_fn: fn(&TEvent, &Element, f64) -> PointerEvent,
    ) {
        let options = AddEventListenerOptions::new();
        options.set_passive(false);
        target
            .add_event_listener_with_callback_and_add_event_listener_options(
                event_name,
                Closure::<dyn FnMut(TEvent)>::new(Box::new(move |browser_event: TEvent| {
                    let mut self_mut = self_ref.borrow_mut();
                    let device_pixel_ratio = self_mut.device_pixel_ratio;
                    let event =
                        event_mapping_fn(&browser_event, &self_mut.target, device_pixel_ratio);
                    if (self_mut.on_event_cb)(event) {
                        let browser_event = browser_event.as_ref();
                        browser_event.prevent_default();
                        browser_event.stop_propagation();
                    }
                }))
                .into_js_value()
                .unchecked_ref(),
                &options,
            )
            .expect("Couldn't register event listener");
    }

    pub fn handle_pointerdown(
        event: &web_sys::PointerEvent,
        target: &Element,
        device_pixel_ratio: f64,
    ) -> PointerEvent {
        target
            .set_pointer_capture(event.pointer_id())
            .expect("Could not capture input pointer");
        PointerEvent::Down((
            event.pointer_id(),
            utils::dom_high_res_timestamp_to_duration(event.time_stamp()),
            Coordinates::new(event.offset_x() as f64, event.offset_y() as f64) * device_pixel_ratio,
        ))
    }

    pub fn handle_pointerup(
        event: &web_sys::PointerEvent,
        _target: &Element,
        device_pixel_ratio: f64,
    ) -> PointerEvent {
        PointerEvent::Up((
            event.pointer_id(),
            utils::dom_high_res_timestamp_to_duration(event.time_stamp()),
            Coordinates::new(event.offset_x() as f64, event.offset_y() as f64) * device_pixel_ratio,
        ))
    }

    pub fn handle_pointermove(
        event: &web_sys::PointerEvent,
        _target: &Element,
        device_pixel_ratio: f64,
    ) -> PointerEvent {
        PointerEvent::Move((
            event.pointer_id(),
            utils::dom_high_res_timestamp_to_duration(event.time_stamp()),
            Coordinates::new(event.offset_x() as f64, event.offset_y() as f64) * device_pixel_ratio,
        ))
    }

    pub fn handle_touchmove(
        _event: &web_sys::TouchEvent,
        _target: &Element,
        _device_pixel_ratio: f64,
    ) -> PointerEvent {
        PointerEvent::TouchMove()
    }

    pub fn handle_wheel(
        _event: &web_sys::MouseScrollEvent,
        _target: &Element,
        _device_pixel_ratio: f64,
    ) -> PointerEvent {
        // TODO(Menno 02.08.2025) Implement scroll handling
        PointerEvent::Wheel(MouseWheel {})
    }
}
