// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board;
use crate::board::{Board, SlideDirection};
use futures::channel::oneshot;
use keyframe::{AnimationSequence, CanTween};
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

/// A visual representation of a game piece
pub struct VisualPiece {
    pub size: VisualSize,
    pub highlighted: bool,
    pub color: String,
    pub position: VisualCoordinates,
    pub visual_offset: VisualCoordinates,
}

/// A visual representation of a gameboard
pub struct VisualBoard {
    pub size: VisualSize,
    pub pieces: HashMap<board::Coordinates, VisualPiece>,
    pub animation: Option<Animation>,
    pub animation_done_sender: Option<oneshot::Sender<()>>,
    pub animation_start_time: Option<Duration>,
}

pub type VisualCoordinates = euclid::Point2D<f64, VisualBoard>;

#[derive(Clone, Copy, Default)]
pub struct AnimatableCoordinates(pub VisualCoordinates);

pub enum AnimationRepeatBehavior {
    None,
    Loop,
}

pub struct Animation {
    pub sequence: AnimationSequence<AnimatableCoordinates>,
    pub target: board::Coordinates,
    pub repeat: AnimationRepeatBehavior,
}

impl AnimatableCoordinates {
    pub fn new(x: f64, y: f64) -> Self {
        Self(euclid::Point2D::new(x, y))
    }

    pub fn zero() -> Self {
        Self(euclid::Point2D::zero())
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

impl CanTween for AnimatableCoordinates {
    fn ease(from: Self, to: Self, time: impl keyframe::num_traits::Float) -> Self {
        Self(from.0.lerp(
            to.0,
            time.to_f64().expect("time cannot be converted to f64"),
        ))
    }
}

pub type VisualSize = euclid::Size2D<f64, VisualBoard>;

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
            animation: None,
            animation_done_sender: None,
            animation_start_time: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            size: VisualSize::zero(),
            pieces: Default::default(),
            animation: None,
            animation_done_sender: None,
            animation_start_time: None,
        }
    }

    pub fn highlight(&mut self, target: Option<&board::Coordinates>) {
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

    pub fn animate(&mut self, animation: Option<Animation>) -> oneshot::Receiver<()> {
        // Reset the visual board
        self.pieces
            .iter_mut()
            .for_each(|(_, piece)| piece.visual_offset = VisualCoordinates::zero());

        // (Re)set the animation
        self.animation = animation;
        self.animation_start_time = None;
        let (sender, receiver) = oneshot::channel();
        self.animation_done_sender = Some(sender);
        receiver
    }

    pub fn update_to(&mut self, timestamp: Duration) -> Result<(), ()> {
        let Some(animation) = &mut self.animation else {
            return Err(());
        };
        let start_time = self.animation_start_time.get_or_insert(timestamp);

        // Update the animated value
        let diff = timestamp - *start_time;
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
                    self.animation_start_time = None;
                }
                AnimationRepeatBehavior::None => {
                    if let Some(animation_done_signal) = self.animation_done_sender.take() {
                        // If someone is still listening, inform them the animation has finished
                        animation_done_signal.send(()).unwrap_or(());
                    }
                    self.animation = None;
                    return Err(());
                }
            }
        }
        Ok(())
    }
}
