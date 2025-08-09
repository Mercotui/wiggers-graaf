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
uniform float fade_in;
uniform float fade_out;

const float CORNER_RADIUS = 16.0;
const float TAU = 6.28318530718;

// Apply rounded rectangle SDF
float vignette(vec2 frag_coord) {
    float radius = CORNER_RADIUS / scale;
    vec2 half_rect_size = (size - vec2(radius * 0.5)) * 0.5;
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
    vec2 truncated_coord = vec2(floor(frag_coord.y * 0.4), floor(frag_coord.x * 0.02));
    float random_value = (white_noise(truncated_coord.x, truncated_coord.y) * 2.0) - 1.0;
    return sign(random_value) * pow(abs(random_value), 200.0);
}

vec3 chromatic_aberration(vec2 uv) {
    vec3 aberration = vec3(uv.y, uv.y + uv.x + 0.33, uv.y - uv.x + TAU * 0.66) * TAU;
    return vec3(cos(aberration.r), cos(aberration.g), cos(aberration.b));
}

void main() {
    vec2 frag_coord = vec2(gl_FragCoord.xy);
    vec2 uv = (frag_coord / size);

    // Create blobby colors with noise
    float noise_offset = noise(frag_coord);
    float angle = (time + frag_coord.y - frag_coord.x) * 0.002 + noise_offset + fade_in * 5.0 + (fade_out * 10.0);
    vec3 angles = vec3(angle) + chromatic_aberration(uv);
    vec3 color = (vec3(cos(angles.r), cos(angles.g), cos(angles.b)) + 1.5) * 0.2;

    // Apply a border to the animation
    float vignette_alpha = vignette(frag_coord);
    vec3 vignetted_color = color * vignette_alpha;

    // Take the average color value as alpha, but slightly increase the value to make the colors darker
    float alpha = (vignetted_color.r + vignetted_color.g + vignetted_color.b) * 0.5;

    // Fade in and out of the animation at the beginning and end respectively
    gl_FragColor = vec4(vignetted_color, alpha) * (fade_in - fade_out);
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
    #fadeInLocation;
    #fadeOutLocation;
    #vao;
    #frameRequestId;
    #fadeIn = 0.0;
    #fadeOut = 0.0;
    #fadeInVelocity = 0.0002;
    #fadeOutVelocity = 0.000;
    #previousTimestamp = null;
    #concludedCb = () => {
    };

    constructor(canvas) {
        this.#gl = canvas.getContext("webgl2");
        if (this.#gl === null) {
            throw new Error("Could not create WebGL2 context inside LazyAnimation Webworker, skipping loading animation");
        }

        this.#program = createProgram(this.#gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        this.#timeLocation = this.#gl.getUniformLocation(this.#program, "time");
        this.#scaleLocation = this.#gl.getUniformLocation(this.#program, "scale");
        this.#sizeLocation = this.#gl.getUniformLocation(this.#program, "size");
        this.#fadeInLocation = this.#gl.getUniformLocation(this.#program, "fade_in");
        this.#fadeOutLocation = this.#gl.getUniformLocation(this.#program, "fade_out");
        this.#vao = setupFullscreenTriangle(this.#gl, this.#program);
        this.#scheduleDraw();
    }

    conclude() {
        // Start fading out the animation
        this.#fadeOutVelocity = 0.00001;
        return new Promise((resolve, _reject) => {
            this.#concludedCb = resolve;
        });
    }

    destroy() {
        this.#cancelDraw();
        this.#gl = null;
        this.#vao = null;
        this.#program = null;
    }

    setPaused(paused) {
        if (paused) {
            this.#cancelDraw();
        } else {
            this.#scheduleDraw();
        }
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
        if (this.#frameRequestId) {
            return;
        }

        this.#frameRequestId = requestAnimationFrame(timestamp => {
            this.#frameRequestId = undefined;
            this.#draw(timestamp);
        });
    }

    #cancelDraw() {
        cancelAnimationFrame(this.#frameRequestId);
        this.#frameRequestId = undefined;
    }

    #updateFade(timestamp) {
        if (this.#previousTimestamp === null) {
            this.#previousTimestamp = timestamp;
            return;
        }

        const delta = timestamp - this.#previousTimestamp;

        const newFadeIn = this.#fadeIn + this.#fadeInVelocity * delta;
        this.#fadeIn = Math.min(1.0, Math.max(0.0, newFadeIn));

        const newFadeOut = this.#fadeOut + this.#fadeOutVelocity * delta;
        this.#fadeOut = Math.min(1.0, Math.max(0.0, newFadeOut));
        if (this.#fadeOut === 1.0) {
            this.#concludedCb();
        }
    }

    #draw(timestamp) {
        const gl = this.#gl;

        // Prepare state
        gl.useProgram(this.#program);
        gl.bindVertexArray(this.#vao);

        // Set Uniforms
        this.#updateFade(timestamp);
        gl.uniform1f(this.#fadeInLocation, this.#fadeIn);
        gl.uniform1f(this.#fadeOutLocation, this.#fadeOut);
        gl.uniform1f(this.#timeLocation, timestamp)

        // draw the fullscreen triangle
        gl.drawArrays(gl.TRIANGLES, 0, 3);

        // Reset state
        gl.bindVertexArray(null)
        gl.useProgram(null);

        this.#scheduleDraw();
    }
}

let lazyAnimation = null;
self.onmessage = (message) => {
    switch (message.data.type) {
        case "init": {
            try {
                // Try to create the animation, depends on WebGL2 support
                lazyAnimation = new LazyAnimation(message.data.canvas);
            } catch (e) {
                console.error(e);
            }
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
        case "pause": {
            if (lazyAnimation !== null) {
                lazyAnimation.setPaused(message.data.paused);
            }
            break;
        }
        case "cancel": {
            if (lazyAnimation === null) {
                self.postMessage({type: "stopped"});
                break;
            }
            lazyAnimation.conclude().then(() => {
                lazyAnimation.destroy();
                lazyAnimation = null;
                self.postMessage({type: "stopped"});
            });
            break;
        }
        default: {
            console.error(`message of unknown type ${message.data.type}`);
            break;
        }
    }
}
