import {SlideDirection} from "../pkg/wiggers_graaf.js";
import {Animation, Delay, Loop, Sequence, ease} from "./animation.js"

let ctx;
let canvas;
let canvasHasResized = true;
let drawIsScheduled = false;
let canvasObserver;
let board;
let previewAnimation = new Delay({delay: 1}); // Init with a dummy value

/**
 * Initialize the GameBoard module, used to draw and animate a game board.
 * @param game_canvas_id the ID of the canvas to draw the game board on
 */
export function init(game_canvas_id) {
    canvas = document.getElementById(game_canvas_id);
    ctx = canvas.getContext("2d");

    canvasObserver = new ResizeObserver(() => {
        canvasHasResized = true;
        scheduleDraw();
    });
    canvasObserver.observe(canvas);
}

/**
 * Start a move preview animation
 * @param move the move to preview
 */
export function preview(move) {
    let translation = getAxisAndDistance(move);
    let piece = board.pieces.find(piece => coordinates2dEq(piece.position, move.start));
    piece.color = getColor(piece.size, true);

    previewAnimation = new Loop({
        animation: new Sequence({
            animations: [new Delay({delay: 1000}), new Animation({
                duration: 150, range: [0.0, translation.distance], easingFunc: ease.inOutQuad,
            }), new Delay({delay: 1000}), new Animation({
                duration: 150, range: [translation.distance, 0.0], easingFunc: ease.inOutQuad,
            }),]
        }), onUpdateFunc: (value) => {
            piece.visualOffset[translation.axis] = value;
            draw();
        }
    });
    previewAnimation.start();

    // Draw once to show the highlight color
    scheduleDraw();
}

/**
 * Cancel the current preview animation (if any) and reset the board
 */
export function cancelPreview() {
    previewAnimation.cancel();
    board.pieces.forEach(piece => {
        piece.color = getColor(piece.size);
        piece.visualOffset.x = 0.0;
        piece.visualOffset.y = 0.0;
    });
    scheduleDraw();
}

/**
 * Execute a move, call onDoneCb when move has been completed
 * @param move the move to show
 * @param onDoneCb the callback to call when the move has completed
 */
export function doMove(move, onDoneCb) {
    let piece = board.pieces.find(piece => coordinates2dEq(piece.position, move.start));
    let translation = getAxisAndDistance(move);
    new Animation({
        duration: 150,
        range: [0.0, translation.distance],
        easingFunc: ease.inOutQuad,
        onDoneFunc: onDoneCb,
        onUpdateFunc: (value) => {
            piece.visualOffset[translation.axis] = value;
            draw();
        },
    }).start();
}

/**
 * Show a new board state
 * @param new_board the new board state to show
 */
export function show(new_board) {
    previewAnimation.cancel();

    // Cache the board in case we need to redraw it (i.e. after canvas resize).
    // We make a deep copy with some additional attributes, to help in rendering.
    board = {
        size: {x: new_board.size.x, y: new_board.size.y}, pieces: new_board.pieces.map(piece => {
            return {
                size: {x: piece.size.x, y: piece.size.y},
                position: {x: piece.position.x, y: piece.position.y},
                color: getColor(piece.size),
                visualOffset: {x: 0, y: 0}, // The visual position offset can be used for animations or user interactions
            }
        })
    }
    scheduleDraw();
}

/**
 * Determine the axis and the signed distance to travel on said axis for a given move
 * @param move the move to analyze
 * @return an object containing an axis label 'x' or 'y', and the signed-distance to travel on said axis.
 */
function getAxisAndDistance(move) {
    switch (move.direction) {
        case SlideDirection.Up: {
            return {axis: 'y', distance: move.distance};
        }
        case SlideDirection.Down: {
            return {axis: 'y', distance: -move.distance};
        }
        case SlideDirection.Left: {
            return {axis: 'x', distance: -move.distance};
        }
        case SlideDirection.Right: {
            return {axis: 'x', distance: move.distance};
        }
    }
}

/**
 * Compare equality of two 2D coordinates
 * @param a coordinates A
 * @param b coordinates B
 * @returns true if A and B are equal, otherwise false
 */
function coordinates2dEq(a, b) {
    return a.x === b.x && a.y === b.y;
}

/**
 * Schedule a board draw
 */
function scheduleDraw() {
    if (!drawIsScheduled) {
        drawIsScheduled = true;
        requestAnimationFrame(draw);
    }
}

/**
 * Do immediate board draw. Note, please prefer scheduleDraw to prevent wasteful draws.
 */
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

/**
 * Calculate a draw layout for a given board
 * @param board the board to analyze
 * @returns object containing a scale of pixels per game unit, and offset in pixels to center the board inside the canvas
 */
function calculateLayout(board) {
    // Find the smallest scale, x or Y, to fit the board inside the canvas
    const rendering_scale = Math.max(0.0, Math.min((canvas.width - board.size.x) / board.size.x, (canvas.height - board.size.y) / board.size.y));
    const offset_x = 0.5 * (canvas.width - (board.size.x + (rendering_scale * board.size.x)));
    const offset_y = 0.5 * (canvas.height - (board.size.y + (rendering_scale * board.size.y)));

    return {scale: rendering_scale, offset: {x: offset_x, y: offset_y}};
}

/**
 * Draw a single game piece to the canvas
 * @param layout the layout to apply
 * @param piece the piece to draw
 */
function drawPiece(layout, piece) {
    const pos = {x: piece.position.x + piece.visualOffset.x, y: piece.position.y + piece.visualOffset.y}
    const size = piece.size
    ctx.beginPath();
    ctx.fillStyle = piece.color;

    // Start rendering from xy offset, then each piece gets an additional pixel offset to create a gap between each other.
    const x = layout.offset.x + pos.x + (pos.x * layout.scale);
    const y = canvas.height - (layout.offset.y + pos.y + (pos.y * layout.scale));
    const width = size.x * layout.scale;
    const height = -size.y * layout.scale;
    const corner_radius = 0.1 * layout.scale;

    ctx.roundRect(x, y, width, height, corner_radius);

    ctx.fill();
}

/**
 * The lookup a piece size in the color scheme
 * @param size the size of the piece
 * @param highlight whether or not the piece is currently highlighted
 * @returns the color of the piece
 */
function getColor(size, highlight = false) {
    // Color palette from https://mycolor.space/?hex=%23754BFF&sub=1
    if (highlight) {
        if (size.x === 1 && size.y === 1) {
            return "rgba(75,123,255,1)"
        } else if (size.x === 1 && size.y === 2) {
            return "rgba(117,75,255,1)"
        } else if (size.x === 2 && size.y === 1) {
            return "rgba(75,213,255,1)"
        } else if (size.x === 2 && size.y === 2) {
            return "rgba(255,207,75,1)"
        }
    } else {
        if (size.x === 1 && size.y === 1) {
            return "rgba(75,123,255,0.8)"
        } else if (size.x === 1 && size.y === 2) {
            return "rgba(117,75,255,0.8)"
        } else if (size.x === 2 && size.y === 1) {
            return "rgba(75,213,255,0.8)"
        } else if (size.x === 2 && size.y === 2) {
            return "rgba(255,207,75,0.8)"
        }
    }
    console.error("Unknown Piece size: (x: " + size.x + ", y: " + size.y + ")")
    return "#000"

}
