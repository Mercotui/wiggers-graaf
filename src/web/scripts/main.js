// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

import LazyAnimation from "./lazy-animation.js"

const LAZY_ANIMATION_CANVAS_ID = "lazy-animation-canvas"
const lazyAnimation = new LazyAnimation(LAZY_ANIMATION_CANVAS_ID);
// Wait until the animation has started, otherwise it might not be shown until after the loading has completed
await lazyAnimation.started;

import init, {WiggersGraaf} from "../pkg/wiggers_graaf.js";
import * as gameMoves from "./game-moves.js";
import {setAutoSolve, updateList} from "./game-moves.js";

const GAME_CONTROL_RESTART_ID = "game-control-restart";
const GAME_CONTROL_SOLVE_ID = "game-control-solve";
const META_CANVAS_ID = "meta-canvas";
const GAME_CANVAS_ID = "game-canvas";
const GAME_MOVES_DIV_ID = "game-moves"

let wiggers_graaf;

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
        gameMoves.setAutoSolve(false);
        wiggers_graaf.restart();
        setTimeout(() => {
            restart_button.classList.remove("clicked")
        }, 200)
    };
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

init().then(() => {
    registerMetaControls();
    registerControls();
    registerSpector();

    gameMoves.init(GAME_MOVES_DIV_ID, GAME_CONTROL_SOLVE_ID, move => {
        wiggers_graaf.do_move(move)
    }, move => {
        wiggers_graaf.preview_move(move)
    });

    wiggers_graaf = new WiggersGraaf(META_CANVAS_ID, GAME_CANVAS_ID, gameMoves.updateList, gameMoves.highlight);

    lazyAnimation.cancel();
});
