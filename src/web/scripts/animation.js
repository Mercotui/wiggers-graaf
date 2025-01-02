/**
 * Animate a number, in a given range over a duration, using an easing function
 */
class Animation {
    duration
    easingFunc
    onUpdateFunc
    onDoneFunc
    range
    startTimestamp
    animationRequestId = undefined

    /**
     * Create a number animation
     * @param duration in milliseconds
     * @param easingFunc easing function
     * @param onUpdateFunc function to call with animated value
     * @param onDoneFunc function to call when animation has completed
     * @param range the range to animate in
     */
    constructor({
                    duration = 1.0,
                    easingFunc = easeLinear,
                    onUpdateFunc = DummyCallback,
                    onDoneFunc = DummyCallback,
                    range = [0.0, 1.0]
                }) {
        this.duration = duration;
        this.easingFunc = easingFunc;
        this.onUpdateFunc = onUpdateFunc;
        this.onDoneFunc = onDoneFunc;
        this.range = range;
    }

    /**
     * Start the animation
     */
    start() {
        this.startTimestamp = undefined;
        this.#schedule();
    }

    /**
     * Cancel the animation
     */
    cancel() {
        if (this.animationRequestId !== undefined) {
            cancelAnimationFrame(this.animationRequestId)
        }

    }

    /**
     * Execute an animation tick, call callbacks if needed
     * @param nextDrawTimestamp the estimated timestamp of when the next frame will be drawn
     */
    #tick(nextDrawTimestamp) {
        if (this.startTimestamp === undefined) {
            this.startTimestamp = nextDrawTimestamp
        }

        // Calculate our current animation value
        const timeDelta = nextDrawTimestamp - this.startTimestamp;
        const progress = timeDelta / this.duration;
        const clamped = Math.max(0.0, Math.min(progress, 1.0));
        const eased = this.easingFunc(clamped);
        const remapped = this.range[0] + (eased * (this.range[1] - this.range[0]));

        // Use the current animation value
        this.onUpdateFunc(remapped);

        // Schedule the next animation step, or notify if done
        if (progress < 1.0) {
            this.#schedule();
        } else {
            this.onDoneFunc()
        }
    }

    /**
     * Schedule an animation tick
     */
    #schedule() {
        // TODO(Menno 02.01.2025) In case of more complex animations/game logic,
        //  it might be better to consolidate these calls into one AnimationRunner.
        //  Would also be nice to unittest this, but I don't want to install NPM stuff for that.
        //  Anyway, for this project requesting a few parallel callbacks is fine.
        this.animationRequestId = requestAnimationFrame((timestamp) => this.#tick(timestamp));
    }
}

/**
 * Delay an animation sequence for a given time, does not emit value updates
 */
class Delay {
    delay
    timeoutId = undefined
    onUpdateFunc
    onDoneFunc

    /**
     * Create a delay
     * @param delay time in milliseconds
     * @param onUpdateFunc animation value update callback, unused
     * @param onDoneFunc callback to call when delay time has elapsed
     */
    constructor({delay, onUpdateFunc = DummyCallback, onDoneFunc = DummyCallback,}) {
        this.onUpdateFunc = onUpdateFunc;
        this.onDoneFunc = onDoneFunc;
        this.delay = delay;
    }

    /**
     * Start the delay
     */
    start() {
        this.cancel();
        this.timeoutId = setTimeout(this.#timeout.bind(this), this.delay);
    }

    /**
     * Cancel the delay
     */
    cancel() {
        if (this.timeoutId !== undefined) {
            clearTimeout(this.timeoutId);
        }
    }

    /**
     * Internal timeout callback
     */
    #timeout() {
        this.timeoutId = undefined;
        this.onDoneFunc();
    }
}

/**
 * Loop an animation
 */
class Loop {
    animation
    countTarget
    count = 0;
    onUpdateFunc
    onDoneFunc

    /**
     * Create an animation loop
     * @param animation The animation to be looped
     * @param count The number of times to loop, -1 will start an infinite loop (until cancel is called)
     * @param onUpdateFunc Animation value update callback
     * @param onDoneFunc Callback to call when the loop has been executed 'count' number of times
     */
    constructor({
                    animation,
                    count = -1,
                    onUpdateFunc = DummyCallback,
                    onDoneFunc = DummyCallback,
                }) {
        this.countTarget = count;
        this.onUpdateFunc = onUpdateFunc;
        this.onDoneFunc = onDoneFunc;

        this.animation = animation;
        this.animation.onUpdateFunc = this.#onSubUpdate.bind(this);
        this.animation.onDoneFunc = this.#startInternal.bind(this);
    }

    /**
     * Start the animation loop
     */
    start() {
        this.count = 0;
        this.#startInternal();
    }

    /**
     * Cancel the animation loop
     */
    cancel() {
        this.animation.cancel()
    }

    /**
     * Internal start of loop
     */
    #startInternal() {
        if (this.countTarget === -1 || this.count < this.countTarget) {
            this.count++;
            this.animation.start();
        } else {
            this.onDoneFunc();
        }
    }

    /**
     * Internal value update function called by 'animation'
     * @param x
     */
    #onSubUpdate(x) {
        // Note it might be tempting to assign this update callback directly to the sub in the constructor,
        // but the callback might be modified after creation.
        this.onUpdateFunc(x);
    }
}

/**
 * Create a sequence of animations
 */
class Sequence {
    animations
    nextIndex = 0;
    onUpdateFunc
    onDoneFunc

    /**
     * Create an animation sequence
     * @param animations An array of animations to run
     * @param onUpdateFunc The animation value update callback
     * @param onDoneFunc The callback to call when the sequence has been fully executed
     */
    constructor({
                    animations,
                    onUpdateFunc = DummyCallback,
                    onDoneFunc = DummyCallback,
                }) {
        this.onUpdateFunc = onUpdateFunc;
        this.onDoneFunc = onDoneFunc;

        this.animations = animations;
        this.animations.forEach(animation => {
            animation.onUpdateFunc = this.#onSubUpdate.bind(this);
            animation.onDoneFunc = this.#next.bind(this);
        })
    }

    /**
     * Start the sequence
     */
    start() {
        this.nextIndex = 0;
        this.#next()
    }

    /**
     * Cancel the sequence
     */
    cancel() {
        if (0 < this.nextIndex && this.nextIndex <= this.animations.length) {
            this.animations[this.nextIndex - 1].cancel();
        }
    }

    /**
     * Internal callback to progress to the next sub-animation
     */
    #next() {
        if (this.nextIndex < this.animations.length) {
            this.animations[this.nextIndex].start();
            this.nextIndex++;
        } else {
            this.onDoneFunc();
        }
    }

    /**
     * Internal value update function called by the sub 'animations'
     * @param x
     */
    #onSubUpdate(x) {
        // Note it might be tempting to assign this update callback directly to the subs in the constructor,
        // but the callback might be modified after creation.
        this.onUpdateFunc(x);
    }
}

export {Animation, Delay, Loop, Sequence};

function DummyCallback() {
    // Do nothing.
}

// Easing functions from https://gist.github.com/gre/1650294
export const ease = {
    // no easing, no acceleration
    linear: t => t,
    // accelerating from zero velocity
    inQuad: t => t * t,
    // decelerating to zero velocity
    outQuad: t => t * (2 - t),
    // acceleration until halfway, then deceleration
    inOutQuad: t => t < .5 ? 2 * t * t : -1 + (4 - 2 * t) * t,
    // accelerating from zero velocity
    inCubic: t => t * t * t,
    // decelerating to zero velocity
    outCubic: t => (--t) * t * t + 1,
    // acceleration until halfway, then deceleration
    inOutCubic: t => t < .5 ? 4 * t * t * t : (t - 1) * (2 * t - 2) * (2 * t - 2) + 1,
    // accelerating from zero velocity
    inQuart: t => t * t * t * t,
    // decelerating to zero velocity
    outQuart: t => 1 - (--t) * t * t * t,
    // acceleration until halfway, then deceleration
    inOutQuart: t => t < .5 ? 8 * t * t * t * t : 1 - 8 * (--t) * t * t * t,
    // accelerating from zero velocity
    inQuint: t => t * t * t * t * t,
    // decelerating to zero velocity
    outQuint: t => 1 + (--t) * t * t * t * t,
    // acceleration until halfway, then deceleration
    inOutQuint: t => t < .5 ? 16 * t * t * t * t * t : 1 + 16 * (--t) * t * t * t * t
}
