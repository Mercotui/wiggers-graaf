// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

use crate::board::{
    get_solved_board, get_start_board, get_valid_moves, is_solution, is_valid, Coordinates,
    SlideDirection, SlideMove,
};
use std::hash::Hash;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_is_solution() {
    init();
    // Start board is not a solution
    assert!(!is_solution(&get_start_board()));
    // Solved board is literally the solution criterion
    assert!(is_solution(&get_solved_board()));
}

#[test]
fn test_is_valid() {
    init();
    // Start and solved boards should be valid
    assert!(is_valid(&get_solved_board()));
    assert!(is_valid(&get_start_board()));
}

#[test]
fn test_board_hash() {
    use std::hash::{DefaultHasher, Hasher};

    init();
    // Test two identical objects have the same hash:
    let mut hasher_1 = DefaultHasher::new();
    get_start_board().hash(&mut hasher_1);
    let mut hasher_2 = DefaultHasher::new();
    get_start_board().hash(&mut hasher_2);
    assert_eq!(hasher_1.finish(), hasher_2.finish());

    // Test two differing objects have the differing hashes:
    let mut hasher_3 = DefaultHasher::new();
    get_solved_board().hash(&mut hasher_3);
    assert_ne!(hasher_2.finish(), hasher_3.finish());
}

#[test]
fn test_get_valid_moves() {
    init();

    let moves = get_valid_moves(&get_start_board());
    assert_eq!(moves.len(), 6);
    assert_eq!(
        moves[0].0,
        SlideMove {
            start: Coordinates { x: 0, y: 0 },
            direction: SlideDirection::Right,
            distance: 1,
        }
    );
    assert_eq!(
        moves[1].0,
        SlideMove {
            start: Coordinates { x: 0, y: 0 },
            direction: SlideDirection::Right,
            distance: 2,
        }
    );
    assert_eq!(
        moves[2].0,
        SlideMove {
            start: Coordinates { x: 1, y: 1 },
            direction: SlideDirection::Down,
            distance: 1,
        }
    );
    assert_eq!(
        moves[3].0,
        SlideMove {
            start: Coordinates { x: 2, y: 1 },
            direction: SlideDirection::Down,
            distance: 1,
        }
    );
    assert_eq!(
        moves[4].0,
        SlideMove {
            start: Coordinates { x: 3, y: 0 },
            direction: SlideDirection::Left,
            distance: 1,
        }
    );    assert_eq!(
        moves[5].0,
        SlideMove {
            start: Coordinates { x: 3, y: 0 },
            direction: SlideDirection::Left,
            distance: 2,
        }
    );
}
