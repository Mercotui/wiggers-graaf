// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

import {SlideDirection} from "../pkg/wiggers_graaf.js";
import {Animation, Delay, Loop, Sequence, ease} from "./animation.js"

let ctx;
let canvas;
let canvasHasResized = true;
let drawIsScheduled = false;
let canvasObserver;
let board;
let layout;
let horizontal_axis;
let vertical_axis;

// Init animations with dummy values
let previewAnimation = new Delay({delay: 1});
let moveAnimation = new Delay({delay: 1});

const AXIS_PADDING = 4;
const HORIZONTAL_AXIS_ID = "game-axis-horizontal"
const VERTICAL_AXIS_ID = "game-axis-vertical"

/**
 * Initialize the GameBoard module, used to draw and animate a game board.
 * @param game_canvas_id the ID of the canvas to draw the game board on
 */
export function init(game_canvas_id) {
    // Find axes
    horizontal_axis = document.getElementById(HORIZONTAL_AXIS_ID)
    vertical_axis = document.getElementById(VERTICAL_AXIS_ID)

    // Setup canvas
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
 * Cancel the current move animation (if any), and reset the board. This prevents the move from being applied.
 */
export function cancelMove() {
    moveAnimation.cancel();
    resetBoard();
}

/**
 * Cancel the current preview animation (if any) and reset the board
 */
export function cancelPreview() {
    previewAnimation.cancel();
    resetBoard();
}

function resetBoard() {
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
    moveAnimation = new Animation({
        duration: 150,
        range: [0.0, translation.distance],
        easingFunc: ease.inOutQuad,
        onDoneFunc: onDoneCb,
        onUpdateFunc: (value) => {
            piece.visualOffset[translation.axis] = value;
            draw();
        },
    });
    moveAnimation.start();
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
    updateLayout();
    scheduleDraw();
}

/**
 * Set the axes based on the layout
 */
function updateAxes() {
    vertical_axis.style.left = `${layout.axes_offset.vertical.x}px`
    vertical_axis.style.bottom = `${layout.axes_offset.vertical.y}px`
    horizontal_axis.style.left = `${layout.axes_offset.horizontal.x}px`
    horizontal_axis.style.bottom = `${layout.axes_offset.horizontal.y}px`

    // Clear the div contents
    vertical_axis.innerHTML = "";
    horizontal_axis.innerHTML = "";

    addAxisTick(vertical_axis);
    addAxisTick(horizontal_axis);
    for (let i = layout.board_size.height; i > 0; i--) {
        let label = addAxisLabel(vertical_axis, String(i));
        label.style.height = `${layout.scale}px`;
        addAxisTick(vertical_axis)
    }
    for (let i = 0; i < layout.board_size.width; i++) {
        // UTF16 code unit 65 is the letter A
        const label_text = String.fromCharCode(65 + i);
        let label = addAxisLabel(horizontal_axis, label_text);
        label.style.width = `${layout.scale}px`;
        addAxisTick(horizontal_axis);
    }
}

/**
 * Add a text label to a game axis
 * @param axis the axis to append the label to
 * @param text the text to display in the label
 * @return the new label div
 */
function addAxisLabel(axis, text) {
    let label_div = document.createElement("div");
    label_div.classList.add("label")
    label_div.append(text);
    axis.append(label_div);
    return label_div
}

/**
 * Add a divider tick to a game axis
 * @param axis the axis to append the tick too
 */
function addAxisTick(axis) {
    let tick_div = document.createElement("div");
    tick_div.classList.add("tick")
    axis.append(tick_div);
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
    if (board === undefined) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        return;
    }

    if (canvasHasResized) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
        updateLayout();
        canvasHasResized = false;
    } else {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
    }
    board.pieces.forEach(piece => drawPiece(piece));
}

function updateLayout() {
    const new_layout = calculateLayout();
    if (!layoutsEqual(layout, new_layout)) {
        layout = new_layout;
        updateAxes();
    }
}

/**
 * Calculate a draw layout for a given board
 * @returns object containing a scale of pixels per game unit, and offset in pixels to center the board inside the canvas
 */
function calculateLayout() {
    // Find space required by the axes
    const axes_x = parseFloat(window.getComputedStyle(vertical_axis).width) + AXIS_PADDING;
    const axes_y = parseFloat(window.getComputedStyle(horizontal_axis).height) + AXIS_PADDING;

    // Add pixel gaps between each element.
    const gaps_x = board.size.x - 1;
    const gaps_y = board.size.y - 1;

    // Find the smallest scale, x or Y, to fit the board inside the canvas
    // Note, double the vertical axis for horizontal symmetry.
    // Also floor the scale so that fractional pixels are avoided.
    const scale_x = (canvas.width - (gaps_x + (axes_x * 2))) / board.size.x;
    const scale_y = (canvas.height - (gaps_y + axes_y)) / board.size.y;
    const rendering_scale = Math.floor(Math.max(0.0, Math.min(scale_x, scale_y)));

    // Calculate the offset of the content, the y-axis is not symmetrical
    const content_offset_x = 0.5 * (canvas.width - (gaps_x + (rendering_scale * board.size.x)));
    const content_offset_y = 0.5 * ((canvas.height) - (gaps_y + (rendering_scale * board.size.y) - axes_y));

    // Calculate the offset of the axes, subtract the 1 pixel tick that should start before the content
    const horizontal_axis_offset_x = content_offset_x - 1;
    const horizontal_axis_offset_y = content_offset_y - axes_y;
    const vertical_axis_offset_x = content_offset_x - axes_x;
    const vertical_axis_offset_y = content_offset_y - 1;

    return {
        scale: rendering_scale, offset: {x: content_offset_x, y: content_offset_y}, axes_offset: {
            horizontal: {
                x: horizontal_axis_offset_x, y: horizontal_axis_offset_y
            }, vertical: {
                x: vertical_axis_offset_x, y: vertical_axis_offset_y
            }
        }, board_size: {width: board.size.x, height: board.size.y}
    };
}

/**
 * Check deep equaliy of two layouts
 * @param a layout A
 * @param b layout B
 * @returns {boolean} true if a and b are deeply equal
 */
function layoutsEqual(a, b) {
    // Check for nulls
    if (!a || !b) {
        return false;
    }
    // Then check for deep equality, split on multiple lines because my auto formatter doesn't care about line length.
    const scale_equals = a.scale === b.scale;
    const offset_equals = a.offset.x === b.offset.x && a.offset.y === b.offset.y;
    const board_size_equals = a.board_size.width === b.board_size.width && a.board_size.height === b.board_size.height;
    const horizontal_axes_offset_equals = a.axes_offset.horizontal.x === b.axes_offset.horizontal.x && a.axes_offset.horizontal.y === b.axes_offset.horizontal.y;
    const vertical_axes_offset_equals = a.axes_offset.vertical.x === b.axes_offset.vertical.x && a.axes_offset.vertical.y === b.axes_offset.vertical.y;
    return scale_equals && offset_equals && board_size_equals && horizontal_axes_offset_equals && vertical_axes_offset_equals;
}

/**
 * Draw a single game piece to the canvas
 * @param piece the piece to draw
 */
function drawPiece(piece) {
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
