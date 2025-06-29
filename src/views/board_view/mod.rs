// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

mod layout;

use crate::board;
use crate::board::Board;
use crate::views::board_view::layout::Layout;
use crate::views::frame_scheduler::FrameScheduler;
use crate::views::resize_observer::ResizeObserver;
use crate::views::utils::get_canvas;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::Duration;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::console::error_1;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn create_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    Ok(canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?)
}

/**
 * The lookup a piece size in the color scheme (palette from https://mycolor.space/?hex=%23754BFF&sub=1)
 * @param size the size of the piece
 * @param highlight whether the piece is currently highlighted
 * @returns the color of the piece
 */
fn get_color(size: &board::Size) -> String {
    if size.x == 1 && size.y == 1 {
        return "75,123,255".into();
    } else if size.x == 1 && size.y == 2 {
        return "117,75,255".into();
    } else if size.x == 2 && size.y == 1 {
        return "75,213,255".into();
    } else if size.x == 2 && size.y == 2 {
        return "255,207,75".into();
    }
    error_1(&JsValue::from_str(
        format!("Unknown Piece size: (x: {}, y: {})", size.x, size.y).as_str(),
    ));
    "255,0,255".into()
}

/// A visual representation of a game piece
struct VisualPiece {
    size: VisualSize,
    highlighted: bool,
    color: String,
    position: VisualCoordinates,
    /// The visual position offset can be used for animations or user interactions
    visual_offset: VisualCoordinates,
}

/// A visual representation of a gameboard
pub struct VisualBoard {
    size: VisualSize,
    pieces: HashMap<board::Coordinates, VisualPiece>,
}

pub type VisualCoordinates = euclid::Point2D<f64, VisualBoard>;
pub type VisualSize = euclid::Size2D<f64, VisualBoard>;

pub struct BoardView {
    _self_ref: Weak<RefCell<Self>>,
    frame_scheduler: FrameScheduler,
    _resize_observer: ResizeObserver,
    visual_board: VisualBoard,
    layout: Layout,
    canvas: HtmlCanvasElement,
    canvas_size: layout::Size,
    ctx: CanvasRenderingContext2d,
}
impl BoardView {
    pub fn new(canvas_id: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let canvas = get_canvas(canvas_id)?;
        let ctx = create_context_2d(&canvas)?;
        Ok(Rc::new_cyclic(|self_ref: &Weak<RefCell<BoardView>>| {
            let self_ref_for_on_frame_cb = self_ref.clone();
            let self_ref_for_resize_observer_cb = self_ref.clone();

            let refcell_self = RefCell::new(Self {
                _self_ref: self_ref.clone(),
                frame_scheduler: FrameScheduler::new(Box::new(move |timestamp: Duration| {
                    self_ref_for_on_frame_cb
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .draw(timestamp);
                })),
                _resize_observer: ResizeObserver::new(
                    &canvas,
                    Box::new(move |width, height| {
                        self_ref_for_resize_observer_cb
                            .upgrade()
                            .unwrap()
                            .borrow_mut()
                            .resize(width, height);
                    }),
                ),
                visual_board: VisualBoard {
                    size: VisualSize::zero(),
                    pieces: Default::default(),
                },
                layout: Layout::new(VisualSize::zero(), layout::Size::zero(), 0.0),
                canvas,
                canvas_size: layout::Size::zero(),
                ctx,
            });
            refcell_self
        }))
    }

    pub fn set_board(&mut self, board: &Board) {
        self.visual_board = VisualBoard {
            size: VisualSize::new(board.size.x as f64, board.size.y as f64),
            pieces: board
                .pieces
                .iter()
                .map(|piece| {
                    (
                        piece.position,
                        VisualPiece {
                            size: VisualSize::new(piece.size.x as f64, piece.size.y as f64),
                            highlighted: false,
                            color: get_color(&piece.size),
                            position: VisualCoordinates::new(
                                piece.position.x as f64,
                                piece.position.y as f64,
                            ),
                            visual_offset: VisualCoordinates::zero(),
                        },
                    )
                })
                .collect(),
        };
        self.frame_scheduler.schedule().unwrap();
    }

    /// Store the new size, application of the size is deferred to the draw function, to avoid flashes.
    fn resize(&mut self, width: f64, height: f64) {
        self.layout.invalidate();
        self.canvas_size = layout::Size::new(width, height);
        self.frame_scheduler.schedule().unwrap();
    }

    fn draw(&mut self, timestamp: Duration) {
        if self.layout.is_valid() {
            // Clear the canvas
            self.ctx.clear_rect(
                0.0,
                0.0,
                self.canvas.width() as f64,
                self.canvas.height() as f64,
            );
        } else {
            // Resize the canvas, which also clears it
            self.canvas.set_width(self.canvas_size.width as u32);
            self.canvas.set_height(self.canvas_size.height as u32);
            self.layout = Layout::new(
                self.visual_board.size,
                layout::Size::new(self.canvas_size.width, self.canvas_size.height),
                web_sys::window().unwrap().device_pixel_ratio(),
            );
        }

        // Draw the game pieces
        self.visual_board
            .pieces
            .iter()
            .for_each(|(position, piece)| self.draw_piece(position, piece));

        // Draw the axes
        // TODO(Menno 22.06.2025) Draw the axes
    }

    /**
     * Draw a single game piece to the canvas
     * @param piece the piece to draw
     */
    fn draw_piece(&self, position: &board::Coordinates, piece: &VisualPiece) {
        self.ctx.begin_path();

        let opacity: f64 = if piece.highlighted { 1.0 } else { 0.8 };
        self.ctx
            .set_fill_style_str(format!("rgba({},{})", piece.color, opacity).as_str());

        let (pos, size, corner_radius) = self.layout.apply_to_piece(piece);
        self.ctx
            .round_rect_with_f64(pos.x, pos.y, size.width, size.height, corner_radius)
            .expect("Failed to draw piece");
        self.ctx.fill();
    }
}
