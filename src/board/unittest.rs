use std::hash::{Hash, Hasher};
use log::info;
use crate::board::{get_solved_board, get_start_board, is_solution, is_valid};

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
fn test_board_hash(){
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
