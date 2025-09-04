// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

import LazyAnimation from "./lazy-animation.js"

const LAZY_ANIMATION_CANVAS_ID = "lazy-animation-canvas"
const lazyAnimation = new LazyAnimation(LAZY_ANIMATION_CANVAS_ID);
// Wait until the animation has started, otherwise it might not be shown until after the loading has completed
await lazyAnimation.started;

import init, {WiggersGraaf} from "../pkg/wiggers_graaf.js";

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

init().then(() => {
    registerSpector();
    wiggers_graaf = new WiggersGraaf(META_CANVAS_ID, GAME_CANVAS_ID, GAME_MOVES_DIV_ID, GAME_CONTROL_RESTART_ID, GAME_CONTROL_SOLVE_ID);
    lazyAnimation.cancel();
});
