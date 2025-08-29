// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, Window};

/// The CanvasSpace unit type
pub struct CanvasSpace {}

/// 2D Coordinates in canvas space
pub type Coordinates = euclid::Point2D<f64, CanvasSpace>;

/// 2D Size in canvas space
pub type Size = euclid::Size2D<f64, CanvasSpace>;

/// 2D Vector in canvas space
pub type Delta = euclid::Vector2D<f64, CanvasSpace>;

/// Get window
pub fn get_window() -> Result<Window, JsValue> {
    web_sys::window().ok_or(JsValue::from_str("Unable to access the window"))
}

/// Get document
pub fn get_document() -> Result<Document, JsValue> {
    get_window()?
        .document()
        .ok_or(JsValue::from_str("Unable to access the DOM"))
}

/// Find an HTML Element by its ID
pub fn get_element(id: &str) -> Result<Element, JsValue> {
    // Access DOM
    let document = get_document()?;
    // Get element from DOM
    document
        .get_element_by_id(id)
        .ok_or(JsValue::from_str(&format!(
            "Could not find element for ID: {id}"
        )))
}

/// Get HTML Element by its ID, and convert it
pub fn get_element_of_type<T: JsCast>(id: &str) -> Result<T, JsValue> {
    get_element(id)?.dyn_into::<T>().map_err(|_x| {
        JsValue::from_str(&format!(
            "Element with ID {id} does not appear to be of type {}",
            std::any::type_name::<T>()
        ))
    })
}
