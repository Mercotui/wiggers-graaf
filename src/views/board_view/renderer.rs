// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board;
use crate::board::Axis;
use crate::views::board_view::layout::Layout;
use crate::views::board_view::visual_board::{VisualBoard, VisualSize};
use crate::views::utils::Coordinates;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d,
};

const AXIS_COLOR: &str = "rgb(179, 179, 179)";

fn create_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    Ok(canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?)
}

fn create_offscreen_context_2d(
    canvas: &OffscreenCanvas,
) -> Result<OffscreenCanvasRenderingContext2d, JsValue> {
    Ok(canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<OffscreenCanvasRenderingContext2d>()?)
}

pub struct Renderer {
    layout: Layout,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    axes_canvas: OffscreenCanvas,
    axes_ctx: OffscreenCanvasRenderingContext2d,
}

impl Renderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = create_context_2d(&canvas)?;
        let axes_canvas = OffscreenCanvas::new(10, 10)?;
        let axes_ctx = create_offscreen_context_2d(&axes_canvas)?;

        Ok(Self {
            layout: Layout::zero(),
            canvas,
            ctx,
            axes_canvas,
            axes_ctx,
        })
    }

    pub fn draw(&mut self, board: &VisualBoard, layout: &Layout) {
        // Don't draw if our layout isn't valid
        if layout.is_zero() {
            return;
        }

        // Rebuild the cache if needed
        if self.cache_layout(layout) {
            // cache was cleared, draw the axes for this layout into cache
            self.draw_axes(&board.size);
        }

        let ctx = &self.ctx;

        // Set the blend mode to ignore the destination buffer,
        // this means that we can use draw_image_with_offscreen_canvas to effectively clear the destination canvas.
        ctx.set_global_composite_operation("copy")
            .expect("Could not set compositing to copy");
        // Reset canvas with axes
        ctx.draw_image_with_offscreen_canvas(&self.axes_canvas, 0.0, 0.0)
            .expect("Could not draw axes to screen");
        ctx.set_global_composite_operation("source-over")
            .expect("Could not reset compositing");

        // Draw the game pieces
        board.pieces.iter().for_each(|(_, piece)| {
            ctx.begin_path();

            let opacity: f64 = if piece.highlighted { 1.0 } else { 0.8 };
            ctx.set_fill_style_str(format!("rgba({},{opacity})", piece.color).as_str());

            let (pos, size, corner_radius) = self.layout.apply_to_piece(piece);
            ctx.round_rect_with_f64(pos.x, pos.y, size.width, size.height, corner_radius)
                .expect("Failed to draw piece");
            ctx.fill();
        });
    }

    fn draw_axes(&self, board_size: &VisualSize) {
        let ctx = &self.axes_ctx;

        // Draw all ticks
        let draw_ticks = |axis: Axis, count: u32| {
            for index in 0..=count {
                let (pos, size) = self.layout.apply_to_axis_tick(index, &axis);
                ctx.fill_rect(pos.x, pos.y, size.width, size.height);
            }
        };
        draw_ticks(Axis::Horizontal, board_size.width as u32);
        draw_ticks(Axis::Vertical, board_size.height as u32);

        // Draw a label
        let draw_label = |position: Coordinates, label: String| {
            ctx.fill_text(label.as_str(), position.x, position.y)
                .expect("Could not draw axis label")
        };

        // draw X axis labes
        for x in 0..board_size.width as u32 {
            let label = board::Coordinates::axis_to_string(Axis::Horizontal, x as u8);
            draw_label(self.layout.apply_to_axis_label(x, &Axis::Horizontal), label);
        }

        // draw Y axis labels
        for y in 0..board_size.height as u32 {
            let label = board::Coordinates::axis_to_string(Axis::Vertical, y as u8);
            draw_label(self.layout.apply_to_axis_label(y, &Axis::Vertical), label);
        }
    }

    /// Rebuild render cache if layout changed
    /// @param layout the layout to apply
    /// @return true if the cache was cleared
    fn cache_layout(&mut self, layout: &Layout) -> bool {
        if *layout == self.layout {
            return false;
        }
        self.layout = *layout;

        // Resize the canvases, which also clears them
        let size = self.layout.get_canvas_size();
        self.canvas.set_width(size.width as u32);
        self.canvas.set_height(size.height as u32);
        self.axes_canvas.set_width(size.width as u32);
        self.axes_canvas.set_height(size.height as u32);

        // Reapply style, as it gets wiped during resize
        // Set font, text centering, and color for axis labels and ticks
        // Roboto Mono should have been loaded from the server by CSS
        self.axes_ctx
            .set_font(format!("{}px Roboto Mono", self.layout.axis_label_font_size_px()).as_str());
        self.axes_ctx.set_text_align("center");
        self.axes_ctx.set_text_baseline("middle");
        self.axes_ctx.set_fill_style_str(AXIS_COLOR);

        true
    }
}
