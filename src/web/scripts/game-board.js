let ctx;
let canvas;
let canvas_observer;
let current_board;

export function init(game_canvas_id) {
    canvas = document.getElementById(game_canvas_id);
    ctx = canvas.getContext("2d");

    canvas_observer = new ResizeObserver(canvasResized);
    canvas_observer.observe(canvas);
    canvasResized();
}

function canvasResized() {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;

    if (current_board !== undefined) {
        draw(current_board)
    }
}

export function draw(board) {
    // cache the board in case we need to redraw it (i.e. after canvas resize)
    current_board = board;

    const layout = calculateLayout(board);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    board.pieces.forEach(piece => drawPiece(layout, piece));
}

function calculateLayout(board) {
    // Find the smallest scale, x or Y, to fit the board inside the canvas
    const rendering_scale = Math.min((canvas.width - board.size.x) / board.size.x, (canvas.height - board.size.y) / board.size.y);
    const offset_x = 0.5 * (canvas.width - (board.size.x + (rendering_scale * board.size.x)));
    const offset_y = 0.5 * (canvas.height - (board.size.y + (rendering_scale * board.size.y)));

    return {scale: rendering_scale, offset: {x: offset_x, y: offset_y}};
}

function drawPiece(layout, piece) {
    const pos = piece.position
    const size = piece.size
    ctx.beginPath();
    ctx.fillStyle = getColor(size);

    // Start rendering from xy offset, then each piece gets an additional pixel offset to create a gap between each other.
    const x = layout.offset.x + pos.x + (pos.x * layout.scale);
    const y = canvas.height - (layout.offset.y + pos.y + (pos.y * layout.scale));
    const width = size.x * layout.scale;
    const height = -size.y * layout.scale;
    const corner_radius = 0.1 * layout.scale;

    ctx.roundRect(x, y, width, height, corner_radius);

    ctx.fill();
}

function getColor(size) {
    // Color palette from https://mycolor.space/?hex=%23754BFF&sub=1
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
