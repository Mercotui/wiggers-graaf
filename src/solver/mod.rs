use crate::board::{
    get_solved_board, get_start_board, get_valid_moves, is_solution, Board, Coordinates,
    SlideDirection, SlideMove,
};
use crate::graph::Graph;

pub struct Solver {
    pub graph: Graph,
    start_board: Board,
    solution_node: Board,
}

impl Solver {
    /// Creates a new solver instance and builds the graph from scratch
    pub fn new() -> Solver {
        // Create graph
        let mut solver: Solver = Solver {
            graph: Graph::new(),
            start_board: get_start_board(),
            solution_node: get_solved_board(),
        };

        // Add solution to graph
        solver.graph.add_node(solver.solution_node);

        // Find all possible moves from start
        generate_moves(&mut solver);

        // Analyze the moves to find the distances from each node to start and solution
        solver
            .graph
            .analyze(&solver.start_board, &solver.solution_node);

        solver
    }
}

fn generate_moves(solver: &mut Solver) {
    // Create process queue and initialize it with the start board
    let mut inspection_queue: Vec<Board> = vec![solver.start_board.clone()];

    while !inspection_queue.is_empty() {
        // print!(".");
        let board = inspection_queue
            .pop()
            .expect("Failed to pop from queue, is it empty?");
        solver.graph.add_node(board);

        get_valid_moves(&board)
            .iter()
            .for_each(|(slide_move, new_board)| {
                if !solver.graph.contains_node(new_board) {
                    inspection_queue.push(*new_board)
                }
                solver.graph.add_edge(&board, new_board, &slide_move);
            });

        if is_solution(&board) {
            // TODO(Menno 13.11.2024) Add some fake SlideMove edge value and undo the pub on SlideMove and components
            solver.graph.add_edge(
                &board,
                &solver.solution_node,
                &SlideMove {
                    start: Coordinates { x: 1, y: 0 },
                    direction: SlideDirection::Down,
                },
            );
            solver.graph.add_edge(
                &solver.solution_node,
                &board,
                &SlideMove {
                    start: Coordinates { x: 1, y: 0 },
                    direction: SlideDirection::Down,
                },
            );
        }
    }
}
