// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board;
use crate::board::{Board, SlideDirection};
use futures::channel::oneshot;
use keyframe::{keyframes, AnimationSequence, CanTween};
use std::collections::HashMap;
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::console::error_1;

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

fn resolve_if_sender(sender: &mut Option<oneshot::Sender<()>>) {
    if let Some(animation_done_signal) = sender.take() {
        // If someone is still listening, inform then them we have finished
        animation_done_signal.send(()).unwrap_or(());
    }
}

/// Granular Coordinates with the same scale as Board::Coordinates
pub type VisualCoordinates = euclid::Point2D<f64, VisualBoard>;
pub type VisualSize = euclid::Size2D<f64, VisualBoard>;
pub type VisualOffset = euclid::Vector2D<f64, VisualBoard>;

pub type VisualRect = euclid::Rect<f64, VisualBoard>;

/// Coordinates that implement CanTween
#[derive(Clone, Copy, Default)]
pub struct AnimatableOffset(pub VisualOffset);

/// How to act when the animation finishes
pub enum AnimationRepeatBehavior {
    None,
    Loop,
}

/// An animation that acts on a single GamePiece
pub struct Animation {
    pub sequence: AnimationSequence<AnimatableOffset>,
    pub target: board::Coordinates,
    pub repeat: AnimationRepeatBehavior,
}

/// The metadata needed to run an Animation
struct AnimationExecution {
    pub animation: Animation,
    pub start_time: Option<Duration>,
    pub done_sender: Option<oneshot::Sender<()>>,
}

struct Drag {
    pub target: board::Coordinates,
    /// Where this dragging started, stored as size so that it can be used for arithmetic
    pub start_coordinates: Option<VisualCoordinates>,
}

/// An optional dynamic element to a game board
enum DynamicElement {
    None,
    Animation(AnimationExecution),
    Drag(Drag),
}

/// A visual representation of a game piece
pub struct VisualPiece {
    pub rect: VisualRect,
    pub visual_offset: VisualOffset,
    pub highlighted: bool,
    pub color: String,
}

/// A visual representation of a gameboard
pub struct VisualBoard {
    pub size: VisualSize,
    pub pieces: HashMap<board::Coordinates, VisualPiece>,
    dynamic_element: DynamicElement,
}

impl AnimatableOffset {
    pub fn new(x: f64, y: f64) -> Self {
        Self(VisualOffset::new(x, y))
    }

    pub fn zero() -> Self {
        Self(VisualOffset::zero())
    }

    pub fn from_distance_and_direction(distance: f64, slide_direction: SlideDirection) -> Self {
        match slide_direction {
            SlideDirection::Up => Self::new(0.0, distance),
            SlideDirection::Down => Self::new(0.0, -distance),
            SlideDirection::Left => Self::new(-distance, 0.0),
            SlideDirection::Right => Self::new(distance, 0.0),
        }
    }
}

impl CanTween for AnimatableOffset {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        Self(from.0.lerp(
            to.0,
            time.to_f64().expect("time cannot be converted to f64"),
        ))
    }
}

impl VisualBoard {
    pub fn new(board: &Board) -> Self {
        Self {
            size: VisualSize::new(board.size.x as f64, board.size.y as f64),
            pieces: board
                .pieces
                .iter()
                .map(|piece| {
                    (
                        piece.position,
                        VisualPiece {
                            rect: VisualRect::new(
                                VisualCoordinates::new(
                                    piece.position.x as f64,
                                    piece.position.y as f64,
                                ),
                                VisualSize::new(piece.size.x as f64, piece.size.y as f64),
                            ),
                            visual_offset: VisualOffset::zero(),
                            highlighted: false,
                            color: get_color(&piece.size),
                        },
                    )
                })
                .collect(),
            dynamic_element: DynamicElement::None,
        }
    }

    pub fn empty() -> Self {
        Self {
            size: VisualSize::zero(),
            pieces: Default::default(),
            dynamic_element: DynamicElement::None,
        }
    }

    pub fn highlight(&mut self, target: &Option<board::Coordinates>) {
        if let Some(target) = target {
            self.pieces
                .get_mut(target)
                .expect("Trying to highlight nonexistent piece")
                .highlighted = true;
        } else {
            for piece in self.pieces.values_mut() {
                piece.highlighted = false;
            }
        }
    }

    pub fn set_drag_target(&mut self, target: Option<VisualCoordinates>) {
        let Some(target) = target else {
            // Return the dragged piece home
            self.start_post_drag_animation();
            return;
        };

        // Find if the cursor is targeting a piece
        let piece: Option<board::Coordinates> = self
            .pieces
            .iter_mut()
            .find(|(_, piece)| piece.rect.contains(target))
            .map(|(base_coordinates, _)| *base_coordinates);

        self.highlight(&piece);
        if let Some(target) = piece {
            // If we are currently running an animation, then clear it
            self.clear_animation();

            // Start dragging
            self.dynamic_element = DynamicElement::Drag(Drag {
                target,
                start_coordinates: None,
            });
        }
    }

    fn start_post_drag_animation(&mut self) {
        // Find the piece that needs returning
        let target = {
            // If we are not dragging, then early exit
            let DynamicElement::Drag(drag) = &mut self.dynamic_element else {
                return;
            };
            drag.target
        };

        self.highlight(&None);

        let from = AnimatableOffset(
            self.pieces
                .get_mut(&target)
                .expect("Trying to post drag animate nonexistent piece")
                .visual_offset,
        );
        let to = AnimatableOffset::zero();

        let animation_done = self.animate(Some(Animation {
            sequence: keyframes![(from, 0.0, keyframe::functions::EaseInOutCubic), (to, 0.15)],
            target,
            repeat: AnimationRepeatBehavior::None,
        }));
        drop(animation_done);
    }

    pub fn drag(&mut self, coordinates: VisualCoordinates) {
        // If we are not dragging, then early exit
        let DynamicElement::Drag(drag) = &mut self.dynamic_element else {
            return;
        };

        let start = drag.start_coordinates.get_or_insert(coordinates);
        let offset = coordinates - *start;

        self.pieces
            .get_mut(&drag.target)
            .expect("Trying to drag nonexistent piece")
            .visual_offset = offset;
    }

    pub fn animate(&mut self, animation: Option<Animation>) -> oneshot::Receiver<()> {
        // Clear animation, if there is currently one
        self.clear_animation();

        // (Re)set the animation
        let (sender, receiver) = oneshot::channel();
        if let Some(animation) = animation {
            self.dynamic_element = DynamicElement::Animation(AnimationExecution {
                animation,
                start_time: None,
                done_sender: Some(sender),
            });
        } else {
            self.dynamic_element = DynamicElement::None;
        }
        receiver
    }

    fn clear_animation(&mut self) {
        if let DynamicElement::Animation(animation) = &mut self.dynamic_element {
            // Reset the visual board
            self.pieces
                .iter_mut()
                .for_each(|(_, piece)| piece.visual_offset = VisualOffset::zero());

            resolve_if_sender(&mut animation.done_sender)
        };
    }

    pub fn update_to(&mut self, timestamp: Duration) -> Result<(), ()> {
        // Decompose the current AnimationExecution, if any
        let DynamicElement::Animation(AnimationExecution {
            animation,
            start_time,
            done_sender,
        }) = &mut self.dynamic_element
        else {
            // We are not running an animation
            return Err(());
        };

        // Update the animated value
        let diff = timestamp - *start_time.get_or_insert(timestamp);
        let excess_time = animation.sequence.advance_to(diff.as_secs_f64());
        if let Some(piece) = self.pieces.get_mut(&animation.target) {
            piece.visual_offset = animation.sequence.now().0;
        }

        // If the animation is finished, handle the looping behavior
        if excess_time > 0.0 {
            match animation.repeat {
                AnimationRepeatBehavior::Loop => {
                    // TODO(Menno 08.07.2025) For now we discard the excess time, which might result
                    //  in a stutter if the animation is not at rest at the loop end/start.
                    *start_time = None;
                }
                AnimationRepeatBehavior::None => {
                    resolve_if_sender(done_sender);

                    self.dynamic_element = DynamicElement::None;
                    return Err(());
                }
            }
        }
        Ok(())
    }
}
