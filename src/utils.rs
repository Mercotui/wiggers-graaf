// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

pub fn get_canvas(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
    // Access DOM
    let document = web_sys::window()
        .ok_or(JsValue::from_str("Unable to access the window"))?
        .document()
        .ok_or(JsValue::from_str("Unable to access the DOM"))?;
    // Get canvas element from DOM
    document
        .get_element_by_id(canvas_id)
        .ok_or(JsValue::from_str(&format!(
            "Could not find canvas: {}",
            canvas_id
        )))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_x| {
            JsValue::from_str(&format!(
                "Element with ID {} does not appear to be a canvas",
                canvas_id
            ))
        })
}
