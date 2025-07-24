// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::views::board_view::visual_board::{VisualCoordinates, VisualPiece, VisualSize};

const AXIS_PADDING: f64 = 4.0;
const AXIS_GIRTH: f64 = 16.0;
const PIECE_PADDING: f64 = 1.0;

pub struct CanvasSpace {}

pub type Coordinates = euclid::Point2D<f64, CanvasSpace>;
pub type Size = euclid::Size2D<f64, CanvasSpace>;

pub enum Axis {
    Horizontal,
    Vertical,
}

/// Layout of a board in canvas space, note we flip the Y-axis for this layout, 0 is at the bottom.
#[derive(PartialEq, Copy, Clone)]
pub struct Layout {
    scale: f64,
    piece_gap: f64,
    canvas: Size,
    board_offset: Coordinates,
    axis_girth: f64,
    horizontal_axis_offset: Coordinates,
    vertical_axis_offset: Coordinates,
}

impl Layout {
    /// Create an empty layout
    pub fn zero() -> Self {
        Self::new(VisualSize::zero(), Size::zero(), 0.0)
    }

    /// Calculate a draw layout for a given board
    /// @param board_size the number of tiles in either axis
    /// @param canvas_size the number of device pixels in either axis
    /// @param axis_size
    /// @returns object containing a scale of pixels per game unit, and offset in pixels to center the board inside the canvas
    pub fn new(board: VisualSize, canvas: Size, device_pixel_ratio: f64) -> Self {
        // Find space required by the axes
        let axis_girth = (AXIS_GIRTH * device_pixel_ratio).round();
        let axis_padding = (AXIS_PADDING * device_pixel_ratio).round();
        let axis_size = axis_girth + axis_padding;
        let piece_gap = (PIECE_PADDING * device_pixel_ratio).round();

        // Add pixel gaps flanking each element (also on the edges of the board for axis rendering).
        let gaps_x = piece_gap * (board.width + 1.0);
        let gaps_y = piece_gap * (board.height + 1.0);

        // Find the smallest scale, x or Y, to fit the board inside the canvas
        // Note, double the vertical axis for horizontal symmetry.
        // Also floor the scale so that fractional pixels are avoided.
        let scale_x = (canvas.width - (gaps_x + (axis_size * 2.0))) / board.width;
        let scale_y = (canvas.height - (gaps_y + axis_size)) / board.height;
        let rendering_scale = f64::min(scale_x, scale_y).max(0.0).floor();

        // Calculate the offset of the content, the y-axis is not symmetrical
        let content_offset_x = 0.5 * (canvas.width - (gaps_x + (rendering_scale * board.width)));
        // This offset is relative to the bottom of the canvas
        let content_offset_y =
            0.5 * (canvas.height - (gaps_y + (rendering_scale * board.height) - axis_size));

        // Calculate the offset of the axes, subtract the tick that should start before the content
        let horizontal_axis_offset_x = content_offset_x - piece_gap;
        let horizontal_axis_offset_y = content_offset_y - axis_padding;

        let vertical_axis_offset_x = content_offset_x - axis_size;
        let vertical_axis_offset_y = content_offset_y;

        Self {
            scale: rendering_scale,
            piece_gap,
            canvas,
            board_offset: Coordinates::new(content_offset_x, content_offset_y),
            axis_girth,
            horizontal_axis_offset: Coordinates::new(
                horizontal_axis_offset_x,
                horizontal_axis_offset_y,
            ),
            vertical_axis_offset: Coordinates::new(vertical_axis_offset_x, vertical_axis_offset_y),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.canvas.is_empty() || self.scale == 0.0
    }

    pub fn get_canvas_size(&self) -> Size {
        self.canvas
    }

    pub fn apply_to_piece(&self, piece: &VisualPiece) -> (Coordinates, Size, f64) {
        let pos = VisualCoordinates::new(
            piece.position.x + piece.visual_offset.x,
            piece.position.y + piece.visual_offset.y,
        );
        let size = piece.size;

        // Start rendering from xy offset, then each piece gets an additional pixel offset to create a gap between each other.
        let x: f64 = self.board_offset.x + pos.x * (self.scale + self.piece_gap);
        let y: f64 =
            self.canvas.height - (self.board_offset.y + pos.y * (self.scale + self.piece_gap));
        let width: f64 = (size.width * (self.scale + self.piece_gap)) - self.piece_gap;
        let height: f64 = -((size.height * (self.scale + self.piece_gap)) - self.piece_gap);
        let corner_radius: f64 = 0.1 * self.scale;

        (
            Coordinates::new(x, y),
            Size::new(width, height),
            corner_radius,
        )
    }

    /// Apply the layout to find where to draw the label
    /// @returns the center of the text label
    pub fn apply_to_axis_label(&self, index: u32, axis: &Axis) -> Coordinates {
        // We use long and narrow side, ls and ns, to denote the axis width and height, depending on orientation.
        let position_ns = self.axis_girth * 0.5;
        let label_start_ls = index as f64 * (self.scale + self.piece_gap);
        let position_ls = label_start_ls + self.scale * 0.5;

        match axis {
            Axis::Horizontal => Coordinates::new(
                self.horizontal_axis_offset.x + position_ls + self.piece_gap,
                self.canvas.height - (self.horizontal_axis_offset.y - position_ns),
            ),
            // Swap the position xy, and size width and height
            Axis::Vertical => Coordinates::new(
                self.vertical_axis_offset.x + position_ns,
                self.canvas.height - (self.vertical_axis_offset.y + position_ls),
            ),
        }
    }

    pub fn apply_to_axis_tick(&self, index: u32, axis: &Axis) -> (Coordinates, Size) {
        // We use long side, ls to denote the axis width or height, depending on orientation.
        let size = Size::new(self.piece_gap, self.axis_girth);
        let position_ls = index as f64 * (self.scale + self.piece_gap);

        match axis {
            Axis::Horizontal => (
                Coordinates::new(
                    self.horizontal_axis_offset.x + position_ls,
                    self.canvas.height - self.horizontal_axis_offset.y,
                ),
                size,
            ),
            Axis::Vertical => {
                // Swap the position xy, and size width and height
                (
                    Coordinates::new(
                        self.vertical_axis_offset.x,
                        self.canvas.height - (self.vertical_axis_offset.y + position_ls),
                    ),
                    Size::new(size.height, size.width),
                )
            }
        }
    }

    pub fn axis_label_font_size_px(&self) -> u8 {
        self.axis_girth as u8
    }
}
