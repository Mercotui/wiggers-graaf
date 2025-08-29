// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::controls::add_listener;
use crate::views::utils::{Coordinates, Delta, Size};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

/// The callback type for the handler to call on a touch event, returns if the event was handled
pub type OnTouchEventCb = dyn FnMut(TouchEvent) -> bool;

#[derive(Clone, Copy)]
pub enum TouchEvent {
    Start(Coordinates),
    Move(Delta),
    End,
}
pub struct TouchHandler {
    device_pixel_ratio: f64,
    target: HtmlElement,
    on_event_cb: Box<OnTouchEventCb>,
    // TODO(Menno 29.08.2025) Turns out touch events don't work as I hoped, seems I need to transition to pointer events instead
    previous_drag_client_coordinates: Option<Coordinates>,
}

impl TouchEvent {
    pub fn from_touchstart(
        event: &web_sys::TouchEvent,
        target_origin: Size,
        previous_coordinates: &mut Option<Coordinates>,
        device_pixel_ratio: f64,
    ) -> Self {
        let touch = event
            .changed_touches()
            .get(0)
            .expect("Touch event has no touch");
        let coordinates =
            Coordinates::new(touch.client_x() as f64, touch.client_y() as f64) * device_pixel_ratio;
        *previous_coordinates = Some(coordinates);

        Self::Start(coordinates - target_origin)
    }

    pub fn from_touchend(
        _event: &web_sys::TouchEvent,
        previous_coordinates: &mut Option<Coordinates>,
        _device_pixel_ratio: f64,
    ) -> Self {
        *previous_coordinates = None;
        Self::End
    }

    pub fn from_touchmove(
        event: &web_sys::TouchEvent,
        previous_coordinates: &mut Option<Coordinates>,
        device_pixel_ratio: f64,
    ) -> Self {
        let touch = event
            .changed_touches()
            .get(0)
            .expect("Touch event has no touch");
        let coordinates =
            Coordinates::new(touch.client_x() as f64, touch.client_y() as f64) * device_pixel_ratio;
        let delta: Delta = match previous_coordinates {
            Some(previous_coordinates) => coordinates - *previous_coordinates,
            None => Delta::zero(),
        };
        *previous_coordinates = Some(coordinates);
        Self::Move(delta)
    }
}

impl TouchHandler {
    pub fn new(
        target: &HtmlElement,
        on_event_cb: Box<OnTouchEventCb>,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        let self_ref = Rc::new(RefCell::new(Self {
            device_pixel_ratio: web_sys::window().unwrap().device_pixel_ratio(),
            target: target.clone(),
            on_event_cb,
            previous_drag_client_coordinates: None,
        }));

        let self_ref_clone = self_ref.clone();
        add_listener(
            target,
            "touchstart",
            Box::new(move |browser_event: web_sys::TouchEvent| {
                let mut self_mut = self_ref_clone.borrow_mut();
                let device_pixel_ratio = self_mut.device_pixel_ratio;
                // This is supposedly not performant due to forcing layout computation, only get it on click down
                let target_rect = self_mut.target.get_bounding_client_rect();
                let target_origin = Size::new(target_rect.left(), target_rect.top());
                let event = TouchEvent::from_touchstart(
                    &browser_event,
                    target_origin,
                    &mut self_mut.previous_drag_client_coordinates,
                    device_pixel_ratio,
                );

                if (self_mut.on_event_cb)(event) {
                    browser_event.prevent_default();
                }
            }),
        );
        // The next event listeners listen to the entire window, so that the user can generously
        // move around, dragging and release anywhere.
        let window = web_sys::window().expect("Unable to access the window");
        Self::add_touch_event_listener(
            &window,
            "touchend",
            self_ref.clone(),
            TouchEvent::from_touchend,
        );
        Self::add_touch_event_listener(
            &window,
            "touchmove",
            self_ref.clone(),
            TouchEvent::from_touchmove,
        );

        Ok(self_ref)
    }

    fn add_touch_event_listener(
        target: &web_sys::EventTarget,
        event_name: &str,
        self_ref: Rc<RefCell<Self>>,
        event_mapping_fn: fn(&web_sys::TouchEvent, &mut Option<Coordinates>, f64) -> TouchEvent,
    ) {
        add_listener(
            target,
            event_name,
            Box::new(move |browser_event: web_sys::TouchEvent| {
                let mut self_mut = self_ref.borrow_mut();
                let device_pixel_ratio = self_mut.device_pixel_ratio;
                let event = event_mapping_fn(
                    &browser_event,
                    &mut self_mut.previous_drag_client_coordinates,
                    device_pixel_ratio,
                );

                if (self_mut.on_event_cb)(event) {
                    browser_event.prevent_default();
                }
            }),
        );
    }
}
