// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

export default class LazyAnimation {
    #canvas;
    #worker;
    #sizeObserver;
    started;

    constructor(canvasId) {
        this.#worker = new Worker("./scripts/lazy-animation-worker.js");
        this.#canvas = document.getElementById(canvasId);

        const offscreenCanvas = this.#canvas.transferControlToOffscreen();
        this.#worker.postMessage({type: "init", canvas: offscreenCanvas}, [offscreenCanvas]);

        // TODO(Menno 09.08.2025) I suspect that this is needed, to prevent Firefox from out-off-memory killing my laptop.
        //  It seems that offscreen RequestAnimationFrame is not paused when the page is hidden,
        //  so my animation code will continue to create and submit canvas data. However as the main thread is suspended,
        //  that offscreen data is never collected and disposed off. I can not verify this at the moment.
        document.addEventListener("visibilitychange", () => {
            if (document.hidden) {
                this.#worker.postMessage({type: "pause", paused: true});
            } else {
                this.#worker.postMessage({type: "pause", paused: false});
            }
        });

        this.#sizeObserver = new ResizeObserver(entries => this.resizeCanvas(entries));
        this.#sizeObserver.observe(this.#canvas);

        let startedResolve;
        this.started = new Promise((resolve, reject) => {
            startedResolve = resolve;
        });
        this.#worker.onmessage = (message) => {
            if (message.data.type === "started") {
                startedResolve();
            } else if (message.data.type === "stopped") {
                this.#canvas.remove();
            } else {
                console.error(`message of unknown type ${message.data.type}`);
            }
        }
    }

    resizeCanvas(entries) {
        const entry = entries[0];
        let width;
        let height;
        if (entry.devicePixelContentBoxSize) {
            // pixel-perfect size for modern browsers
            width = entry.devicePixelContentBoxSize[0].inlineSize;
            height = entry.devicePixelContentBoxSize[0].blockSize;
        } else if (entry.contentBoxSize) {
            // best-effort fallback for Safari
            width = Math.round(entry.contentBoxSize[0].inlineSize * window.devicePixelRatio);
            height = Math.round(entry.contentBoxSize[0].blockSize * window.devicePixelRatio);
        }
        this.#worker.postMessage({
            type: "resize",
            canvasSize: {width, height},
            devicePixelRatio: window.devicePixelRatio,
        });
    }

    cancel() {
        this.#worker.postMessage({type: "cancel"});
    }
}
