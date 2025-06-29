// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, ResizeObserverEntry, ResizeObserverSize};

/// Get the content's (width, height) in device pixels
fn get_size(entry: ResizeObserverEntry) -> (f64, f64) {
    // Try to access devicePixelContentBoxSize, availability depends on browser
    if let Ok(size) = entry
        .device_pixel_content_box_size()
        .get(0)
        .dyn_into::<ResizeObserverSize>()
    {
        // pixel-perfect size for modern browsers
        return (size.inline_size(), size.block_size());
    }

    // best-effort fallback for Safari
    let size = entry
        .content_box_size()
        .get(0)
        .dyn_into::<ResizeObserverSize>()
        .unwrap();
    let device_pixel_ratio = web_sys::window().unwrap().device_pixel_ratio();
    (
        (size.inline_size() * device_pixel_ratio).round(),
        (size.block_size() * device_pixel_ratio).round(),
    )
}

/// The callback type for the observer to call on a resize event
pub type OnResizeCb = dyn FnMut(f64, f64);

/// An observer that tracks the content size of the target in device pixels
pub struct ResizeObserver {
    observer: web_sys::ResizeObserver,
}

impl ResizeObserver {
    /// Register the on_resize_cb for resize events on target
    pub fn new(target: &Element, mut on_resize_cb: Box<OnResizeCb>) -> Self {
        let instance = Self {
            observer: web_sys::ResizeObserver::new(
                Closure::new(move |entries: js_sys::Array| {
                    let entry: ResizeObserverEntry = entries.at(0).dyn_into().unwrap();
                    let (width, height) = get_size(entry);
                    on_resize_cb(width, height);
                })
                .into_js_value()
                .unchecked_ref(),
            )
            .unwrap(),
        };
        instance.observer.observe(target);
        instance
    }
}
