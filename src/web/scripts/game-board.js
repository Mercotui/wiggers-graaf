import {SlideDirection} from "../pkg/wiggers_graaf.js";

let ctx;
let canvas;
let canvasHasResized = true;
let drawIsScheduled = false;
let canvasObserver;
let board;

export function init(game_canvas_id) {
    canvas = document.getElementById(game_canvas_id);
    ctx = canvas.getContext("2d");

    canvasObserver = new ResizeObserver(() => {
        canvasHasResized = true;
        scheduleDraw();
    });
    canvasObserver.observe(canvas);
}

export function preview(move) {
    board.pieces.forEach(piece => {
        if (coordinates2dEq(piece.position, move.start)) {
            switch (move.direction) {
                case SlideDirection.Up: {
                    piece.visualOffset.y = move.distance;
                    break;
                }
                case SlideDirection.Down: {
                    piece.visualOffset.y = -move.distance;
                    break;
                }
                case SlideDirection.Left: {
                    piece.visualOffset.x = -move.distance;
                    break;
                }
                case SlideDirection.Right: {
                    piece.visualOffset.x = move.distance;
                    break;
                }
            }
        }
    });
    scheduleDraw();
}

export function cancelPreview() {
    board.pieces.forEach(piece => {
        piece.visualOffset.x = 0.0;
        piece.visualOffset.y = 0.0;
    });
    scheduleDraw();
}

export function show(new_board) {
    // Cache the board in case we need to redraw it (i.e. after canvas resize).
    // We make a deep copy with some additional attributes, to help in rendering.
    board = {
        size: {x: new_board.size.x, y: new_board.size.y},
        pieces: new_board.pieces.map(piece => {
                return {
                    size: {x: piece.size.x, y: piece.size.y},
                    position: {x: piece.position.x, y: piece.position.y},
                    // The visual position offset can be used for animations or user interactions
                    visualOffset: {x: 0, y: 0},
                }
            }
        )
    }
    scheduleDraw();
}

function coordinates2dEq(a, b) {
    return a.x === b.x && a.y === b.y;
}

function scheduleDraw() {
    if (!drawIsScheduled) {
        drawIsScheduled = true;
        requestAnimationFrame(draw);
    }
}

function draw() {
    drawIsScheduled = false;
    if (canvasHasResized) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        canvasHasResized = false;
    }
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (board !== undefined) {
        const layout = calculateLayout(board);
        board.pieces.forEach(piece => drawPiece(layout, piece));
    }
}

function calculateLayout(board) {
    // Find the smallest scale, x or Y, to fit the board inside the canvas
    const rendering_scale = Math.max(0.0, Math.min((canvas.width - board.size.x) / board.size.x, (canvas.height - board.size.y) / board.size.y));
    const offset_x = 0.5 * (canvas.width - (board.size.x + (rendering_scale * board.size.x)));
    const offset_y = 0.5 * (canvas.height - (board.size.y + (rendering_scale * board.size.y)));

    return {scale: rendering_scale, offset: {x: offset_x, y: offset_y}};
}

function drawPiece(layout, piece) {
    const pos = {x: piece.position.x + piece.visualOffset.x, y: piece.position.y + piece.visualOffset.y}
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
