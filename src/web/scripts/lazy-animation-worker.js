// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

// Vertex shader sourcecode
const VERTEX_SHADER_SOURCE = `
attribute vec4 position;
void main() {
    gl_Position = position;
}
`
// Fragment shader sourcecode
const FRAGMENT_SHADER_SOURCE = `
precision mediump float;
uniform float time;
void main() {
    float offset = gl_FragCoord.x - gl_FragCoord.y;
    float color = cos((time - offset) / 200.0) - 0.5;
    gl_FragColor = vec4(color);
}
`;

// Buffer containing vertices of a quad ABCD
const ARRAY_BUFFER_DATA = new Float32Array([
    -1.0, -1.0, 0.0, 1.0,
    1.0, -1.0, 0.0, 1.0,
    1.0, 1.0, 0.0, 1.0,
    -1.0, 1.0, 0.0, 1.0
]);

// Buffer containing elements of two triangles ABC and ACD forming quad ABCD
const ELEMENT_BUFFER_DATA = new Uint8Array([
    0, 1, 2,
    0, 2, 3,
]);

function compileShader(gl, source, type) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const info = gl.getShaderInfoLog(shader);
        throw `Could not compile WebGL program. \n\n${info}`;
    }
    return shader;
}

function createProgram(gl, vertexShaderSource, fragmentShaderSource) {
    const vertexShader = compileShader(gl, vertexShaderSource, gl.VERTEX_SHADER);
    const fragmentShader = compileShader(gl, fragmentShaderSource, gl.FRAGMENT_SHADER);

    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);

    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        const info = gl.getProgramInfoLog(program);
        throw `Could not compile WebGL program. \n\n${info}`;
    }

    return program;
}

function setupQuadVao(gl, program) {
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    const array_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, array_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, ARRAY_BUFFER_DATA, gl.STATIC_DRAW);

    const element_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, element_buffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, ELEMENT_BUFFER_DATA, gl.STATIC_DRAW);

    const positionLocation = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 4, gl.FLOAT, false, 0, 0);

    return vao;
}

class LazyAnimation {
    #gl;
    #program;
    #timeLocation;
    #vao;
    #frameRequestId;

    constructor(canvas) {
        this.#gl = canvas.getContext("webgl2");
        this.#program = createProgram(this.#gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        this.#timeLocation = this.#gl.getUniformLocation(this.#program, "time");
        this.#vao = setupQuadVao(this.#gl, this.#program);
        this.#scheduleDraw();
    }

    destroy() {
        cancelAnimationFrame(this.#frameRequestId);
        this.#gl = null;
        this.#vao = null;
        this.#program = null;
    }

    #scheduleDraw() {
        this.#frameRequestId = requestAnimationFrame((timestamp) => {
            this.#draw(timestamp)
        });
    }

    #draw(timestamp) {
        let gl = this.#gl;

        // Prepare state
        gl.useProgram(this.#program);
        gl.bindVertexArray(this.#vao);

        // Set Uniforms
        gl.uniform1f(this.#timeLocation, timestamp)

        // draw quad
        gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_BYTE, 0);

        // Reset state
        gl.bindVertexArray(null)
        gl.useProgram(null);

        this.#scheduleDraw();
    }
}

let lazyAnimation;
self.onmessage = (message) => {
    if (message.data instanceof OffscreenCanvas) {
        lazyAnimation = new LazyAnimation(message.data)
        self.postMessage(true);
    } else if (message.data === null) {
        // TODO(Menno 23.05.2025) We should not destroy the animation abruptly, instead it should fade out.
        lazyAnimation.destroy();
        lazyAnimation = null;
        self.postMessage(null);
    }
}
