// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

import {SlideDirection, MoveEffectiveness} from "../pkg/wiggers_graaf.js";

let moves_div;
let make_move_cb;
let preview_move_cb;
let best_move;
let best_move_div;
let is_auto_solve_enabled = false;
let auto_solve_timer = undefined;
let auto_solve_toggle_div = undefined;


/**
 * Initialize the game-moves module
 * @param div_id a div where game moves can be listed using list(moves)
 * @param auto_solve_toggle_id a toggle that controls the auto solve
 * @param move_cb callback to call when the user wants to make a move
 * @param preview_cb callback to call when the user wants to preview a move
 */
export function init(div_id, auto_solve_toggle_id, move_cb, preview_cb) {
    make_move_cb = move_cb;
    preview_move_cb = preview_cb;
    moves_div = document.getElementById(div_id);

    auto_solve_toggle_div = document.getElementById(auto_solve_toggle_id);
    auto_solve_toggle_div.onclick = event => {
        // Toggle auto-solve
        setAutoSolve(!is_auto_solve_enabled);
    };
}

/**
 * Show the next possible moves to the user as a list
 * @param moves the moves to show
 */
export function updateList(moves) {
    // Clear the div contents
    moves_div.innerHTML = "";

    // Map the neighbor ids into a list of divs
    best_move = moves.at(0);

    moves.forEach(move => {
        let move_div = document.createElement("div");
        let indicator_div = document.createElement("div");

        if (move === best_move) {
            best_move_div = move_div;
        }

        indicator_div.classList.add("game-move-indicator")
        indicator_div.style.backgroundColor = getColor(move.effectiveness);

        move_div.append(indicator_div)
        move_div.append(`${convertMoveToString(move.slide_move)}  ${move.resulting_distance} steps left`);
        move_div.classList.add("game-move")
        move_div.onclick = () => {
            move_div.classList.add("clicked");
            make_move_cb(move);
        };
        move_div.onmouseenter = () => {
            preview_move_cb(move);
            setHighlight(move_div, true);
        };
        move_div.onmouseleave = () => {
            setHighlight(move_div, false);
            preview_move_cb(undefined);
        }
        moves_div.append(move_div);
    })

    // If we are in auto-solve mode, we soon start the next move.
    if (is_auto_solve_enabled) {
        // Keep going until we run out of good moves to make
        if (best_move.effectiveness === MoveEffectiveness.Positive) {
            clearTimeout(auto_solve_timer);
            auto_solve_timer = setTimeout(doBestMove, 200);
        } else {
            setAutoSolve(false);
        }
    }

}

export function highlight(_criteria) {
    // TODO(Menno 29.06.2025) Match the criterion to the moves in the moves list
}

export function setAutoSolve(enable) {
    if (is_auto_solve_enabled === enable) {
        // do nothing
        return;
    }

    is_auto_solve_enabled = enable;
    if (is_auto_solve_enabled) {
        auto_solve_toggle_div.classList.add("clicked");
        // Start chain of moves
        doBestMove();
    } else {
        clearTimeout(auto_solve_timer);
        auto_solve_toggle_div.classList.remove("clicked");
    }
}

function doBestMove() {
    if (best_move !== undefined) {
        make_move_cb(best_move);
        setHighlight(best_move_div, true)
    }
}

function getMoveEndpoint(move) {
    let move_end = move.start;
    switch (move.direction) {
        case SlideDirection.Up: {
            move_end.y += move.distance;
            break;
        }
        case SlideDirection.Down: {
            move_end.y -= move.distance;
            break;
        }
        case SlideDirection.Left: {
            move_end.x -= move.distance;
            break;
        }
        case SlideDirection.Right: {
            move_end.x += move.distance;
            break;
        }
    }
    return move_end;
}

function convertMoveToString(move) {
    // UTF16 code unit 65 is the letter A
    const start_x = String.fromCharCode(65 + move.start.x);
    const start_y = String(move.start.y + 1);
    const endpoint = getMoveEndpoint(move);
    const end_x = String.fromCharCode(65 + endpoint.x);
    const end_y = String(endpoint.y + 1);
    return `${start_x}${start_y}⮕${end_x}${end_y}`
}

function setHighlight(move_div, enable) {
    if (enable) {
        move_div.classList.add("highlight")
    } else {
        move_div.classList.remove("highlight")
    }
}

/**
 * Get the move indicator color for a given delta-distance
 * @param effectiveness How the move impacts resulting distance to the solution
 * @returns {string} The move-indicator color
 */
function getColor(effectiveness) {
    switch (effectiveness) {
        case MoveEffectiveness.Positive:
            return "#009d77"
        case MoveEffectiveness.Neutral:
            return "#4B7BFF"
        case MoveEffectiveness.Negative:
            return "#ff443a"
    }
}
