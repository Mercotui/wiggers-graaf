let moves_div;
let make_move_cb;
let preview_move_cb;
let preview_move_cancel_cb;

/**
 * Initialize the game-moves module
 * @param div_id a div where game moves can be listed using list(moves)
 * @param move_cb callback to call when the user wants to make a move
 * @param preview_cb callback to call when the user wants to preview a move
 * @param preview_cancel_cb callback to call when the user no longer wants to see a move preview
 */
export function init(div_id, move_cb, preview_cb, preview_cancel_cb) {
    make_move_cb = move_cb;
    preview_move_cb = preview_cb;
    preview_move_cancel_cb = preview_cancel_cb;
    moves_div = document.getElementById(div_id)
}

/**
 * Show the provided moves to the user as a list
 * @param moves the moves to show
 */
export function list(moves) {
    // Clear the div contents
    moves_div.innerHTML = "";

    // Map the neighbor ids into a list of divs
    moves.sort((a, b) => {
        if (a.distance < b.distance) {
            return -1;
        } else if (a.distance > b.distance) {
            return 1;
        } else {
            return 0;
        }
    });

    moves.forEach(move => {
        let move_div = document.createElement("div");
        let indicator_div = document.createElement("div");

        indicator_div.classList.add("game-move-indicator")
        indicator_div.style.backgroundColor = getColor(move.distance_delta);

        move_div.append(indicator_div)
        move_div.append(String(move.distance) + " steps to solution");
        move_div.classList.add("game-move")
        move_div.onclick = event => {
            move_div.classList.add("clicked");
            make_move_cb(move);
        };
        move_div.onmouseenter = event => {
            preview_move_cb(move);
        };
        move_div.onmouseleave = event => {
            preview_move_cancel_cb();
        }
        moves_div.append(move_div);
    })
}

/**
 * Get the move indicator color for a given delta-distance
 * @param delta The delta of the distance to the solution
 * @returns {string} The move-indicator color
 */
function getColor(delta) {
    if (delta < 0) {
        return "#009d77"
    } else if (delta === 0) {
        return "#4B7BFF"
    } else {
        return "#ff443a"
    }
}
