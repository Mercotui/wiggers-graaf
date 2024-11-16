#[cfg(test)]
mod unittest;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use std::cmp::{Ordering, PartialEq};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Size {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct SlideMove {
    pub start: Coordinates,
    pub direction: SlideDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Piece {
    /// The coordinates of the piece's bottom left most tile
    position: Coordinates,
    /// The size, in the x direction right, and y direction up
    size: Size,
}

/// A game board filled with all tiles
#[derive(Debug, Clone, Copy)]
pub struct Board {
    pieces: [Piece; 10],
    size: Size,
}

impl Eq for Board {}

impl PartialEq<Self> for Board {
    fn eq(&self, other: &Self) -> bool {
        let mut sorted_self = self.pieces;
        let mut sorted_other = other.pieces;
        self.size.eq(&other.size) && sorted_self.sort().eq(&sorted_other.sort())
    }
}

impl PartialOrd<Self> for Board {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut sorted_self = self.pieces;
        let mut sorted_other = other.pieces;
        sorted_self.sort().partial_cmp(&sorted_other.sort())
    }
}

impl Ord for Board {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut sorted_self = self.pieces;
        let mut sorted_other = other.pieces;
        sorted_self.sort().cmp(&sorted_other.sort())
    }
}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut sorted_pieces= self.pieces;
        sorted_pieces.sort();
        sorted_pieces.hash(state);
        self.size.hash(state);
    }
}

pub fn get_start_board() -> Board {
    /// Standard Klotski board is 4 by 5 tiles
    const SIZE: Size = Size { x: 4, y: 5 };

    /// Standard Klotski board is:
    /// ABBC
    /// ABBC
    /// DEEF
    /// DGHF
    /// I  J
    const PIECES: [Piece; 10] = [
        // A
        Piece {
            position: Coordinates { x: 0, y: 3 },
            size: Size { x: 1, y: 2 },
        },
        // B
        Piece {
            position: Coordinates { x: 1, y: 3 },
            size: Size { x: 2, y: 2 },
        },
        // C
        Piece {
            position: Coordinates { x: 3, y: 3 },
            size: Size { x: 1, y: 2 },
        },
        // D
        Piece {
            position: Coordinates { x: 0, y: 1 },
            size: Size { x: 1, y: 2 },
        },
        // E
        Piece {
            position: Coordinates { x: 1, y: 2 },
            size: Size { x: 2, y: 1 },
        },
        // F
        Piece {
            position: Coordinates { x: 3, y: 1 },
            size: Size { x: 1, y: 2 },
        },
        // G
        Piece {
            position: Coordinates { x: 1, y: 1 },
            size: Size { x: 1, y: 1 },
        },
        // H
        Piece {
            position: Coordinates { x: 2, y: 1 },
            size: Size { x: 1, y: 1 },
        },
        // I
        Piece {
            position: Coordinates { x: 0, y: 0 },
            size: Size { x: 1, y: 1 },
        },
        // J
        Piece {
            position: Coordinates { x: 3, y: 0 },
            size: Size { x: 1, y: 1 },
        },
    ];

    Board {
        pieces: PIECES,
        size: SIZE,
    }
}

pub fn get_solved_board() -> Board {
    /// The solution criterion for Klotski is:
    /// ....
    /// ....
    /// ....
    /// .BB.
    /// .BB.
    /// Where . can be any piece
    const PIECES: [Piece; 10] = [
        // A, fake
        Piece {
            position: Coordinates { x: 0, y: 0 },
            size: Size { x: 1, y: 1 },
        },
        // B, we only care about this big piece, the other pieces here are dummy data to pad the array to 10
        Piece {
            position: Coordinates { x: 1, y: 0 },
            size: Size { x: 2, y: 2 },
        },
        // C, fake
        Piece {
            position: Coordinates { x: 0, y: 1 },
            size: Size { x: 1, y: 1 },
        },
        // D, fake
        Piece {
            position: Coordinates { x: 0, y: 2 },
            size: Size { x: 1, y: 1 },
        },
        // E, fake
        Piece {
            position: Coordinates { x: 0, y: 3 },
            size: Size { x: 1, y: 1 },
        },
        // F, fake
        Piece {
            position: Coordinates { x: 0, y: 4 },
            size: Size { x: 1, y: 1 },
        },
        // G, fake
        Piece {
            position: Coordinates { x: 3, y: 0 },
            size: Size { x: 1, y: 1 },
        },
        // H, fake
        Piece {
            position: Coordinates { x: 3, y: 1 },
            size: Size { x: 1, y: 1 },
        },
        // I, fake
        Piece {
            position: Coordinates { x: 3, y: 2 },
            size: Size { x: 1, y: 1 },
        },
        // J, fake
        Piece {
            position: Coordinates { x: 3, y: 3 },
            size: Size { x: 1, y: 1 },
        },
    ];

    Board {
        pieces: PIECES,
        size: get_start_board().size,
    }
}

pub fn get_valid_moves(board: &Board) -> Vec<(SlideMove, Board)> {
    board
        .pieces
        .iter()
        .cartesian_product([
            SlideDirection::Up,
            SlideDirection::Down,
            SlideDirection::Left,
            SlideDirection::Right,
        ])
        .filter_map(|(piece, slide_direction)| {
            let slide_move: SlideMove = SlideMove {
                start: piece.position,
                direction: slide_direction,
            };
            let new_board = make_move(&board, &slide_move);
            return if new_board.is_ok() {
                Option::from((slide_move, new_board.unwrap()))
            } else {
                None
            };
        })
        .collect()
}

pub fn make_move(board: &Board, slide_move: &SlideMove) -> Result<Board> {
    // Copy the board into a new board
    let mut new_board = *board;

    // Find the piece at the move start coordinates
    let piece_to_move: &mut Piece = new_board
        .pieces
        .iter_mut()
        .find(|piece: &&mut Piece| piece.position == slide_move.start)
        .context("No piece to move")?;

    // move the piece by 1
    match slide_move.direction {
        SlideDirection::Up => piece_to_move.position.y += 1,
        SlideDirection::Down => piece_to_move.position.y -= 1,
        SlideDirection::Left => piece_to_move.position.x -= 1,
        SlideDirection::Right => piece_to_move.position.x += 1,
    }

    if !is_valid(&new_board) {
        return Err(anyhow!("Invalid move"));
    }
    Ok(new_board)
}

/// Find if this board is a valid solution
pub fn is_solution(board: &Board) -> bool {
    board.pieces.iter().any(|piece: &Piece| {
        piece.size.x == 2 && piece.size.y == 2 && piece.position.x == 1 && piece.position.y == 0
    })
}

/// Check that board only contains validly placed pieces
fn is_valid(board: &Board) -> bool {
    board
        .pieces
        .iter()
        .all(|piece: &Piece| is_on_board(piece, board))
        && !has_collision(board)
}

/// Check that piece is entirely contained within the bounds of the board
fn is_on_board(piece: &Piece, board: &Board) -> bool {
    (piece.position.x + piece.size.x) <= board.size.x
        && (piece.position.y + piece.size.y) <= board.size.y
}

fn has_collision(board: &Board) -> bool {
    let is_inside = |coordinates: &Coordinates, piece: &Piece| -> bool {
        // Check if coordinates fall inside a piece
        piece.position.x <= coordinates.x
            && coordinates.x < (piece.position.x + piece.size.x)
            && piece.position.y <= coordinates.y
            && coordinates.y < (piece.position.y + piece.size.y)
    };

    // for permutation of 2 pieces, check if any collide
    board
        .pieces
        .iter()
        .permutations(2)
        .any(|piece_combo: Vec<&Piece>| -> bool {
            // Note: we only check if B starts within A, because the inverse check is done elsewhere in the iteration.
            return if is_inside(&piece_combo[0].position, &piece_combo[1]) {
                log::debug!(
                    "Collision between {:?} and {:?}",
                    piece_combo[0],
                    piece_combo[1]
                );
                true
            } else {
                false
            };
        })
}
