// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Window;

pub type OnFrameCb = dyn FnMut(Duration);
type OnFrameCbInternal = dyn FnMut(f64);

pub struct FrameScheduler {
    window: Window,
    on_frame_closure: Closure<OnFrameCbInternal>,
    frame_request_id: i32,
    frame_requested: Rc<Cell<bool>>,
}

impl FrameScheduler {
    pub fn new(mut on_frame_cb: Box<OnFrameCb>) -> Self {
        let frame_requested = Rc::new(Cell::new(false));

        let frame_requested_clone = frame_requested.clone();
        let on_frame_closure = Closure::new(move |timestamp: f64| {
            frame_requested_clone.set(false);

            // The DOMHighResTimeStamp is in milliseconds, convert it to a std time Duration
            let timestamp = Duration::from_micros((timestamp * 1000.0) as u64);
            on_frame_cb(timestamp);
        });

        Self {
            window: web_sys::window().expect("Unable to access the window"),
            on_frame_closure,
            frame_request_id: 0,
            frame_requested,
        }
    }

    pub fn schedule(&mut self) -> Result<(), JsValue> {
        if self.frame_requested.get() {
            // A frame request is already pending.
            return Ok(());
        }
        self.frame_request_id = self
            .window
            .request_animation_frame(self.on_frame_closure.as_ref().unchecked_ref())?;
        self.frame_requested.set(true);
        Ok(())
    }

    pub fn _cancel(&self) {
        self.window
            .cancel_animation_frame(self.frame_request_id)
            .unwrap();
    }
}
