import init, {generate, draw, get_start_id, get_state} from "../../pkg/wiggers_graaf.js";

await init();

const META_CANVAS_ID = "meta-canvas";
const GAME_CANVAS_ID = "game-canvas";
let board_ctx = document.getElementById(GAME_CANVAS_ID).getContext("2d");
let solver;
let current_state_id;
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

function ListNeighbors(neighbor_ids){
    // const neighbors = neighbors.map(neighbor_id => {
    //     const state = get_state(neighbor_id)
    //     {state.id, state.}
    // })
}

function GameCanvasResized() {
    let canvas = document.getElementById(GAME_CANVAS_ID)
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    DrawBoard(current_state.board)
}

const meta_canvas_observer = new ResizeObserver(MetaCanvasResized);
meta_canvas_observer.observe(document.getElementById(META_CANVAS_ID));
const game_canvas_observer = new ResizeObserver(GameCanvasResized);
game_canvas_observer.observe(document.getElementById(GAME_CANVAS_ID));


RegisterDragScrollHandler();
solver = generate();
current_state_id = get_start_id();
current_state = get_state(solver, current_state_id);

DrawBoard(current_state.board)
ListNeighbors()
MetaCanvasResized();
