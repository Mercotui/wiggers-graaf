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
uniform vec2 size;
uniform float scale;

const float BLUR_RADIUS = 16.0;

// Apply rounded rectangle SDF
float vignette(vec2 frag_coord) {
    float radius = BLUR_RADIUS / scale;
    vec2 half_rect_size = (size - vec2(BLUR_RADIUS)) * 0.5;
    vec2 rect_pos = size * 0.5;

    vec2 d = abs(gl_FragCoord.xy - rect_pos) - half_rect_size + vec2(radius);
    float dist = length(max(d, 0.0)) - radius;

    // Anti-aliased edge
    float alpha = smoothstep(1.0, 0.0, dist);

    return alpha;
}

// Noise function from https://www.shadertoy.com/view/tlcBRl
float white_noise(float seed1, float seed2){
    return (fract(seed1 + 12.34567 * fract(100.0 * (abs(seed1 * 0.91) + seed2 + 94.68)
        * fract((abs(seed2 * 0.41) + 45.46) * fract((abs(seed2) + 757.21) *
        fract(seed1 * 0.0171)))))) * 1.0038 - 0.00185;
}

float noise(vec2 frag_coord) {
    float random_value = (white_noise(frag_coord.x, frag_coord.y) * 2.0) - 1.0;
    return sign(random_value) * pow(abs(random_value), 100.0) * 1000.0;
}

void main() {
    float offset = (noise(vec2(gl_FragCoord.xy)) + gl_FragCoord.x - gl_FragCoord.y) * scale;
    float color = cos((time - offset) * 0.005) - 0.3;
    float vignetted_color = color * vignette(gl_FragCoord.xy);
    gl_FragColor = vec4(vignetted_color);
}
`;

// Array containing 3 vertices that construct a fullscreen triangle. Using // for auto-formatting.
const FULLSCREEN_TRIANGLE_VERTICES = new Float32Array([ //
    -1.0, -1.0, 0.0, 1.0,   //
    3.0, -1.0, 0.0, 1.0,    //
    -1.0, 3.0, 0.0, 1.0     //
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

function setupFullscreenTriangle(gl, program) {
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    const array_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, array_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, FULLSCREEN_TRIANGLE_VERTICES, gl.STATIC_DRAW);

    const positionLocation = gl.getAttribLocation(program, "position");
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 4, gl.FLOAT, false, 0, 0);

    return vao;
}

class LazyAnimation {
    #gl;
    #program;
    #timeLocation;
    #sizeLocation;
    #scaleLocation;
    #vao;
    #frameRequestId;

    constructor(canvas) {
        this.#gl = canvas.getContext("webgl2");
        this.#program = createProgram(this.#gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        this.#timeLocation = this.#gl.getUniformLocation(this.#program, "time");
        this.#scaleLocation = this.#gl.getUniformLocation(this.#program, "scale");
        this.#sizeLocation = this.#gl.getUniformLocation(this.#program, "size");
        this.#vao = setupFullscreenTriangle(this.#gl, this.#program);
        this.#scheduleDraw();
    }

    destroy() {
        cancelAnimationFrame(this.#frameRequestId);
        this.#gl = null;
        this.#vao = null;
        this.#program = null;
    }

    resizeCanvas(size, devicePixelRatio) {
        const gl = this.#gl;

        // Update canvas size and viewport
        gl.canvas.width = size.width;
        gl.canvas.height = size.height;
        gl.viewport(0, 0, size.width, size.height);

        // Set uniforms
        gl.useProgram(this.#program);
        gl.uniform2f(this.#sizeLocation, size.width, size.height);
        gl.uniform1f(this.#scaleLocation, 1 / devicePixelRatio);
        gl.useProgram(null);
    }

    #scheduleDraw() {
        this.#frameRequestId = requestAnimationFrame(timestamp => this.#draw(timestamp));
    }

    #draw(timestamp) {
        const gl = this.#gl;

        // Prepare state
        gl.useProgram(this.#program);
        gl.bindVertexArray(this.#vao);

        // Set Uniforms
        gl.uniform1f(this.#timeLocation, timestamp)

        // draw the fullscreen triangle
        gl.drawArrays(gl.TRIANGLES, 0, 3);

        // Reset state
        gl.bindVertexArray(null)
        gl.useProgram(null);

        this.#scheduleDraw();
    }
}

let lazyAnimation;
self.onmessage = (message) => {
    switch (message.data.type) {
        case "init": {
            lazyAnimation = new LazyAnimation(message.data.canvas);
            break;
        }
        case "resize": {
            if (lazyAnimation !== null) {
                lazyAnimation.resizeCanvas(message.data.canvasSize, message.data.devicePixelRatio);
            }
            // Let the main thread know we have received the first resize event, but for simplicity we fire on every event.
            self.postMessage({type: "started"});
            break;
        }
        case "cancel": {
            // TODO(Menno 23.05.2025) We should not destroy the animation abruptly, instead it should fade out.
            lazyAnimation.destroy();
            lazyAnimation = null;
            self.postMessage({type: "stopped"});
            break;
        }
        default: {
            console.error(`message of unknown type ${message.data.type}`);
            break;
        }
    }
}
