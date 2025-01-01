use crate::board::{get_start_board, make_move, Coordinates, SlideDirection, SlideMove};
use crate::graph::{to_id, Graph};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_analyse() {
    init();
    let mut graph: Graph = Graph::new();

    let board_1 = get_start_board();
    let move_1 = SlideMove {
        start: Coordinates { x: 1, y: 1 },
        direction: SlideDirection::Down,
        distance: 1,
    };
    let board_2 = make_move(
        &board_1,
        &move_1,
    ).expect("Failed to make move");
    let move_2 = SlideMove {
        start: Coordinates { x: 1, y: 0 },
        direction: SlideDirection::Right,
        distance: 1,
    };
    let board_3 = make_move(
        &board_2,
        &move_2,
    ).expect("Failed to make move");
    let move_3 = SlideMove {
        start: Coordinates { x: 2, y: 1 },
        direction: SlideDirection::Left,
        distance: 1,
    };
    let board_4 = make_move(
        &board_3,
        &move_3,
    ).expect("Failed to make move");
    let move_4 = SlideMove {
        start: Coordinates { x: 2, y: 0 },
        direction: SlideDirection::Up,
        distance: 1,
    };
    // This should be the start board once again, we have swapped around to similar pieces
    let board_5 = make_move(
        &board_4,
        &move_4,
    ).expect("Failed to make move");

    assert_eq!(board_1, board_5);

    graph.add_node(board_1);
    graph.add_node(board_2);
    graph.add_node(board_3);
    graph.add_node(board_4);
    // This should not have effect, as board_5 equals board_1
    graph.add_node(board_5);

    graph.add_edge(&board_1, &board_2, &move_1);
    graph.add_edge(&board_2, &board_1, &move_1);
    graph.add_edge(&board_2, &board_3, &move_2);
    graph.add_edge(&board_3, &board_2, &move_2);
    graph.add_edge(&board_3, &board_4, &move_3);
    graph.add_edge(&board_4, &board_3, &move_3);
    graph.add_edge(&board_4, &board_5, &move_4);
    graph.add_edge(&board_5, &board_4, &move_4);

    graph.analyze(&board_1, &board_4);

    assert_eq!(graph.map.get(&to_id(&board_1)).unwrap().distance_to_start.unwrap(), 0);
    assert_eq!(graph.map.get(&to_id(&board_2)).unwrap().distance_to_start.unwrap(), 1);
    assert_eq!(graph.map.get(&to_id(&board_3)).unwrap().distance_to_start.unwrap(), 2);
    assert_eq!(graph.map.get(&to_id(&board_4)).unwrap().distance_to_start.unwrap(), 1);
    assert_eq!(graph.map.get(&to_id(&board_5)).unwrap().distance_to_start.unwrap(), 0);

    assert_eq!(graph.map.get(&to_id(&board_1)).unwrap().distance_to_solution.unwrap(), 1);
    assert_eq!(graph.map.get(&to_id(&board_2)).unwrap().distance_to_solution.unwrap(), 2);
    assert_eq!(graph.map.get(&to_id(&board_3)).unwrap().distance_to_solution.unwrap(), 1);
    assert_eq!(graph.map.get(&to_id(&board_4)).unwrap().distance_to_solution.unwrap(), 0);
    assert_eq!(graph.map.get(&to_id(&board_5)).unwrap().distance_to_solution.unwrap(), 1);
}
