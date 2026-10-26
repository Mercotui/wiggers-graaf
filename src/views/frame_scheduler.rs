// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::utils::dom_high_res_timestamp_to_duration;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::Window;

pub type OnFrameCb = dyn FnMut(Duration);
type OnFrameCbInternal = dyn FnMut(f64);

pub struct FrameRequester {
    frame_wanted: AtomicBool,
}

impl FrameRequester {
    pub fn new() -> Self {
        Self {
            frame_wanted: AtomicBool::new(false),
        }
    }

    pub fn request_frame(&self) {
        self.frame_wanted.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.frame_wanted.store(false, Ordering::Relaxed);
    }

    pub fn is_frame_requested(&self) -> bool {
        self.frame_wanted.load(Ordering::Relaxed)
    }
}

pub struct FrameScheduler {
    window: Window,
    on_frame_closure: Closure<OnFrameCbInternal>,
    frame_request_id: i32,
    frame_scheduled: Rc<Cell<bool>>,
    pub frame_requester: Rc<FrameRequester>,
}

impl FrameScheduler {
    pub fn new(mut on_frame_cb: Box<OnFrameCb>) -> Self {
        let frame_scheduled = Rc::new(Cell::new(false));

        let frame_scheduled_clone = frame_scheduled.clone();
        let on_frame_closure = Closure::new(move |timestamp: f64| {
            frame_scheduled_clone.set(false);
            on_frame_cb(dom_high_res_timestamp_to_duration(timestamp));
        });

        let mut instance = Self {
            window: web_sys::window().expect("Unable to access the window"),
            on_frame_closure,
            frame_request_id: 0,
            frame_scheduled,
            frame_requester: Rc::new(FrameRequester::new()),
        };
        instance.schedule().expect("Unable to schedule first frame");
        instance
    }

    pub fn schedule(&mut self) -> Result<(), JsValue> {
        self.frame_requester.reset();

        if self.frame_scheduled.get() {
            // A frame request is already pending.
            return Ok(());
        }
        self.frame_request_id = self
            .window
            .request_animation_frame(self.on_frame_closure.as_ref().unchecked_ref())?;
        self.frame_scheduled.set(true);
        Ok(())
    }

    pub fn _cancel(&self) {
        self.window
            .cancel_animation_frame(self.frame_request_id)
            .unwrap();
    }
}
