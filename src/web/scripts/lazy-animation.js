// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

export default class LazyAnimation {
    #canvas;
    #worker;
    started;

    constructor(canvas_id) {
        this.#worker = new Worker("./scripts/lazy-animation-worker.js");
        this.#canvas = document.getElementById(canvas_id);
        const offscreen_canvas = this.#canvas.transferControlToOffscreen();
        this.#worker.postMessage(offscreen_canvas, [offscreen_canvas]);

        let started_resolve;
        this.started = new Promise((resolve, reject) => {
            started_resolve = resolve;
        });

        this.#worker.onmessage = (message) => {
            if (message.data === true) {
                started_resolve();
            } else if (message.data === null) {
                this.#canvas.remove();
            }
        }

    }

    cancel () {
        this.#worker.postMessage(null);
    }
}
