let moves_div;
let make_move_cb;

/**
 * Initialize the game-moves module
 * @param div_id a div where game moves can be listed using list(moves)
 * @param move_cb callback to call when the user wants to make a move
 */
export function init(div_id, move_cb) {
    make_move_cb = move_cb;
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
        function getColor(delta) {
            if (delta < 0) {
                return "#009d77"
            } else if (delta === 0) {
                return "#4B7BFF"
            } else {
                return "#ff443a"
            }
        }

        let move_div = document.createElement("div");
        let indicator_div = document.createElement("div");

        indicator_div.classList.add("game-move-indicator")
        indicator_div.style.backgroundColor = getColor(move.distance_delta);

        move_div.append(indicator_div)
        move_div.append(String(move.distance) + " steps to solution");
        move_div.classList.add("game-move")
        move_div.onclick = event => {
            move_div.classList.add("clicked");

            setTimeout(() => {
                make_move_cb(move.id);
            }, 200);
        };
        moves_div.append(move_div);
    })
}
