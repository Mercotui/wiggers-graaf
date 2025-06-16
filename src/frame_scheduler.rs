// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Window;

type OnFrameCb = dyn FnMut(f64);

pub struct FrameScheduler {
    window: Window,
    on_frame_cb: Closure<OnFrameCb>,
    frame_requested: Rc<Cell<bool>>,
}

impl FrameScheduler {
    /// A no-operation scheduler. Does nothing!
    pub fn noop() -> Self {
        FrameScheduler::new(Box::new(|_| {})).unwrap()
    }

    pub fn new(on_frame_cb: Box<OnFrameCb>) -> Result<Self, JsValue> {
        let frame_requested = Rc::new(Cell::new(false));
        let on_frame_closure = Closure::wrap(Box::new(move |timestamp: f64| {
            frame_requested.set(false);

            let timestamp = Duration::from_micros((timestamp * 1000.0) as u64);
            (*on_frame_cb).call(timestamp);
        }));

        Ok(Self {
            window: web_sys::window().ok_or(JsValue::from_str("Unable to access the window"))?,
            on_frame_closure,
            frame_requested,
        })
    }

    pub fn schedule(&self) -> Result<(), JsValue> {
        if self.frame_requested.get() {
            return Ok(());
        }
        self.frame_requested.set(true);
        // TODO(Menno 18.05.2025) No idea if this works
        self.window
            .request_animation_frame(self.on_frame_cb.as_ref().unchecked_ref())?;
        Ok(())
    }

    pub fn cancel() {}
}
