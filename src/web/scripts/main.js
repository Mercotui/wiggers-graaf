import init, {generate, draw, get_start_id, get_state} from "../pkg/wiggers_graaf.js";
import * as gameBoard from "./game-board.js";
import * as gameMoves from "./game-moves.js";

await init();

const GAME_CONTROL_RESTART_ID = "game-control-restart";
const GAME_CONTROL_SOLVE_ID = "game-control-solve";
const META_CONTAINER_ID = "meta-container";
const META_CANVAS_ID = "meta-canvas";
const GAME_CANVAS_ID = "game-canvas";
const GAME_MOVES_DIV_ID = "game-moves"

let solver;
let current_state;
let current_state_id;
let auto_solve_toggle_div = document.getElementById(GAME_CONTROL_SOLVE_ID);
let is_auto_solve_enabled = false;
let auto_solve_timer;

function RegisterControls() {
    let restart_button = document.getElementById(GAME_CONTROL_RESTART_ID)
    restart_button.onclick = event => {
        restart_button.classList.add("clicked");
        gameBoard.cancelMove();
        setAutoSolve(false);
        setCurrentState(get_start_id());
        setTimeout(() => {
            restart_button.classList.remove("clicked")
        }, 200)
    };

    auto_solve_toggle_div.onclick = event => {
        // Toggle auto-solve
        setAutoSolve(!is_auto_solve_enabled);
    };
}

function setAutoSolve(enable) {
    if (is_auto_solve_enabled === enable) {
        // do nothing
        return;
    }

    is_auto_solve_enabled = enable;
    if (is_auto_solve_enabled) {
        auto_solve_toggle_div.classList.add("clicked");
        // Start chain of moves
        gameMoves.doBestMove();
    } else {
        clearTimeout(auto_solve_timer);
        auto_solve_toggle_div.classList.remove("clicked");
    }
}

function RegisterDragScrollHandler() {
    const container = document.getElementById(META_CONTAINER_ID);

    // Set the scroll area start at the bottom
    container.scrollTop = container.scrollHeight;

    // From https://codepen.io/Gutto/pen/GBLPyN
    let startY;
    let startX;
    let scrollLeft;
    let scrollTop;
    let isDown;

    container.addEventListener('mousedown', event => {
        isDown = true;
        startY = event.pageY - container.offsetTop;
        startX = event.pageX - container.offsetLeft;
        scrollLeft = container.scrollLeft;
        scrollTop = container.scrollTop;
    });
    container.addEventListener('mouseup', () => {
        isDown = false;
    });
    container.addEventListener('mouseleave', () => {
        isDown = false;
    });
    container.addEventListener('mousemove', event => {
        if (isDown) {
            event.preventDefault();
            //Move vertically
            const y = event.pageY - container.offsetTop;
            const walkY = y - startY;
            container.scrollTop = scrollTop - walkY;

            //Move Horizontally
            const x = event.pageX - container.offsetLeft;
            const walkX = x - startX;
            container.scrollLeft = scrollLeft - walkX;
        }
    });
}

function MetaCanvasResized() {
    let canvas = document.getElementById(META_CANVAS_ID)
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    draw(META_CANVAS_ID, current_state_id, solver);
}

/**
 * Gather all possible moves for the given state
 * @param state the state to gather the moves for
 * @returns {Object[]} an array of objects containing move data
 */
function collectMoves(state) {
    return state.edges.map(edge => {
        const move = edge.slide_move;
        const id = edge.neighbor;
        const neighbor_state = get_state(solver, id);
        const distance = neighbor_state.distance_to_solution;
        const delta = distance - state.distance_to_solution;
        return {move: move, id: id, distance: distance, distance_delta: delta};
    }).filter(move => {
        // Hide our "fake" solution moves
        // TODO(Menno 24.04.2025) These fake moves should probably never be exported from the rust lib
        return move.distance > 0;
    });
}

/**
 * The user wants to preview a move
 * @param move the move to preview
 */
function previewMove(move) {
    // preview the actual move data
    gameBoard.preview(move.move);
}

/**
 * The user does no longer wants to see a move preview
 */
function cancelMovePreview() {
    gameBoard.cancelPreview();
}

/**
 * The user wants to do a move
 * @param move the move to execute
 */
function doMove(move) {
    gameBoard.cancelPreview()
    gameBoard.doMove(move.move, () => {
            setCurrentState(move.id);

            // The new state should now be applied.
            // If we are in auto-solve mode, we soon start the next move.
            if (is_auto_solve_enabled) {
                if (move.distance > 1) {
                    auto_solve_timer = setTimeout(gameMoves.doBestMove, 200);
                } else {
                    setAutoSolve(false);
                }
            }
        }
    );
}

function setCurrentState(id) {
    // TODO(Menno 23.12.2024) I think the current-state stuff should be moved to the rust lib.
    //  In general it would be good to reconsider the rust lib interface,
    //  I'm currently just hacking as I go to find out how wasm-bindgen works
    current_state_id = id;
    current_state = get_state(solver, current_state_id);

    draw(META_CANVAS_ID, current_state_id, solver);
    gameBoard.show(current_state.board);
    gameMoves.list(collectMoves(current_state));
}

const meta_canvas_observer = new ResizeObserver(MetaCanvasResized);
meta_canvas_observer.observe(document.getElementById(META_CANVAS_ID));

gameMoves.init(GAME_MOVES_DIV_ID, doMove, previewMove, cancelMovePreview);
gameBoard.init(GAME_CANVAS_ID);
RegisterDragScrollHandler();
RegisterControls();
solver = generate();
setCurrentState(get_start_id());
MetaCanvasResized();
