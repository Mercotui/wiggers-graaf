// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

/// The CanvasSpace unit type
pub struct CanvasSpace {}

/// 2D Coordinates in canvas space
pub type Coordinates = euclid::Point2D<f64, CanvasSpace>;

/// 2D Size in canvas space
pub type Size = euclid::Size2D<f64, CanvasSpace>;

/// Get HTML canvas element from its ID
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
            "Could not find canvas: {canvas_id}"
        )))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_x| {
            JsValue::from_str(&format!(
                "Element with ID {canvas_id} does not appear to be a canvas"
            ))
        })
}
