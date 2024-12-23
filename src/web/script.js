import init, {generate, draw, get_start_id, get_state} from "../../pkg/wiggers_graaf.js";

await init();

const META_CANVAS_ID = "meta-canvas";
const GAME_CANVAS_ID = "game-canvas";
const GAME_MOVES_DIV_ID = "game-moves"
let board_ctx = document.getElementById(GAME_CANVAS_ID).getContext("2d");
let game_moves_div = document.getElementById(GAME_MOVES_DIV_ID)
let solver;
let current_state;

function RegisterDragScrollHandler() {
    // From https://codepen.io/Gutto/pen/GBLPyN
    const container = document.querySelector('#meta-container');

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

    draw(META_CANVAS_ID, solver);
}

function DrawBoard(board) {
    function getColor(size) {
        if (size.x === 1 && size.y === 1) {
            return "rgba(75,123,255,0.6)"
        } else if (size.x === 1 && size.y === 2) {
            return "rgba(117,75,255,0.6)"
        } else if (size.x === 2 && size.y === 1) {
            return "rgba(75,213,255,0.6)"
        } else if (size.x === 2 && size.y === 2) {
            return "rgba(255,207,75,0.6)"
        } else {
            console.error("Unknown Piece size: (x: " + size.x + ", y: " + size.y + ")")
            return "#000"
        }
    }

    // Find the smallest scale, x or Y, to fit the board inside the canvas
    const rendering_scale = Math.min((board_ctx.canvas.width - board.size.x) / board.size.x, (board_ctx.canvas.height - board.size.y) / board.size.y);
    board_ctx.clearRect(0, 0, board_ctx.canvas.width, board_ctx.canvas.height);
    board.pieces.forEach(piece => {
        const pos = piece.position
        const size = piece.size
        board_ctx.beginPath();
        board_ctx.fillStyle = getColor(size);

        board_ctx.roundRect(pos.x + (pos.x * rendering_scale), board_ctx.canvas.height - (pos.y + (pos.y * rendering_scale)), size.x * rendering_scale, -size.y * rendering_scale, 0.1 * rendering_scale);

        board_ctx.fill();

    });
}

function ListNeighbors(neighbor_ids) {
    // Clear the div contents
    game_moves_div.innerHTML = "";

    // Map the neighbor ids into a list of divs
    // TODO(Menno 22.12.2024) I tried using .map() to clean up this code, but it gave me "invalid bigint syntax" errors
    let neighbors = [];
    neighbor_ids.forEach(neighbor_id => {
        const state = get_state(solver, neighbor_id);
        const distance = state.distance_to_solution;
        const delta =  distance - current_state.distance_to_solution;
        neighbors.push({id: neighbor_id, distance: distance, distance_delta: delta});
    })

    neighbors.sort((a, b) => {
        if (a.distance < b.distance) {
            return -1;
        } else if (a.distance > b.distance) {
            return 1;
        } else {
            return 0;
        }
    });

    neighbors.forEach(neighbor => {
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
        indicator_div.style.backgroundColor = getColor(neighbor.distance_delta);

        move_div.append(indicator_div)
        move_div.append(String(neighbor.distance) + " steps to solution");
        move_div.classList.add("game-move")
        move_div.onclick = event => {
            SetCurrentState(neighbor.id);
        };
        game_moves_div.append(move_div);
    })
}

function GameCanvasResized() {
    let canvas = document.getElementById(GAME_CANVAS_ID)
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    DrawBoard(current_state.board)
}

function SetCurrentState(id) {
    current_state = get_state(solver, id);

    DrawBoard(current_state.board)
    ListNeighbors(current_state.neighbors)
}

const meta_canvas_observer = new ResizeObserver(MetaCanvasResized);
meta_canvas_observer.observe(document.getElementById(META_CANVAS_ID));
const game_canvas_observer = new ResizeObserver(GameCanvasResized);
game_canvas_observer.observe(document.getElementById(GAME_CANVAS_ID));


RegisterDragScrollHandler();
solver = generate();
SetCurrentState(get_start_id());
MetaCanvasResized();
GameCanvasResized();
