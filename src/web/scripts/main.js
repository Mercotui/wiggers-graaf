// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

import LazyAnimation from "./lazy-animation.js"

const LAZY_ANIMATION_CANVAS_ID = "lazy-animation-canvas"
const lazyAnimation = new LazyAnimation(LAZY_ANIMATION_CANVAS_ID);
// Wait until the animation has started, otherwise it might not be shown until after the loading has completed
await lazyAnimation.started;

import init, {WiggersGraaf} from "../pkg/wiggers_graaf.js";
import * as gameMoves from "./game-moves.js";

const GAME_CONTROL_RESTART_ID = "game-control-restart";
const GAME_CONTROL_SOLVE_ID = "game-control-solve";
const META_CANVAS_ID = "meta-canvas";
const GAME_CANVAS_ID = "game-canvas";
const GAME_MOVES_DIV_ID = "game-moves"

let wiggers_graaf;
let current_state;
let current_state_id;
let auto_solve_toggle_div = document.getElementById(GAME_CONTROL_SOLVE_ID);
let is_auto_solve_enabled = false;
let auto_solve_timer;

function registerSpector() {
    if (typeof SPECTOR !== 'undefined') {
        const spector = new SPECTOR.Spector();
        spector.displayUI();
    }
}

function registerControls() {
    let restart_button = document.getElementById(GAME_CONTROL_RESTART_ID)
    restart_button.onclick = event => {
        restart_button.classList.add("clicked");
        // gameBoard.cancelMove();
        setAutoSolve(false);
        setCurrentState(WiggersGraaf.get_start_id());
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

function registerMetaControls() {
    const canvas = document.getElementById(META_CANVAS_ID);
    let previousY;
    let previousX;
    let isDown;

    canvas.addEventListener("wheel", event => {
        if (event.deltaMode !== 0) {
            console.error("wheel event with unexpected deltaMode " + event.deltaMode);
        }
        // invert scale to the behavior of "dragging down increases scale"
        wiggers_graaf.accumulate_zoom(-event.deltaY, event.offsetX, event.offsetY);
    });
    canvas.addEventListener("mousedown", event => {
        isDown = true;
        previousX = event.x;
        previousY = event.y;
    });
    // The mouse up and move events are listened for on the window not the canvas, this allows for larger move gestures
    window.addEventListener("mouseup", () => {
        isDown = false;
    });
    window.addEventListener("mousemove", event => {
        if (!isDown) {
            return;
        }
        event.preventDefault();
        const deltaX = event.x - previousX;
        // Invert browser Y direction to match OpenGL Y direction
        const deltaY = -(event.y - previousY);
        wiggers_graaf.accumulate_translation(deltaX, deltaY);
        previousX = event.x;
        previousY = event.y;
    });
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
        const neighbor_state = wiggers_graaf.get_state(id);
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
    // gameBoard.preview(move.move);
}

/**
 * The user no longer wants to see a move preview
 */
function cancelMovePreview() {
    // gameBoard.cancelPreview();
}

/**
 * The user wants to do a move
 * @param move the move to execute
 */
function doMove(move) {
    // gameBoard.cancelPreview()
    // gameBoard.doMove(move.move, () => {
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
    // });
}

function setCurrentState(id) {
    // TODO(Menno 23.12.2024) I think the current-state stuff should be moved to the rust lib.
    //  In general it would be good to reconsider the rust lib interface,
    //  I'm currently just hacking as I go to find out how wasm-bindgen works
    current_state_id = id;
    current_state = wiggers_graaf.get_state(current_state_id);

    wiggers_graaf.set_active_state(current_state_id)
    gameMoves.list(collectMoves(current_state));
}

init().then(() => {
    wiggers_graaf = new WiggersGraaf(META_CANVAS_ID, GAME_CANVAS_ID);
    gameMoves.init(GAME_MOVES_DIV_ID, doMove, previewMove, cancelMovePreview);
    registerMetaControls();
    registerControls();
    registerSpector();
    setCurrentState(WiggersGraaf.get_start_id());
    lazyAnimation.cancel();
});
