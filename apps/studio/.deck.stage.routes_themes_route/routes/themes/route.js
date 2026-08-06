import { a as openTheme, i as loadThemes, n as deleteTheme, o as subscribeDesign, r as getDesignState, t as addThemeToProject } from "../../design-CaMptCCL.js";
import { __tpl, bindAttr, bindFor, bindIf, bindProp, bindText, component, controller, delegateEvents, markHydrateText, on, runAction } from "@ship/sail-runtime/internal";
import { mountAmbient, signal } from "@ship/sail-runtime";
import { navigate } from "@ship/deck/router";

//#region ../../packages/sail/runtime/src/actions/variant-live.ts
/**
* Observe `variant` and flip role/aria-live for connecting placeholders.
* Static defaults on the template (`role=note`, `aria-live=off`) keep the
* host inert before hydrate; this action only upgrades `connecting` and
* re-evaluates when the attribute changes.
*/
const variantLive = (node) => {
	const el = node;
	const sync = () => {
		const connecting = el.getAttribute("variant") === "connecting";
		el.setAttribute("role", connecting ? "status" : "note");
		el.setAttribute("aria-live", connecting ? "polite" : "off");
	};
	sync();
	const obs = new MutationObserver(sync);
	obs.observe(el, {
		attributes: true,
		attributeFilter: ["variant"]
	});
	return { destroy: () => obs.disconnect() };
};

//#endregion
//#region ../../packages/sail/runtime/src/transition-runner.ts
/** Stub: animations are opt-in by importing transition.ts. */
let _runEnter = () => {};
let _runLeave = () => null;
let _cancelTransitions = () => {};
/** Called by transition.ts at module-init time to swap stubs for
*  the real impl. Idempotent — re-importing transition.ts is a
*  no-op (last writer wins, but the impls are the same). */
function setTransitionRunner(impl) {
	_runEnter = impl.runEnter;
	_runLeave = impl.runLeave;
	_cancelTransitions = impl.cancelTransitions;
}

//#endregion
//#region ../../packages/sail/runtime/src/error-boundary.ts
/** Symbol property the boundary host carries. Using a Symbol keeps
*  the key invisible to ordinary property enumeration and avoids name
*  collisions with user attributes. */
const BOUNDARY_KEY = Symbol.for("sail:error-boundary");
/**
* Walk up the DOM from `node` until the nearest element carrying a
* `BoundaryHandle` is found, or the root is reached. L1 is light DOM.
* Residual: when the walk hits a `ShadowRoot` (foreign-content wrapper),
* continues from the host so a boundary outside that tree still catches
* inner throws.
*
* @param node Any node (element, text, comment) inside the subtree
*   being protected. Typically a binding anchor or a custom-element
*   host that just threw.
* @returns The handle of the innermost boundary ancestor, or `null`
*   if there is none.
*/
function findNearestBoundary(node) {
	let cur = node;
	while (cur) {
		if (cur instanceof Element) {
			const handle = cur[BOUNDARY_KEY];
			if (handle) return handle;
		}
		const parent = cur.parentNode;
		if (parent) {
			cur = parent;
			continue;
		}
		const root = cur;
		if (root.host) {
			cur = root.host;
			continue;
		}
		cur = null;
	}
	return null;
}

//#endregion
//#region ../../packages/sail/runtime/src/error-overlay.ts
/**
* Dispatch the `sail:error` fallback event. Bubbles + composed from `host`
* when it's an element (so an ancestor overlay listener catches it even
* across shadow-root boundaries); dispatched on `document` otherwise.
* `tag` defaults to `host.dataset.component`, set by the component wrapper
* before setup/effects can run.
*/
function dispatchSailError(host, error, phase) {
	const detail = {
		error,
		phase
	};
	let target = typeof document !== "undefined" ? document : null;
	if (host != null && typeof host.tagName === "string") {
		detail.host = host;
		target = host;
		const tag = host.dataset?.component;
		if (tag) detail.tag = tag;
	}
	if (typeof target?.dispatchEvent !== "function") return;
	if (typeof CustomEvent !== "function") return;
	try {
		target.dispatchEvent(new CustomEvent("sail:error", {
			detail,
			bubbles: true,
			composed: true
		}));
	} catch {}
}

//#endregion
//#region ../../packages/sail/runtime/src/effect-scope.ts
let CURRENT_SCOPE = null;
/** Run `body` with `scope` as the current scope. Cleanups registered
*  inside `body` enroll on `scope`. Restores the prior scope on
*  return, including on throw. Nestable. */
function withScope(scope, body) {
	const prev = CURRENT_SCOPE;
	CURRENT_SCOPE = scope;
	try {
		return body();
	} finally {
		CURRENT_SCOPE = prev;
	}
}
/** Register a teardown function with the current scope. No-op when
*  there is no current scope (i.e. called outside any `withScope`). */
function onCleanup(fn) {
	const scope = CURRENT_SCOPE;
	if (!scope) return;
	scope.cleanup(fn);
}
/** The active scope, or `null` if no `withScope` is currently on the
*  stack. Diagnostic / advanced use only — most callers should use
*  `onCleanup` directly. */
function currentScope() {
	return CURRENT_SCOPE;
}

//#endregion
//#region ../../packages/sail/runtime/src/signal.ts
const CLEAN = 0;
const DIRTY = 2;
/** Build an effect node's reused ownership-scope adapter. Cleanups registered
*  while this scope is current (the body run) land directly on `node.cleanups`,
*  which `runEffect`'s `runCleanups` fires before each re-run and the effect's
*  disposer fires on teardown. `dispose`/`disposed` are inert: the node owns
*  its own lifecycle, so the scope is purely a routing target for `onCleanup`. */
function makeOwnerScope(node) {
	const ctx = /* @__PURE__ */ new Map();
	return {
		cleanup(fn) {
			node.cleanups.push(fn);
		},
		registerMount(fn) {
			node.owner?.registerMount(fn);
		},
		runMounts() {},
		dispose() {},
		disposed: false,
		parent: node.owner ?? null,
		contexts: ctx
	};
}
let CURRENT_OBSERVER = null;
const HOST_STACK = [];
function currentHost() {
	return HOST_STACK.length > 0 ? HOST_STACK[HOST_STACK.length - 1] : null;
}
/** Run and clear a node's custom teardown thunks. */
function runCleanups(node) {
	const cs = node.cleanups;
	if (cs.length === 0) return;
	node.cleanups = [];
	for (let i = 0; i < cs.length; i++) try {
		cs[i]();
	} catch (e) {
		console.error("[sail] cleanup threw:", e);
	}
}
/** Drop this computation from every source it was subscribed to. */
function unlinkSources(node) {
	const srcs = node.sources;
	for (let i = 0; i < srcs.length; i++) srcs[i].observers.delete(node);
	node.sources = [];
}
/** Run an effect body, capturing any returned cleanup. Errors route to
*  the nearest error boundary (or console) — one bad effect never breaks
*  the graph. */
function runEffect(node) {
	runCleanups(node);
	unlinkSources(node);
	node.state = CLEAN;
	const scope = node.scope ??= makeOwnerScope(node);
	const prev = CURRENT_OBSERVER;
	CURRENT_OBSERVER = node;
	try {
		const c = withScope(scope, () => node.fn());
		if (typeof c === "function") node.cleanups.push(c);
	} catch (e) {
		const boundary = node.host ? findNearestBoundary(node.host) : null;
		if (boundary) boundary.catch(e, "effect");
		else {
			console.error("[sail] effect threw:", e);
			dispatchSailError(node.host ?? null, e, "effect");
		}
	} finally {
		CURRENT_OBSERVER = prev;
	}
}
/**
* Run `fn` once eagerly, then re-run on every tracked dependency
* change. Returns a disposer that unsubscribes and stops further runs.
*
* `fn` may return a cleanup callback. The cleanup runs once before each
* re-run (so users can detach listeners, abort fetches, etc.) and once
* on disposal. Errors thrown inside `fn` or its cleanup are logged and
* swallowed — one bad effect never breaks the reactive graph.
*
* Dependencies are re-collected each run: signals read in run N but not in
* run N+1 are unsubscribed. Disposal proactively unsubscribes from every
* source the effect was tracking.
*
* @param fn Side-effecting function; tracked reads register as deps.
*   May return an optional cleanup callback.
* @returns A disposer that detaches the effect.
*
* @example
* ```ts
* const url = signal('/api/feed')
* const dispose = effect(() => {
*   const ctrl = new AbortController()
*   fetch(url(), { signal: ctrl.signal }).catch(() => {})
*   return () => ctrl.abort()  // cleanup runs before next re-run
* })
* // later...
* dispose()
* ```
*
* @see signal, derived; batch to coalesce writes that would otherwise
*   re-fire the effect multiple times.
*/
function effect(fn) {
	const node = {
		fn,
		state: DIRTY,
		sources: [],
		observers: /* @__PURE__ */ new Set(),
		cleanups: [],
		isEffect: true,
		active: true,
		computation: true,
		host: currentHost(),
		owner: currentScope()
	};
	runEffect(node);
	const dispose = () => {
		if (!node.active) return;
		node.active = false;
		runCleanups(node);
		unlinkSources(node);
		node.state = CLEAN;
	};
	if (CURRENT_OBSERVER !== null) onCleanup(dispose);
	return dispose;
}
/**
* Read signals without subscribing the current observer to them.
* Useful inside effects/derived when a value is consulted but
* shouldn't trigger re-runs (timestamps, RNG seeds, peeking at peers).
*
* @param fn Code that reads signals; reads inside are not tracked.
* @returns Whatever `fn` returns.
*
* @example
* ```ts
* const count = signal(0), label = signal('hits')
* effect(() => {
*   const stamp = untrack(() => Date.now())     // no dep on Date
*   console.log(`${label()}: ${count()} @${stamp}`)
* })
* ```
*
* @see effect, derived.
*/
function untrack(fn) {
	const prev = CURRENT_OBSERVER;
	CURRENT_OBSERVER = null;
	try {
		return fn();
	} finally {
		CURRENT_OBSERVER = prev;
	}
}

//#endregion
//#region ../../node_modules/motion-utils/dist/es/clamp.mjs
const clamp = (min, max, v) => {
	if (v > max) return max;
	if (v < min) return min;
	return v;
};

//#endregion
//#region ../../node_modules/motion-utils/dist/es/format-error-message.mjs
function formatErrorMessage(message, errorCode) {
	return errorCode ? `${message}. For more information and steps for solving, visit https://motion.dev/troubleshooting/${errorCode}` : message;
}

//#endregion
//#region ../../node_modules/motion-utils/dist/es/errors.mjs
let warning = () => {};
let invariant = () => {};
if (typeof process !== "undefined" && true) {
	warning = (check, message, errorCode) => {
		if (!check && typeof console !== "undefined") console.warn(formatErrorMessage(message, errorCode));
	};
	invariant = (check, message, errorCode) => {
		if (!check) throw new Error(formatErrorMessage(message, errorCode));
	};
}

//#endregion
//#region ../../node_modules/motion-utils/dist/es/memo.mjs
/*#__NO_SIDE_EFFECTS__*/
function memo(callback) {
	let result;
	return () => {
		if (result === void 0) result = callback();
		return result;
	};
}

//#endregion
//#region ../../node_modules/motion-utils/dist/es/noop.mjs
const noop = /* @__NO_SIDE_EFFECTS__ */ (any) => any;

//#endregion
//#region ../../node_modules/motion-utils/dist/es/time-conversion.mjs
/**
* Converts seconds to milliseconds
*
* @param seconds - Time in seconds.
* @return milliseconds - Converted time in milliseconds.
*/
const secondsToMilliseconds = /* @__NO_SIDE_EFFECTS__ */ (seconds) => seconds * 1e3;
const millisecondsToSeconds = /* @__NO_SIDE_EFFECTS__ */ (milliseconds) => milliseconds / 1e3;

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/cubic-bezier.mjs
const calcBezier = (t, a1, a2) => (((1 - 3 * a2 + 3 * a1) * t + (3 * a2 - 6 * a1)) * t + 3 * a1) * t;
const subdivisionPrecision = 1e-7;
const subdivisionMaxIterations = 12;
function binarySubdivide(x, lowerBound, upperBound, mX1, mX2) {
	let currentX;
	let currentT;
	let i = 0;
	do {
		currentT = lowerBound + (upperBound - lowerBound) / 2;
		currentX = calcBezier(currentT, mX1, mX2) - x;
		if (currentX > 0) upperBound = currentT;
		else lowerBound = currentT;
	} while (Math.abs(currentX) > subdivisionPrecision && ++i < subdivisionMaxIterations);
	return currentT;
}
/*#__NO_SIDE_EFFECTS__*/
function cubicBezier(mX1, mY1, mX2, mY2) {
	if (mX1 === mY1 && mX2 === mY2) return noop;
	const getTForX = (aX) => binarySubdivide(aX, 0, 1, mX1, mX2);
	return (t) => t === 0 || t === 1 ? t : calcBezier(getTForX(t), mY1, mY2);
}

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/modifiers/mirror.mjs
const mirrorEasing = /* @__NO_SIDE_EFFECTS__ */ (easing) => (p) => p <= .5 ? easing(2 * p) / 2 : (2 - easing(2 * (1 - p))) / 2;

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/modifiers/reverse.mjs
const reverseEasing = /* @__NO_SIDE_EFFECTS__ */ (easing) => (p) => 1 - easing(1 - p);

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/back.mjs
const backOut = /*@__PURE__*/ cubicBezier(.33, 1.53, .69, .99);
const backIn = /*@__PURE__*/ reverseEasing(backOut);
const backInOut = /*@__PURE__*/ mirrorEasing(backIn);

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/anticipate.mjs
const anticipate = (p) => p >= 1 ? 1 : (p *= 2) < 1 ? .5 * backIn(p) : .5 * (2 - Math.pow(2, -10 * (p - 1)));

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/circ.mjs
const circIn = (p) => 1 - Math.sin(Math.acos(p));
const circOut = reverseEasing(circIn);
const circInOut = mirrorEasing(circIn);

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/ease.mjs
const easeIn = /*@__PURE__*/ cubicBezier(.42, 0, 1, 1);
const easeOut = /*@__PURE__*/ cubicBezier(0, 0, .58, 1);
const easeInOut = /*@__PURE__*/ cubicBezier(.42, 0, .58, 1);

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/utils/is-bezier-definition.mjs
const isBezierDefinition = /* @__NO_SIDE_EFFECTS__ */ (easing) => Array.isArray(easing) && typeof easing[0] === "number";

//#endregion
//#region ../../node_modules/motion-utils/dist/es/easing/utils/map.mjs
const easingLookup = {
	linear: noop,
	easeIn,
	easeInOut,
	easeOut,
	circIn,
	circInOut,
	circOut,
	backIn,
	backInOut,
	backOut,
	anticipate
};
const isValidEasing = (easing) => {
	return typeof easing === "string";
};
const easingDefinitionToFunction = (definition) => {
	if (isBezierDefinition(definition)) {
		invariant(definition.length === 4, `Cubic bezier arrays must contain four numerical values.`, "cubic-bezier-length");
		const [x1, y1, x2, y2] = definition;
		return cubicBezier(x1, y1, x2, y2);
	} else if (isValidEasing(definition)) {
		invariant(easingLookup[definition] !== void 0, `Invalid easing type '${definition}'`, "invalid-easing-type");
		return easingLookup[definition];
	}
	return definition;
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/utils/linear.mjs
const generateLinearEasing = (easing, duration, resolution = 10) => {
	let points = "";
	const numPoints = Math.max(Math.round(duration / resolution), 2);
	for (let i = 0; i < numPoints; i++) points += Math.round(easing(i / (numPoints - 1)) * 1e4) / 1e4 + ", ";
	return `linear(${points.substring(0, points.length - 2)})`;
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/generators/utils/calc-duration.mjs
/**
* Implement a practical max duration for keyframe generation
* to prevent infinite loops
*/
const maxGeneratorDuration = 2e4;
function calcGeneratorDuration(generator) {
	let duration = 0;
	const timeStep = 50;
	let state = generator.next(duration);
	while (!state.done && duration < 2e4) {
		duration += timeStep;
		state = generator.next(duration);
	}
	return duration >= 2e4 ? Infinity : duration;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/generators/utils/create-generator-easing.mjs
/**
* Create a progress => progress easing function from a generator.
*/
function createGeneratorEasing(options, scale = 100, createGenerator) {
	const generator = createGenerator({
		...options,
		keyframes: [0, scale]
	});
	const duration = Math.min(calcGeneratorDuration(generator), maxGeneratorDuration);
	return {
		type: "keyframes",
		ease: (progress) => {
			return generator.next(duration * progress).value / scale;
		},
		duration: millisecondsToSeconds(duration)
	};
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/generators/spring.mjs
const springDefaults = {
	stiffness: 100,
	damping: 10,
	mass: 1,
	velocity: 0,
	duration: 800,
	bounce: .3,
	visualDuration: .3,
	restSpeed: {
		granular: .01,
		default: 2
	},
	restDelta: {
		granular: .005,
		default: .5
	},
	minDuration: .01,
	maxDuration: 10,
	minDamping: .05,
	maxDamping: 1
};
function calcAngularFreq(undampedFreq, dampingRatio) {
	return undampedFreq * Math.sqrt(1 - dampingRatio * dampingRatio);
}
const rootIterations = 12;
function approximateRoot(envelope, derivative, initialGuess) {
	let result = initialGuess;
	for (let i = 1; i < rootIterations; i++) result = result - envelope(result) / derivative(result);
	return result;
}
/**
* This is ported from the Framer implementation of duration-based spring resolution.
*/
const safeMin = .001;
function findSpring({ duration = springDefaults.duration, bounce = springDefaults.bounce, velocity = springDefaults.velocity, mass = springDefaults.mass }) {
	let envelope;
	let derivative;
	warning(duration <= secondsToMilliseconds(springDefaults.maxDuration), "Spring duration must be 10 seconds or less", "spring-duration-limit");
	let dampingRatio = 1 - bounce;
	/**
	* Restrict dampingRatio and duration to within acceptable ranges.
	*/
	dampingRatio = clamp(springDefaults.minDamping, springDefaults.maxDamping, dampingRatio);
	duration = clamp(springDefaults.minDuration, springDefaults.maxDuration, millisecondsToSeconds(duration));
	if (dampingRatio < 1) {
		/**
		* Underdamped spring
		*/
		envelope = (undampedFreq) => {
			const exponentialDecay = undampedFreq * dampingRatio;
			const delta = exponentialDecay * duration;
			const a = exponentialDecay - velocity;
			const b = calcAngularFreq(undampedFreq, dampingRatio);
			const c = Math.exp(-delta);
			return safeMin - a / b * c;
		};
		derivative = (undampedFreq) => {
			const delta = undampedFreq * dampingRatio * duration;
			const d = delta * velocity + velocity;
			const e = Math.pow(dampingRatio, 2) * Math.pow(undampedFreq, 2) * duration;
			const f = Math.exp(-delta);
			const g = calcAngularFreq(Math.pow(undampedFreq, 2), dampingRatio);
			return (-envelope(undampedFreq) + safeMin > 0 ? -1 : 1) * ((d - e) * f) / g;
		};
	} else {
		/**
		* Critically-damped spring
		*/
		envelope = (undampedFreq) => {
			return -.001 + Math.exp(-undampedFreq * duration) * ((undampedFreq - velocity) * duration + 1);
		};
		derivative = (undampedFreq) => {
			return Math.exp(-undampedFreq * duration) * ((velocity - undampedFreq) * (duration * duration));
		};
	}
	const initialGuess = 5 / duration;
	const undampedFreq = approximateRoot(envelope, derivative, initialGuess);
	duration = secondsToMilliseconds(duration);
	if (isNaN(undampedFreq)) return {
		stiffness: springDefaults.stiffness,
		damping: springDefaults.damping,
		duration
	};
	else {
		const stiffness = Math.pow(undampedFreq, 2) * mass;
		return {
			stiffness,
			damping: dampingRatio * 2 * Math.sqrt(mass * stiffness),
			duration
		};
	}
}
const durationKeys = ["duration", "bounce"];
const physicsKeys = [
	"stiffness",
	"damping",
	"mass"
];
function isSpringType(options, keys) {
	return keys.some((key) => options[key] !== void 0);
}
function getSpringOptions(options) {
	let springOptions = {
		velocity: springDefaults.velocity,
		stiffness: springDefaults.stiffness,
		damping: springDefaults.damping,
		mass: springDefaults.mass,
		isResolvedFromDuration: false,
		...options
	};
	if (!isSpringType(options, physicsKeys) && isSpringType(options, durationKeys)) {
		springOptions.velocity = 0;
		if (options.visualDuration) {
			const visualDuration = options.visualDuration;
			const root = 2 * Math.PI / (visualDuration * 1.2);
			const stiffness = root * root;
			const damping = 2 * clamp(.05, 1, 1 - (options.bounce || 0)) * Math.sqrt(stiffness);
			springOptions = {
				...springOptions,
				mass: springDefaults.mass,
				stiffness,
				damping
			};
		} else {
			const derived = findSpring({
				...options,
				velocity: 0
			});
			springOptions = {
				...springOptions,
				...derived,
				mass: springDefaults.mass
			};
			springOptions.isResolvedFromDuration = true;
		}
	}
	return springOptions;
}
function spring(optionsOrVisualDuration = springDefaults.visualDuration, bounce = springDefaults.bounce) {
	const options = typeof optionsOrVisualDuration !== "object" ? {
		visualDuration: optionsOrVisualDuration,
		keyframes: [0, 1],
		bounce
	} : optionsOrVisualDuration;
	let { restSpeed, restDelta } = options;
	const origin = options.keyframes[0];
	const target = options.keyframes[options.keyframes.length - 1];
	/**
	* This is the Iterator-spec return value. We ensure it's mutable rather than using a generator
	* to reduce GC during animation.
	*/
	const state = {
		done: false,
		value: origin
	};
	const { stiffness, damping, mass, duration, velocity, isResolvedFromDuration } = getSpringOptions({
		...options,
		velocity: -millisecondsToSeconds(options.velocity || 0)
	});
	const initialVelocity = velocity || 0;
	const dampingRatio = damping / (2 * Math.sqrt(stiffness * mass));
	const initialDelta = target - origin;
	const undampedAngularFreq = millisecondsToSeconds(Math.sqrt(stiffness / mass));
	/**
	* If we're working on a granular scale, use smaller defaults for determining
	* when the spring is finished.
	*
	* These defaults have been selected emprically based on what strikes a good
	* ratio between feeling good and finishing as soon as changes are imperceptible.
	*/
	const isGranularScale = Math.abs(initialDelta) < 5;
	restSpeed || (restSpeed = isGranularScale ? springDefaults.restSpeed.granular : springDefaults.restSpeed.default);
	restDelta || (restDelta = isGranularScale ? springDefaults.restDelta.granular : springDefaults.restDelta.default);
	let resolveSpring;
	let resolveVelocity;
	let angularFreq;
	let A;
	let sinCoeff;
	let cosCoeff;
	if (dampingRatio < 1) {
		angularFreq = calcAngularFreq(undampedAngularFreq, dampingRatio);
		A = (initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) / angularFreq;
		resolveSpring = (t) => {
			return target - Math.exp(-dampingRatio * undampedAngularFreq * t) * (A * Math.sin(angularFreq * t) + initialDelta * Math.cos(angularFreq * t));
		};
		sinCoeff = dampingRatio * undampedAngularFreq * A + initialDelta * angularFreq;
		cosCoeff = dampingRatio * undampedAngularFreq * initialDelta - A * angularFreq;
		resolveVelocity = (t) => {
			return Math.exp(-dampingRatio * undampedAngularFreq * t) * (sinCoeff * Math.sin(angularFreq * t) + cosCoeff * Math.cos(angularFreq * t));
		};
	} else if (dampingRatio === 1) {
		resolveSpring = (t) => target - Math.exp(-undampedAngularFreq * t) * (initialDelta + (initialVelocity + undampedAngularFreq * initialDelta) * t);
		const C = initialVelocity + undampedAngularFreq * initialDelta;
		resolveVelocity = (t) => Math.exp(-undampedAngularFreq * t) * (undampedAngularFreq * C * t - initialVelocity);
	} else {
		const dampedAngularFreq = undampedAngularFreq * Math.sqrt(dampingRatio * dampingRatio - 1);
		resolveSpring = (t) => {
			const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
			const freqForT = Math.min(dampedAngularFreq * t, 300);
			return target - envelope * ((initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) * Math.sinh(freqForT) + dampedAngularFreq * initialDelta * Math.cosh(freqForT)) / dampedAngularFreq;
		};
		const P = (initialVelocity + dampingRatio * undampedAngularFreq * initialDelta) / dampedAngularFreq;
		const sinhCoeff = dampingRatio * undampedAngularFreq * P - initialDelta * dampedAngularFreq;
		const coshCoeff = dampingRatio * undampedAngularFreq * initialDelta - P * dampedAngularFreq;
		resolveVelocity = (t) => {
			const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
			const freqForT = Math.min(dampedAngularFreq * t, 300);
			return envelope * (sinhCoeff * Math.sinh(freqForT) + coshCoeff * Math.cosh(freqForT));
		};
	}
	const generator = {
		calculatedDuration: isResolvedFromDuration ? duration || null : null,
		velocity: (t) => secondsToMilliseconds(resolveVelocity(t)),
		next: (t) => {
			/**
			* For underdamped physics springs we need both position and
			* velocity each tick. Compute shared trig values once to avoid
			* duplicate Math.exp/sin/cos calls on the hot path.
			*/
			if (!isResolvedFromDuration && dampingRatio < 1) {
				const envelope = Math.exp(-dampingRatio * undampedAngularFreq * t);
				const sin = Math.sin(angularFreq * t);
				const cos = Math.cos(angularFreq * t);
				const current = target - envelope * (A * sin + initialDelta * cos);
				const currentVelocity = secondsToMilliseconds(envelope * (sinCoeff * sin + cosCoeff * cos));
				state.done = Math.abs(currentVelocity) <= restSpeed && Math.abs(target - current) <= restDelta;
				state.value = state.done ? target : current;
				return state;
			}
			const current = resolveSpring(t);
			if (!isResolvedFromDuration) {
				const currentVelocity = secondsToMilliseconds(resolveVelocity(t));
				state.done = Math.abs(currentVelocity) <= restSpeed && Math.abs(target - current) <= restDelta;
			} else state.done = t >= duration;
			state.value = state.done ? target : current;
			return state;
		},
		toString: () => {
			const calculatedDuration = Math.min(calcGeneratorDuration(generator), maxGeneratorDuration);
			const easing = generateLinearEasing((progress) => generator.next(calculatedDuration * progress).value, calculatedDuration, 30);
			return calculatedDuration + "ms " + easing;
		},
		toTransition: () => {}
	};
	return generator;
}
spring.applyToOptions = (options) => {
	const generatorOptions = createGeneratorEasing(options, 100, spring);
	options.ease = generatorOptions.ease;
	options.duration = secondsToMilliseconds(generatorOptions.duration);
	options.type = "keyframes";
	return options;
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/keyframes/get-final.mjs
const isNotNull = (value) => value !== null;
function getFinalKeyframe(keyframes, { repeat, repeatType = "loop" }, finalKeyframe, speed = 1) {
	const resolvedKeyframes = keyframes.filter(isNotNull);
	const index = speed < 0 || repeat && repeatType !== "loop" && repeat % 2 === 1 ? 0 : resolvedKeyframes.length - 1;
	return !index || finalKeyframe === void 0 ? resolvedKeyframes[index] : finalKeyframe;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/utils/WithPromise.mjs
var WithPromise = class {
	constructor() {
		this.updateFinished();
	}
	get finished() {
		return this._finished;
	}
	updateFinished() {
		this._finished = new Promise((resolve) => {
			this.resolve = resolve;
		});
	}
	notifyFinished() {
		this.resolve();
	}
	/**
	* Allows the animation to be awaited.
	*
	* @deprecated Use `finished` instead.
	*/
	then(onResolve, onReject) {
		return this.finished.then(onResolve, onReject);
	}
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/keyframes/utils/fill-wildcards.mjs
function fillWildcards(keyframes) {
	for (let i = 1; i < keyframes.length; i++) keyframes[i] ?? (keyframes[i] = keyframes[i - 1]);
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/render/dom/is-css-var.mjs
const isCSSVar = (name) => name.startsWith("--");

//#endregion
//#region ../../node_modules/motion-dom/dist/es/render/dom/style-set.mjs
function setStyle(element, name, value) {
	isCSSVar(name) ? element.style.setProperty(name, value) : element.style[name] = value;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/supports/flags.mjs
/**
* Add the ability for test suites to manually set support flags
* to better test more environments.
*/
const supportsFlags = {};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/supports/memo.mjs
function memoSupports(callback, supportsFlag) {
	const memoized = memo(callback);
	return () => supportsFlags[supportsFlag] ?? memoized();
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/supports/scroll-timeline.mjs
const supportsScrollTimeline = /* @__PURE__ */ memoSupports(() => window.ScrollTimeline !== void 0, "scrollTimeline");

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/supports/linear-easing.mjs
const supportsLinearEasing = /*@__PURE__*/ memoSupports(() => {
	try {
		document.createElement("div").animate({ opacity: 0 }, { easing: "linear(0, 1)" });
	} catch (e) {
		return false;
	}
	return true;
}, "linearEasing");

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/easing/cubic-bezier.mjs
const cubicBezierAsString = ([a, b, c, d]) => `cubic-bezier(${a}, ${b}, ${c}, ${d})`;

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/easing/supported.mjs
const supportedWaapiEasing = {
	linear: "linear",
	ease: "ease",
	easeIn: "ease-in",
	easeOut: "ease-out",
	easeInOut: "ease-in-out",
	circIn: /*@__PURE__*/ cubicBezierAsString([
		0,
		.65,
		.55,
		1
	]),
	circOut: /*@__PURE__*/ cubicBezierAsString([
		.55,
		0,
		1,
		.45
	]),
	backIn: /*@__PURE__*/ cubicBezierAsString([
		.31,
		.01,
		.66,
		-.59
	]),
	backOut: /*@__PURE__*/ cubicBezierAsString([
		.33,
		1.53,
		.69,
		.99
	])
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/easing/map-easing.mjs
function mapEasingToNativeEasing(easing, duration) {
	if (!easing) return;
	else if (typeof easing === "function") return supportsLinearEasing() ? generateLinearEasing(easing, duration) : "ease-out";
	else if (isBezierDefinition(easing)) return cubicBezierAsString(easing);
	else if (Array.isArray(easing)) return easing.map((segmentEasing) => mapEasingToNativeEasing(segmentEasing, duration) || supportedWaapiEasing.easeOut);
	else return supportedWaapiEasing[easing];
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/start-waapi-animation.mjs
function startWaapiAnimation(element, valueName, keyframes, { delay = 0, duration = 300, repeat = 0, repeatType = "loop", ease = "easeOut", times } = {}, pseudoElement = void 0) {
	const keyframeOptions = { [valueName]: keyframes };
	if (times) keyframeOptions.offset = times;
	const easing = mapEasingToNativeEasing(ease, duration);
	/**
	* If this is an easing array, apply to keyframes, not animation as a whole
	*/
	if (Array.isArray(easing)) keyframeOptions.easing = easing;
	const options = {
		delay,
		duration,
		easing: !Array.isArray(easing) ? easing : "linear",
		fill: "both",
		iterations: repeat + 1,
		direction: repeatType === "reverse" ? "alternate" : "normal"
	};
	if (pseudoElement) options.pseudoElement = pseudoElement;
	return element.animate(keyframeOptions, options);
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/generators/utils/is-generator.mjs
function isGenerator(type) {
	return typeof type === "function" && "applyToOptions" in type;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/utils/apply-generator.mjs
function applyGeneratorOptions({ type, ...options }) {
	if (isGenerator(type) && supportsLinearEasing()) return type.applyToOptions(options);
	else {
		options.duration ?? (options.duration = 300);
		options.ease ?? (options.ease = "easeOut");
	}
	return options;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/NativeAnimation.mjs
/**
* NativeAnimation implements AnimationPlaybackControls for the browser's Web Animations API.
*/
var NativeAnimation = class extends WithPromise {
	constructor(options) {
		super();
		this.finishedTime = null;
		this.isStopped = false;
		/**
		* Tracks a manually-set start time that takes precedence over WAAPI's
		* dynamic startTime. This is cleared when play() or time setter is called,
		* allowing WAAPI to take over timing.
		*/
		this.manualStartTime = null;
		if (!options) return;
		const { element, name, keyframes, pseudoElement, allowFlatten = false, finalKeyframe, onComplete } = options;
		this.isPseudoElement = Boolean(pseudoElement);
		this.allowFlatten = allowFlatten;
		this.options = options;
		invariant(typeof options.type !== "string", `Mini animate() doesn't support "type" as a string.`, "mini-spring");
		const transition = applyGeneratorOptions(options);
		this.animation = startWaapiAnimation(element, name, keyframes, transition, pseudoElement);
		if (transition.autoplay === false) this.animation.pause();
		this.animation.onfinish = () => {
			this.finishedTime = this.time;
			if (!pseudoElement) {
				const keyframe = getFinalKeyframe(keyframes, this.options, finalKeyframe, this.speed);
				if (this.updateMotionValue) this.updateMotionValue(keyframe);
				/**
				* If we can, we want to commit the final style as set by the user,
				* rather than the computed keyframe value supplied by the animation.
				* We always do this, even when a motion value is present, to prevent
				* a visual flash in Firefox where the WAAPI animation's fill is removed
				* during cancel() before the scheduled render can apply the correct value.
				*/
				setStyle(element, name, keyframe);
				this.animation.cancel();
			}
			onComplete?.();
			this.notifyFinished();
		};
	}
	play() {
		if (this.isStopped) return;
		this.manualStartTime = null;
		this.animation.play();
		if (this.state === "finished") this.updateFinished();
	}
	pause() {
		this.animation.pause();
	}
	complete() {
		this.animation.finish?.();
	}
	cancel() {
		try {
			this.animation.cancel();
		} catch (e) {}
	}
	stop() {
		if (this.isStopped) return;
		this.isStopped = true;
		const { state } = this;
		if (state === "idle" || state === "finished") return;
		if (this.updateMotionValue) this.updateMotionValue();
		else this.commitStyles();
		if (!this.isPseudoElement) this.cancel();
	}
	/**
	* WAAPI doesn't natively have any interruption capabilities.
	*
	* In this method, we commit styles back to the DOM before cancelling
	* the animation.
	*
	* This is designed to be overridden by NativeAnimationExtended, which
	* will create a renderless JS animation and sample it twice to calculate
	* its current value, "previous" value, and therefore allow
	* Motion to also correctly calculate velocity for any subsequent animation
	* while deferring the commit until the next animation frame.
	*/
	commitStyles() {
		const element = this.options?.element;
		if (!this.isPseudoElement && element?.isConnected) this.animation.commitStyles?.();
	}
	get duration() {
		const duration = this.animation.effect?.getComputedTiming?.().duration || 0;
		return millisecondsToSeconds(Number(duration));
	}
	get iterationDuration() {
		const { delay = 0 } = this.options || {};
		return this.duration + millisecondsToSeconds(delay);
	}
	get time() {
		return millisecondsToSeconds(Number(this.animation.currentTime) || 0);
	}
	set time(newTime) {
		const wasFinished = this.finishedTime !== null;
		this.manualStartTime = null;
		this.finishedTime = null;
		this.animation.currentTime = secondsToMilliseconds(newTime);
		if (wasFinished) this.animation.pause();
	}
	/**
	* The playback speed of the animation.
	* 1 = normal speed, 2 = double speed, 0.5 = half speed.
	*/
	get speed() {
		return this.animation.playbackRate;
	}
	set speed(newSpeed) {
		if (newSpeed < 0) this.finishedTime = null;
		this.animation.playbackRate = newSpeed;
	}
	get state() {
		return this.finishedTime !== null ? "finished" : this.animation.playState;
	}
	get startTime() {
		return this.manualStartTime ?? Number(this.animation.startTime);
	}
	set startTime(newStartTime) {
		this.manualStartTime = this.animation.startTime = newStartTime;
	}
	/**
	* Attaches a timeline to the animation, for instance the `ScrollTimeline`.
	*/
	attachTimeline({ timeline, rangeStart, rangeEnd, observe }) {
		if (this.allowFlatten) this.animation.effect?.updateTiming({ easing: "linear" });
		this.animation.onfinish = null;
		if (timeline && supportsScrollTimeline()) {
			this.animation.timeline = timeline;
			if (rangeStart) this.animation.rangeStart = rangeStart;
			if (rangeEnd) this.animation.rangeEnd = rangeEnd;
			return noop;
		} else return observe(this);
	}
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/GroupAnimation.mjs
var GroupAnimation = class {
	constructor(animations) {
		this.stop = () => this.runAll("stop");
		this.animations = animations.filter(Boolean);
	}
	get finished() {
		return Promise.all(this.animations.map((animation) => animation.finished));
	}
	/**
	* TODO: Filter out cancelled or stopped animations before returning
	*/
	getAll(propName) {
		return this.animations[0][propName];
	}
	setAll(propName, newValue) {
		for (let i = 0; i < this.animations.length; i++) this.animations[i][propName] = newValue;
	}
	attachTimeline(timeline) {
		const subscriptions = this.animations.map((animation) => animation.attachTimeline(timeline));
		return () => {
			subscriptions.forEach((cancel, i) => {
				cancel && cancel();
				this.animations[i].stop();
			});
		};
	}
	get time() {
		return this.getAll("time");
	}
	set time(time) {
		this.setAll("time", time);
	}
	get speed() {
		return this.getAll("speed");
	}
	set speed(speed) {
		this.setAll("speed", speed);
	}
	get state() {
		return this.getAll("state");
	}
	get startTime() {
		return this.getAll("startTime");
	}
	get duration() {
		return getMax(this.animations, "duration");
	}
	get iterationDuration() {
		return getMax(this.animations, "iterationDuration");
	}
	runAll(methodName) {
		this.animations.forEach((controls) => controls[methodName]());
	}
	play() {
		this.runAll("play");
	}
	pause() {
		this.runAll("pause");
	}
	cancel() {
		this.runAll("cancel");
	}
	complete() {
		this.runAll("complete");
	}
};
function getMax(animations, propName) {
	let max = 0;
	for (let i = 0; i < animations.length; i++) {
		const value = animations[i][propName];
		if (value !== null && value > max) max = value;
	}
	return max;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/GroupAnimationWithThen.mjs
var GroupAnimationWithThen = class extends GroupAnimation {
	then(onResolve, _onReject) {
		return this.finished.finally(onResolve).then(() => {});
	}
};

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/utils/active-animations.mjs
const animationMaps = /* @__PURE__ */ new WeakMap();
const animationMapKey = (name, pseudoElement = "") => `${name}:${pseudoElement}`;
function getAnimationMap(element) {
	let map = animationMaps.get(element);
	if (!map) {
		map = /* @__PURE__ */ new Map();
		animationMaps.set(element, map);
	}
	return map;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/utils/resolve-transition.mjs
/**
* If `transition` has `inherit: true`, shallow-merge it with
* `parentTransition` (child keys win) and strip the `inherit` key.
* Otherwise return `transition` unchanged.
*/
function resolveTransition(transition, parentTransition) {
	if (transition?.inherit && parentTransition) {
		const { inherit: _, ...rest } = transition;
		return {
			...parentTransition,
			...rest
		};
	}
	return transition;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/utils/get-value-transition.mjs
function getValueTransition(transition, key) {
	const valueTransition = transition?.[key] ?? transition?.["default"] ?? transition;
	if (valueTransition !== transition) return resolveTransition(valueTransition, transition);
	return valueTransition;
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/border-radius.mjs
/**
* The four corner-radius longhands. Shared so the projection mixer, scale
* corrector, WAAPI px-value set and view-transition crop pass don't each carry
* their own copy. Order is irrelevant - every consumer mixes/corrects/animates
* each corner independently.
*/
const cornerRadiusProps = [
	"borderTopLeftRadius",
	"borderTopRightRadius",
	"borderBottomRightRadius",
	"borderBottomLeftRadius"
];

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/waapi/utils/px-values.mjs
const pxValues = new Set([
	"borderWidth",
	"borderTopWidth",
	"borderRightWidth",
	"borderBottomWidth",
	"borderLeftWidth",
	"borderRadius",
	...cornerRadiusProps,
	"width",
	"maxWidth",
	"height",
	"maxHeight",
	"top",
	"right",
	"bottom",
	"left",
	"inset",
	"insetBlock",
	"insetBlockStart",
	"insetBlockEnd",
	"insetInline",
	"insetInlineStart",
	"insetInlineEnd",
	"padding",
	"paddingTop",
	"paddingRight",
	"paddingBottom",
	"paddingLeft",
	"paddingBlock",
	"paddingBlockStart",
	"paddingBlockEnd",
	"paddingInline",
	"paddingInlineStart",
	"paddingInlineEnd",
	"margin",
	"marginTop",
	"marginRight",
	"marginBottom",
	"marginLeft",
	"marginBlock",
	"marginBlockStart",
	"marginBlockEnd",
	"marginInline",
	"marginInlineStart",
	"marginInlineEnd",
	"fontSize",
	"backgroundPositionX",
	"backgroundPositionY"
]);

//#endregion
//#region ../../node_modules/motion-dom/dist/es/animation/keyframes/utils/apply-px-defaults.mjs
function applyPxDefaults(keyframes, name) {
	for (let i = 0; i < keyframes.length; i++) if (typeof keyframes[i] === "number" && pxValues.has(name)) keyframes[i] = keyframes[i] + "px";
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/resolve-elements.mjs
function resolveElements(elementOrSelector, scope, selectorCache) {
	if (elementOrSelector == null) return [];
	if (elementOrSelector instanceof EventTarget) return [elementOrSelector];
	else if (typeof elementOrSelector === "string") {
		let root = document;
		if (scope) root = scope.current;
		const elements = selectorCache?.[elementOrSelector] ?? root.querySelectorAll(elementOrSelector);
		return elements ? Array.from(elements) : [];
	}
	return Array.from(elementOrSelector).filter((element) => element != null);
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/render/dom/style-computed.mjs
function getComputedStyle(element, name) {
	const computedStyle = window.getComputedStyle(element);
	return isCSSVar(name) ? computedStyle.getPropertyValue(name) : computedStyle[name];
}

//#endregion
//#region ../../node_modules/motion-dom/dist/es/utils/stagger.mjs
function getOriginIndex(from, total) {
	if (from === "first") return 0;
	else {
		const lastIndex = total - 1;
		return from === "last" ? lastIndex : lastIndex / 2;
	}
}
function stagger(duration = .1, { startDelay = 0, from = 0, ease } = {}) {
	return (i, total) => {
		const fromIndex = typeof from === "number" ? from : getOriginIndex(from, total);
		let delay = duration * Math.abs(fromIndex - i);
		if (ease) {
			const maxDelay = total * duration;
			delay = easingDefinitionToFunction(ease)(delay / maxDelay) * maxDelay;
		}
		return startDelay + delay;
	};
}

//#endregion
//#region ../../node_modules/framer-motion/dist/es/animation/animators/waapi/animate-elements.mjs
function animateElements(elementOrSelector, keyframes, options, scope) {
	if (elementOrSelector == null) return [];
	const elements = resolveElements(elementOrSelector, scope);
	const numElements = elements.length;
	invariant(Boolean(numElements), "No valid elements provided.", "no-valid-elements");
	/**
	* WAAPI doesn't support interrupting animations.
	*
	* Therefore, starting animations requires a three-step process:
	* 1. Stop existing animations (write styles to DOM)
	* 2. Resolve keyframes (read styles from DOM)
	* 3. Create new animations (write styles to DOM)
	*
	* The hybrid `animate()` function uses AsyncAnimation to resolve
	* keyframes before creating new animations, which removes style
	* thrashing. Here, we have much stricter filesize constraints.
	* Therefore we do this in a synchronous way that ensures that
	* at least within `animate()` calls there is no style thrashing.
	*
	* In the motion-native-animate-mini-interrupt benchmark this
	* was 80% faster than a single loop.
	*/
	const animationDefinitions = [];
	/**
	* Step 1: Build options and stop existing animations (write)
	*/
	for (let i = 0; i < numElements; i++) {
		const element = elements[i];
		const elementTransition = { ...options };
		/**
		* Resolve stagger function if provided.
		*/
		if (typeof elementTransition.delay === "function") elementTransition.delay = elementTransition.delay(i, numElements);
		for (const valueName in keyframes) {
			let valueKeyframes = keyframes[valueName];
			if (!Array.isArray(valueKeyframes)) valueKeyframes = [valueKeyframes];
			const valueOptions = { ...getValueTransition(elementTransition, valueName) };
			valueOptions.duration && (valueOptions.duration = secondsToMilliseconds(valueOptions.duration));
			valueOptions.delay && (valueOptions.delay = secondsToMilliseconds(valueOptions.delay));
			/**
			* If there's an existing animation playing on this element then stop it
			* before creating a new one.
			*/
			const map = getAnimationMap(element);
			const key = animationMapKey(valueName, valueOptions.pseudoElement || "");
			const currentAnimation = map.get(key);
			currentAnimation && currentAnimation.stop();
			animationDefinitions.push({
				map,
				key,
				unresolvedKeyframes: valueKeyframes,
				options: {
					...valueOptions,
					element,
					name: valueName,
					allowFlatten: !elementTransition.type && !elementTransition.ease
				}
			});
		}
	}
	/**
	* Step 2: Resolve keyframes (read)
	*/
	for (let i = 0; i < animationDefinitions.length; i++) {
		const { unresolvedKeyframes, options: animationOptions } = animationDefinitions[i];
		const { element, name, pseudoElement } = animationOptions;
		if (!pseudoElement && unresolvedKeyframes[0] === null) unresolvedKeyframes[0] = getComputedStyle(element, name);
		fillWildcards(unresolvedKeyframes);
		applyPxDefaults(unresolvedKeyframes, name);
		/**
		* If we only have one keyframe, explicitly read the initial keyframe
		* from the computed style. This is to ensure consistency with WAAPI behaviour
		* for restarting animations, for instance .play() after finish, when it
		* has one vs two keyframes.
		*/
		if (!pseudoElement && unresolvedKeyframes.length < 2) unresolvedKeyframes.unshift(getComputedStyle(element, name));
		animationOptions.keyframes = unresolvedKeyframes;
	}
	/**
	* Step 3: Create new animations (write)
	*/
	const animations = [];
	for (let i = 0; i < animationDefinitions.length; i++) {
		const { map, key, options: animationOptions } = animationDefinitions[i];
		const animation = new NativeAnimation(animationOptions);
		map.set(key, animation);
		animation.finished.finally(() => map.delete(key));
		animations.push(animation);
	}
	return animations;
}

//#endregion
//#region ../../node_modules/framer-motion/dist/es/animation/animators/waapi/animate-style.mjs
const createScopedWaapiAnimate = (scope) => {
	function scopedAnimate(elementOrSelector, keyframes, options) {
		return new GroupAnimationWithThen(animateElements(elementOrSelector, keyframes, options, scope));
	}
	return scopedAnimate;
};
const animateMini = /*@__PURE__*/ createScopedWaapiAnimate();

//#endregion
//#region ../../packages/sail/runtime/src/motion.ts
let reducedMedia = null;
function prefersReducedMotion() {
	if (typeof matchMedia === "undefined") return false;
	if (!reducedMedia) {
		const mql = matchMedia("(prefers-reduced-motion: reduce)");
		reducedMedia = mql;
		if (typeof mql.addEventListener === "function") mql.addEventListener("change", () => {});
	}
	return reducedMedia.matches;
}
const REST_VALUE = {
	opacity: 0,
	scale: .96,
	"scale-x": 0,
	"scale-y": 0,
	"translate-y": 16,
	"translate-x": 16,
	rotate: -6
};
const FINAL_VALUE = {
	opacity: 1,
	scale: 1,
	"scale-x": 1,
	"scale-y": 1,
	"translate-y": 0,
	"translate-x": 0,
	rotate: 0
};
/** Does this magnitude already carry its own unit? `'120px'` and `'-12%'`
*  do; `'120'`, `120` and `''` do not. */
function hasUnit(v) {
	return typeof v === "string" && v.trim() !== "" && Number.isNaN(Number(v));
}
/** Render a magnitude into a CSS length/angle. A unit-bearing string wins
*  verbatim; anything numeric gets the channel's native unit appended. This
*  is the ONE place a unit is decided, so `translate-x: { to: 120 }` and
*  `translate-x: { to: '120px' }` cannot drift apart. */
function withUnit(v, unit, fallback) {
	if (hasUnit(v)) return v.trim();
	const n = v === void 0 || v === "" ? fallback : Number(v);
	return `${Number.isNaN(n) ? fallback : n}${unit}`;
}
/** Render a magnitude for a UNITLESS channel (scale, opacity). A string that
*  smuggled in a unit here would produce an invalid keyframe, so it is
*  coerced back to a bare number. */
function withoutUnit(v, fallback) {
	if (v === void 0 || v === "") return fallback;
	const n = typeof v === "number" ? v : Number.parseFloat(v);
	return Number.isNaN(n) ? fallback : n;
}
function isHTMLElement(el) {
	return "style" in el;
}
function applyRestStyle(el, cfg) {
	if (!isHTMLElement(el)) return;
	const value = REST_VALUE[cfg.property];
	switch (cfg.property) {
		case "opacity":
			el.style.opacity = String(value);
			return;
		case "scale":
			el.style.scale = String(value);
			return;
		case "scale-x":
			el.style.scale = `${value} 1`;
			return;
		case "scale-y":
			el.style.scale = `1 ${value}`;
			return;
		case "translate-y":
			el.style.translate = `0 ${value}px`;
			return;
		case "translate-x":
			el.style.translate = `${value}px 0`;
			return;
		case "rotate":
			el.style.rotate = `${value}deg`;
			return;
	}
}
function applyFinalStyle(el, cfg) {
	if (!isHTMLElement(el)) return;
	const value = FINAL_VALUE[cfg.property];
	switch (cfg.property) {
		case "opacity":
			el.style.opacity = String(value);
			return;
		case "scale":
			el.style.scale = String(value);
			return;
		case "scale-x":
			el.style.scale = `${value} 1`;
			return;
		case "scale-y":
			el.style.scale = `1 ${value}`;
			return;
		case "translate-y":
			el.style.translate = "0 0";
			return;
		case "translate-x":
			el.style.translate = "0 0";
			return;
		case "rotate":
			el.style.rotate = "0deg";
			return;
	}
}
/** Reduced-motion policy for a whole `motions` set, in one place (every
*  branch below — loop/enter/reveal/scroll/event — used to duplicate this).
*
*  Ruling: translate/scale/rotate are vestibular-hazard motion under
*  `prefers-reduced-motion`, so they never PLAY. Every config in the set —
*  including a companion `opacity` config — is snapped straight to its
*  FINAL resting value: opacity to fully visible, translate/scale/rotate
*  to their neutral identity transform (0 / 1 / 0deg — no jump, since
*  that IS the rest pose). The element is visible immediately; nothing is
*  left at zero opacity or off-screen.
*
*  An earlier version of this also fired a real WAAPI opacity fade
*  (0 → 1) for the translate/scale/rotate case, so reduced-motion users
*  would still see a settle-in cue instead of an instant appearance.
*  Browser verification (Playwright, `reducedMotion: 'reduce'`) found
*  that starting a `element.animate()` opacity keyframe from 0 on a
*  reduced-motion page leaves Chromium's computed style for that
*  property stuck at the FROM value even after the animation reports
*  `finished` and the inline style is re-asserted — reproducible with a
*  bare native `el.animate()` call, independent of this runtime, so it
*  is a browser/engine interaction with the reduced-motion emulation,
*  not a bug in this function. The mandate's own ruling allows either
*  "opacity-only entries still visibly run OR content is simply fully
*  visible" — given the fade path's unreliability, this takes the
*  synchronous-snap branch of that OR unconditionally, which is
*  guaranteed correct and was already this codebase's pre-existing
*  behavior before the fade was attempted. */
function applyReducedMotionSettle(el, motions) {
	if (!isHTMLElement(el)) return;
	for (const m of motions) applyFinalStyle(el, m);
}
/** `x`/`y` are motion/mini's own animatable-value names — NOT CSS
*  properties. `motion/mini`'s `NativeAnimation` (unlike the full
*  `motion` build's `visualElement`) does not map them to a transform;
*  it hands the key straight to `element.animate({ x: [...] }, ...)`.
*  Browsers accept unknown WAAPI keyframe properties silently — no
*  error, no effect — so a `translate-x`/`translate-y` motion "ran"
*  forever at `iterations: Infinity` while moving nothing. The one CSS
*  property that actually moves an HTML element on both axes is
*  `translate` (distinct from the `transform` shorthand, same as
*  `rotate`/`scale` already used here), so every translate-x/y preset
*  resolves through it instead. */
/** The two ends of a translate channel, already rendered as CSS lengths so
*  a unit-bearing inline value (`'-12%'`) survives to the keyframe. */
function axisRange(cfg, el) {
	if (cfg.preset === "bounce") {
		if (cfg.property === "translate-y") return ["0px", `${-(el.getBoundingClientRect().height || el.offsetHeight || 8)}px`];
		if (cfg.property === "translate-x") return ["0px", `${-(el.getBoundingClientRect().width || el.offsetWidth || 8)}px`];
	}
	return [withUnit(cfg.from, "px", REST_VALUE[cfg.property]), withUnit(cfg.to, "px", FINAL_VALUE[cfg.property])];
}
function buildKeyframes(cfg, el) {
	if (cfg.property === "translate-y" || cfg.property === "translate-x") {
		const [from, to] = axisRange(cfg, el);
		return cfg.property === "translate-y" ? { translate: [`0px ${from}`, `0px ${to}`] } : { translate: [`${from} 0px`, `${to} 0px`] };
	}
	const from = withoutUnit(cfg.from, REST_VALUE[cfg.property]);
	const to = withoutUnit(cfg.to, FINAL_VALUE[cfg.property]);
	switch (cfg.property) {
		case "opacity": return { opacity: [from, to] };
		case "scale": return { scale: [from, to] };
		case "scale-x": return { scale: [`${from} 1`, `${to} 1`] };
		case "scale-y": return { scale: [`1 ${from}`, `1 ${to}`] };
		case "rotate": return { rotate: [withUnit(cfg.from, "deg", REST_VALUE.rotate), withUnit(cfg.to, "deg", FINAL_VALUE.rotate)] };
	}
}
/** Two motions on one element that both animate `translate` (one
*  `translate-x`, one `translate-y`) would otherwise fight: WAAPI has
*  exactly one `translate` property per element, and motion/mini's
*  interruption contract stops whichever animation on that property
*  was created first the moment the second is created (see
*  `animateElements`'s per-key animation map) — so the two `animate()`
*  calls the naive per-motion loop would issue don't compose, the
*  first one is cancelled before it ever plays. The honest fix is to
*  merge both axes into ONE `translate` keyframe track up front. */
function buildMergedTranslateKeyframes(xCfg, yCfg, el) {
	const [xFrom, xTo] = xCfg ? axisRange(xCfg, el) : ["0px", "0px"];
	const [yFrom, yTo] = yCfg ? axisRange(yCfg, el) : ["0px", "0px"];
	return { translate: [`${xFrom} ${yFrom}`, `${xTo} ${yTo}`] };
}
/** The full set of CSS properties this runtime's preset table can ever
*  emit as a WAAPI keyframe key. `SILENCE IS THE BUG`: a keyframe
*  property outside this set animates zero pixels/opacity/anything on
*  an HTML element — WAAPI accepts (and ignores) unknown keyframe
*  properties with no error, so a bug here would otherwise ship
*  invisibly, exactly like the `x`/`y` defect this guards against. */
const ANIMATABLE_KEYFRAME_PROPERTIES = new Set([
	"opacity",
	"scale",
	"translate",
	"rotate"
]);
function warnIfNotAnimatable(el, keyframes) {
	const supports = typeof CSS !== "undefined" && typeof CSS.supports === "function" ? CSS.supports : null;
	for (const prop of Object.keys(keyframes)) {
		if (!ANIMATABLE_KEYFRAME_PROPERTIES.has(prop)) {
			console.warn(`[sail/motion] keyframe property "${prop}" is not one of this runtime's known-animatable properties (${[...ANIMATABLE_KEYFRAME_PROPERTIES].join(", ")}) — WAAPI accepts unknown keyframe properties silently, so this animation will run forever and move nothing.`, el);
			continue;
		}
		if (!supports) continue;
		for (const value of keyframes[prop]) if (!supports(prop, String(value))) console.warn(`[sail/motion] keyframe value ${JSON.stringify(value)} is not valid for CSS property "${prop}" — the animation will silently no-op.`, el);
	}
}
/** SILENCE IS THE BUG, opacity edition: `CSS.supports('opacity', '1.04')`
*  passes — `1.04` is syntactically a valid `<number>` — but the
*  computed style clamps it to `1`, so an `opacity` motion authored
*  against a preset whose `from`/`to` live outside `0..1` (e.g. a scale
*  preset like `pulse`, 1.0 → 1.04, applied to the wrong property)
*  renders NO visible change at all: the keyframe is valid CSS, the
*  animation genuinely runs, and nothing moves. `warnIfNotAnimatable`'s
*  `CSS.supports` check cannot catch this — it only rejects malformed
*  values, and an out-of-range opacity number is well-formed. */
function warnIfOutOfVisibleRange(el, cfg, keyframes) {
	if (cfg.property !== "opacity") return;
	for (const value of keyframes.opacity ?? []) {
		const n = typeof value === "number" ? value : Number(value);
		if (Number.isFinite(n) && (n < 0 || n > 1)) {
			console.warn(`[sail/motion] opacity keyframe value ${JSON.stringify(value)} (from preset "${cfg.preset}") is outside the visible 0..1 range — the computed style clamps it, so this animation runs and moves nothing. "${cfg.preset}" looks like a preset authored for a different property (e.g. scale) — check the property this preset is attached to.`, el);
			return;
		}
	}
}
function buildTransition(cfg, delaySec) {
	const t = {};
	if (cfg.kind === "spring") {
		const springArgs = { keyframes: [0, 1] };
		if (cfg.stiffness != null) springArgs.stiffness = cfg.stiffness;
		if (cfg.damping != null) springArgs.damping = cfg.damping;
		if (cfg.mass != null) springArgs.mass = cfg.mass;
		if (cfg.restSpeed != null) springArgs.restSpeed = cfg.restSpeed;
		if (cfg.restDelta != null) springArgs.restDelta = cfg.restDelta;
		const generator = spring(springArgs);
		const durationMs = calcGeneratorDuration(generator);
		t.duration = durationMs / 1e3;
		t.ease = generateLinearEasing((s) => generator.next(s * durationMs).value, durationMs);
	} else {
		if (cfg.duration != null) t.duration = cfg.duration;
		if (cfg.easing) t.ease = cfg.easing;
	}
	if (cfg.iterations != null) t.repeat = cfg.iterations;
	if (cfg.direction != null) t.repeatType = cfg.direction === "normal" ? "loop" : cfg.direction === "alternate" ? "reverse" : cfg.direction;
	if (delaySec != null || cfg.delay != null) t.delay = (delaySec ?? 0) + (cfg.delay ?? 0);
	return t;
}
/** `stagger` describes the delay BETWEEN SIBLING ELEMENTS sharing a
*  trigger (the 5 dealt cards, the 3 thinking-dots) — never between two
*  PROPERTIES animated on the SAME element (an element's own
*  translate-y + opacity, both driven by one preset, must stay in
*  lockstep with each other, or the position and the fade visibly
*  desync every cycle). The compiler has no visibility into an
*  element's siblings at compile time (a `{#for}` list is
*  unknown-length; even a static run of sibling tags isn't tracked),
*  so the runtime assigns each element's phase index the moment it
*  attaches, keyed by (immediate parent, trigger) — a fresh parent (a
*  `{#if}`-gated container torn down and rebuilt, e.g. the "Deal
*  again" replay) naturally resets the count back to 0.
*
*  Previously this index/total pair was `(i, config.motions.length)` —
*  this element's OWN property list, not its sibling position — so a
*  single-property element (every card, every loop dot) always saw
*  `total === 1` and the `total > 1` guard silently zeroed every
*  delay: the whole group animated in lockstep. A multi-property
*  element (thinking-dots' paired translate-y + opacity) saw
*  `total === 2` instead, and its own two properties got staggered
*  AGAINST EACH OTHER — the flashing: position starts moving while
*  opacity is still delayed (or vice versa), every cycle. */
const staggerIndices = /* @__PURE__ */ new WeakMap();
function nextStaggerIndex(parent, trigger) {
	let byTrigger = staggerIndices.get(parent);
	if (!byTrigger) {
		byTrigger = /* @__PURE__ */ new Map();
		staggerIndices.set(parent, byTrigger);
	}
	const index = byTrigger.get(trigger) ?? 0;
	byTrigger.set(trigger, index + 1);
	return index;
}
/** Resolved ONCE per `__sail_motion_attach` call (per element, per
*  mount) — never recomputed on repeat triggers (hover in/out, replay)
*  — so an element's sibling phase is fixed for its lifetime. Motion's
*  own `stagger()` distance formula (`from: 0`, the default) is
*  `duration * index` regardless of `total`, so no sibling count needs
*  to be known in advance. */
function computeStaggerDelay(config, el) {
	if (config.stagger == null) return void 0;
	const parent = el.parentElement;
	if (!parent) return 0;
	const index = nextStaggerIndex(parent, config.trigger);
	return stagger(config.stagger)(index, index + 1);
}
/** Resume a freshly-created set of animations at `timeMs` — used when a
*  reactive config change re-creates an ambient loop and we do not want it
*  to visibly snap back to the start. Best-effort by construction: a driver
*  that does not expose a settable time simply plays from zero. */
function seekTo(el, timeMs) {
	if (timeMs == null) return;
	try {
		for (const a of el.getAnimations?.() ?? []) {
			const total = a.effect?.getTiming?.().duration ?? 0;
			a.currentTime = total > 0 ? timeMs % total : timeMs;
		}
	} catch {}
}
function createAnimations(el, config, delaySec) {
	const controls = [];
	const total = config.motions.length;
	const xIndex = config.motions.findIndex((m) => m.property === "translate-x");
	const yIndex = config.motions.findIndex((m) => m.property === "translate-y");
	const merged = /* @__PURE__ */ new Set();
	if (xIndex !== -1 && yIndex !== -1) {
		const xCfg = config.motions[xIndex];
		const yCfg = config.motions[yIndex];
		const keyframes = buildMergedTranslateKeyframes(xCfg, yCfg, el);
		warnIfNotAnimatable(el, keyframes);
		const leadIndex = Math.min(xIndex, yIndex);
		const transition = buildTransition(config.motions[leadIndex], delaySec);
		try {
			const c = animateMini(el, keyframes, transition);
			controls.push(c);
		} catch {
			applyFinalStyle(el, xCfg);
			applyFinalStyle(el, yCfg);
		}
		merged.add(xIndex);
		merged.add(yIndex);
	}
	for (let i = 0; i < total; i++) {
		if (merged.has(i)) continue;
		const cfg = config.motions[i];
		const keyframes = buildKeyframes(cfg, el);
		warnIfNotAnimatable(el, keyframes);
		warnIfOutOfVisibleRange(el, cfg, keyframes);
		const transition = buildTransition(cfg, delaySec);
		try {
			const c = animateMini(el, keyframes, transition);
			controls.push(c);
		} catch {
			applyFinalStyle(el, cfg);
		}
	}
	return controls;
}
/** Create a motion's animations once its element is part of a live
*  document, then return a disposer that cancels them.
*
*  A WAAPI animation created on a DETACHED element never resolves its
*  play task: it stays `pending` with `startTime === null`, frozen at
*  `currentTime` 0. Because Motion attaches with `fill: 'both'`, that
*  PINS the element to the first keyframe (e.g. `opacity: 0`) forever,
*  and the animation never surfaces in `getAnimations()` — a silent,
*  permanent freeze with no console error.
*
*  The compiler wires Style motion during detached subtree construction
*  (`el = __tpl(...); __sail_motion_attach(el, ...)`), so on first mount
*  the element is not yet in the document. We pin it to its rest pose
*  immediately (so it does not flash its final value for a frame) and
*  create the real animations on the frame after it joins the document.
*  Elements already connected at attach time animate synchronously,
*  exactly as before. Environments without `requestAnimationFrame`
*  (SSR, tests) fall through to the synchronous path. */
function createAnimationsWhenConnected(el, config, delaySec, carriedTimeMs = null, afterCreate) {
	if (el.isConnected || typeof requestAnimationFrame === "undefined") {
		const controls = createAnimations(el, config, delaySec);
		seekTo(el, carriedTimeMs);
		const extra = afterCreate?.(controls);
		return () => {
			extra?.();
			for (const c of controls) c.cancel();
		};
	}
	for (const m of config.motions) applyRestStyle(el, m);
	let controls = [];
	let extra;
	let disposed = false;
	const start = () => {
		if (disposed) return;
		if (!el.isConnected) {
			requestAnimationFrame(start);
			return;
		}
		controls = createAnimations(el, config, delaySec);
		seekTo(el, carriedTimeMs);
		extra = afterCreate?.(controls);
	};
	requestAnimationFrame(start);
	return () => {
		disposed = true;
		extra?.();
		for (const c of controls) c.cancel();
	};
}
function makeProgressTimeline(kind, el) {
	const g = globalThis;
	try {
		if (kind === "reveal") return g.ViewTimeline ? new g.ViewTimeline({ subject: el }) : void 0;
		if (typeof document === "undefined") return void 0;
		return g.ScrollTimeline ? new g.ScrollTimeline({ source: document.scrollingElement }) : void 0;
	} catch {
		return;
	}
}
/** No scroll timeline available. `reveal` still has a correct meaning —
*  play the reveal once, time-driven, the first time the element
*  intersects the viewport. `scroll` (progress = container scroll) has
*  no time-driven equivalent, so it lands on the END state — visible,
*  not invisible — and SAYS SO: this runtime's history is silent
*  degradations shipping as "works on my machine", so the degraded
*  path is loud by policy. */
function observeProgressFallback(el, kind, c) {
	if (kind === "reveal" && typeof IntersectionObserver === "function") {
		c.pause?.();
		const io = new IntersectionObserver((entries) => {
			if (entries.some((e) => e.isIntersecting)) {
				io.disconnect();
				c.play?.();
			}
		}, { threshold: .15 });
		io.observe(el);
		return () => io.disconnect();
	}
	console.warn(`[sail/motion] style.${kind} needs ${kind === "reveal" ? "ViewTimeline or IntersectionObserver" : "ScrollTimeline"}, and this environment has neither — settling at the final pose instead of animating. The trigger is degraded, not silently dropped.`, el);
	c.complete?.();
	return () => {};
}
/** Re-time a freshly created animation set onto its progress timeline.
*  Runs synchronously after `createAnimations` (via the `afterCreate`
*  hook), before the just-started time-driven animation can advance or
*  finish. Returns the disposer for whatever the fallback installed. */
function attachProgressTimeline(el, kind, controls) {
	const timeline = makeProgressTimeline(kind, el);
	const detachers = [];
	for (const c of controls) if (typeof c.attachTimeline === "function") detachers.push(c.attachTimeline({
		timeline,
		...kind === "reveal" && timeline ? {
			rangeStart: "entry 0%",
			rangeEnd: "entry 100%"
		} : {},
		observe: () => observeProgressFallback(el, kind, c)
	}));
	else detachers.push(observeProgressFallback(el, kind, c));
	return () => {
		for (const d of detachers) d();
	};
}
const TRIGGER_EVENTS = {
	hover: ["mouseenter", "mouseleave"],
	press: [
		"pointerdown",
		"pointerup",
		"pointercancel"
	],
	focus: ["focus", "blur"],
	"focus-visible": ["focusin", "focusout"]
};
let focusSource = "unknown";
let focusTrackerInstalled = false;
function trackFocusSource() {
	if (focusTrackerInstalled || typeof window === "undefined") return;
	focusTrackerInstalled = true;
	window.addEventListener("keydown", () => {
		focusSource = "keyboard";
	}, { capture: true });
	window.addEventListener("mousedown", () => {
		focusSource = "mouse";
	}, { capture: true });
}
function isFocusVisible(ev) {
	const target = ev.target;
	if (target && typeof target.matches === "function") try {
		return target.matches(":focus-visible");
	} catch {}
	return focusSource === "keyboard";
}
function classifyTrigger(trigger) {
	if (trigger === "loop") return "loop";
	if (trigger === "enter" || trigger === "exit") return trigger;
	if (trigger === "reveal" || trigger === "scroll") return trigger;
	if (TRIGGER_EVENTS[trigger]) return "event";
	return "noop";
}
/** Attach Style motion (`style.<trigger>({…})`).
*
*  The compiler emits a PLAIN OBJECT when every field of the config is a
*  literal, and a THUNK when any field holds a signal. The thunk form is
*  the reactive lane:
*
*      style={style.loop({ translate-x: { to: dist() }, duration: speed() })}
*
*  ## Reactive semantics (one rule, deliberately)
*
*  A config-field signal change TEARS DOWN and RE-CREATES the animation
*  with the new values. This holds for every field class — timing
*  (duration/delay/easing) and keyframe (from/to) alike — because WAAPI
*  offers no coherent way to retime or re-key a running animation: the
*  honest primitive is a new animation. Re-creation is the semantics, not
*  an implementation detail, so it is the same regardless of which field
*  moved and there is no per-field rule for an author to memorise.
*
*  For an INFINITE loop the playback position is carried across the
*  re-creation where the driver exposes it, so an ambient loop does not
*  visibly snap back to its start when an unrelated signal changes. A
*  finite animation (enter/exit/hover) restarts from zero, which is what a
*  one-shot whose target just changed should do.
*
*  The whole thing is effect-tracked, so it follows every signal the config
*  reads, and the returned disposer stops the effect AND cancels the live
*  animations.
*
*  ## The seam with `timeline()`
*
*  A signal here is reactive CONFIG — it says what the animation IS, and
*  changing it makes a new animation. A signal driving a CONTINUOUS,
*  per-frame value is the other lane: `timeline()` + `(--x)` reflection.
*  Do not use Style motion to scrub a value every frame; each change here
*  costs an animation teardown. */
function __sail_motion_attach(el, config) {
	if (typeof config !== "function") return attachOnce(el, config);
	let inner = null;
	let carried = null;
	const stop = effect(() => {
		const next = config();
		untrack(() => {
			if (inner) {
				carried = readPlaybackTime(el, next);
				inner();
				inner = null;
			}
			inner = attachOnce(el, next, carried);
		});
	});
	return () => {
		stop();
		if (inner) inner();
		inner = null;
	};
}
/** The playback position to carry across a re-creation, or `null` to start
*  from zero. Only infinite loops carry — a finite animation whose target
*  just changed should play the new one from the top. */
function readPlaybackTime(el, cfg) {
	if (!(cfg.motions.some((m) => m.iterations === Infinity) || cfg.trigger === "loop")) return null;
	try {
		const anims = el.getAnimations?.() ?? [];
		for (const a of anims) {
			const t = a.currentTime;
			if (typeof t === "number" && Number.isFinite(t)) return t;
		}
	} catch {}
	return null;
}
function attachOnce(el, config, carriedTimeMs = null) {
	if (!isHTMLElement(el)) return () => {};
	const reduced = prefersReducedMotion();
	const branch = classifyTrigger(config.trigger);
	if (branch === "noop") return () => {};
	const delaySec = computeStaggerDelay(config, el);
	if (branch === "loop") {
		if (reduced) {
			applyReducedMotionSettle(el, config.motions);
			return () => {};
		}
		return createAnimationsWhenConnected(el, {
			...config,
			motions: config.motions.map((m) => ({
				...m,
				iterations: m.iterations ?? Infinity,
				direction: m.direction ?? "alternate"
			}))
		}, delaySec, carriedTimeMs);
	}
	if (branch === "enter" || branch === "exit") {
		installLifecycleRunner();
		if (branch === "enter") {
			if (reduced) {
				applyReducedMotionSettle(el, config.motions);
				return () => {};
			}
			return createAnimationsWhenConnected(el, config, delaySec);
		}
		if (reduced) return () => {};
		EXITS.set(el, {
			config,
			controls: [],
			delaySec
		});
		return () => {
			const entry = EXITS.get(el);
			EXITS.delete(el);
			if (entry) for (const c of entry.controls) c.cancel();
		};
	}
	if (branch === "reveal" || branch === "scroll") {
		if (reduced) {
			applyReducedMotionSettle(el, config.motions);
			return () => {};
		}
		return createAnimationsWhenConnected(el, config, delaySec, null, (controls) => attachProgressTimeline(el, branch, controls));
	}
	const events = TRIGGER_EVENTS[config.trigger];
	if (config.trigger === "focus-visible") trackFocusSource();
	const [enterEvent, leaveEvent, cancelEvent] = events;
	let live = [];
	const playForward = () => {
		if (prefersReducedMotion()) return;
		for (const c of live) c.cancel();
		live = createAnimations(el, config, delaySec);
	};
	const onEnter = (ev) => {
		if (config.trigger === "focus-visible" && !isFocusVisible(ev)) return;
		playForward();
	};
	const onLeave = () => {
		playForward();
	};
	el.addEventListener(enterEvent, onEnter);
	el.addEventListener(leaveEvent, onLeave);
	if (cancelEvent) el.addEventListener(cancelEvent, onLeave);
	return () => {
		el.removeEventListener(enterEvent, onEnter);
		el.removeEventListener(leaveEvent, onLeave);
		if (cancelEvent) el.removeEventListener(cancelEvent, onLeave);
		for (const c of live) try {
			c.cancel();
		} catch {}
	};
}
const EXITS = /* @__PURE__ */ new WeakMap();
let runnerInstalled = false;
/** Upper bound on how long `runLeave` will hold a node in the DOM for its
*  exit animation. A broken/never-settling animation (a bad easing curve,
*  a preset with an unreachable rest state, a test double that never
*  resolves `finished`) must not wedge removal forever — after this many
*  ms the animation is cancelled, the final pose is snapped on, and
*  removal proceeds regardless of playback state. 4000ms is generous
*  headroom over the slowest shipped preset (`spring:deliberate`, well
*  under 1s to rest) while still being a bounded, user-perceptible cap
*  rather than an unbounded wait. */
const EXIT_TIMEOUT_MS = 4e3;
function installLifecycleRunner() {
	if (runnerInstalled) return;
	runnerInstalled = true;
	setTransitionRunner({
		runEnter: () => null,
		runLeave: (nodes) => {
			const matched = [];
			for (const n of nodes) {
				if (!(n instanceof Element)) continue;
				const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll("*")].filter((e) => EXITS.has(e));
				for (const t of targets) {
					const entry = EXITS.get(t);
					if (entry) matched.push({
						el: t,
						entry
					});
				}
			}
			if (matched.length === 0) return null;
			if (prefersReducedMotion()) {
				for (const { el, entry } of matched) for (const m of entry.config.motions) applyFinalStyle(el, m);
				return null;
			}
			const playing = [];
			for (const { el, entry } of matched) {
				const controls = createAnimations(el, entry.config, entry.delaySec);
				entry.controls = controls;
				for (const c of controls) playing.push(c.finished.catch(() => {}));
			}
			let timeoutId;
			return Promise.race([Promise.all(playing).then(() => {}), new Promise((resolve) => {
				timeoutId = setTimeout(resolve, EXIT_TIMEOUT_MS);
			})]).then(() => {
				clearTimeout(timeoutId);
				for (const { el, entry } of matched) {
					for (const c of entry.controls) c.cancel();
					for (const m of entry.config.motions) applyFinalStyle(el, m);
					entry.controls = [];
				}
			});
		},
		cancelTransitions: (node) => {
			if (!(node instanceof Element)) return;
			const entry = EXITS.get(node);
			if (entry) {
				for (const c of entry.controls) c.cancel();
				entry.controls = [];
			}
		}
	});
}

//#endregion
//#region ../../packages/sail/runtime/src/actions/icon.ts
const ENDPOINT = "/_deck/icons/";
const cache = /* @__PURE__ */ new Map();
const inflight = /* @__PURE__ */ new Map();
function fetchKey(url, key) {
	const pending = inflight.get(key);
	if (pending) return pending;
	const p = fetch(url).then((r) => r.ok ? r.text() : null).catch(() => null).then((svg) => {
		cache.set(key, svg);
		inflight.delete(key);
		return svg;
	});
	inflight.set(key, p);
	return p;
}
/**
* `src=` allowlist: relative paths, same-origin absolute URLs, and
* data/blob SVG only. Cross-origin http(s) is always rejected (no CDN).
*/
function isAllowedIconSrc(url) {
	const u = url.trim();
	if (!u) return false;
	if (u.startsWith("//")) return false;
	if (u.startsWith("/") || u.startsWith("./") || u.startsWith("../")) return true;
	if (u.startsWith("blob:")) return true;
	if (u.startsWith("data:image/svg")) return true;
	if (!/^https?:\/\//i.test(u)) return !u.includes(":");
	try {
		const base = typeof globalThis !== "undefined" && "location" in globalThis && globalThis.location?.href ? globalThis.location.href : "http://localhost/";
		const parsed = new URL(u, base);
		if (typeof globalThis !== "undefined" && "location" in globalThis) {
			const loc = globalThis.location;
			if (loc?.origin && parsed.origin === loc.origin) return true;
		}
		return false;
	} catch {
		return false;
	}
}
function parseSvg(doc, markup) {
	const root = new DOMParser().parseFromString(markup, "image/svg+xml").documentElement;
	if (root.nodeName.toLowerCase() !== "svg") return null;
	return doc.adoptNode(root);
}
const icon = (node) => {
	let svgEl = null;
	let lastKey = null;
	let seq = 0;
	let spinDispose = null;
	const syncAria = () => {
		const label = node.getAttribute("label") ?? "";
		if (label) {
			node.setAttribute("role", "img");
			node.setAttribute("aria-label", label);
			node.removeAttribute("aria-hidden");
		} else {
			node.setAttribute("aria-hidden", "true");
			node.removeAttribute("role");
			node.removeAttribute("aria-label");
		}
		if (svgEl) if (label) svgEl.removeAttribute("aria-hidden");
		else svgEl.setAttribute("aria-hidden", "true");
	};
	const syncSpin = () => {
		spinDispose?.();
		spinDispose = null;
		if (!node.hasAttribute("spin")) return;
		spinDispose = __sail_motion_attach(node, {
			trigger: "loop",
			motions: [{
				property: "rotate",
				preset: "spin",
				kind: "tween",
				from: 0,
				to: 360,
				duration: .7,
				easing: "linear",
				iterations: Infinity,
				direction: "normal"
			}]
		});
	};
	const drop = () => {
		if (svgEl) {
			svgEl.remove();
			svgEl = null;
		}
		lastKey = null;
	};
	const inject = (markup, key) => {
		if (key === lastKey) return;
		const parsed = parseSvg(node.ownerDocument, markup);
		if (!parsed) return;
		if (svgEl) svgEl.replaceWith(parsed);
		else node.insertBefore(parsed, node.firstChild);
		svgEl = parsed;
		lastKey = key;
		syncAria();
	};
	const render = () => {
		syncAria();
		syncSpin();
		const src = node.getAttribute("src") ?? "";
		const name = node.getAttribute("name") ?? "";
		if (name && !src) {
			const key = `name:${name}`;
			let cached = cache.get(key);
			if (cached === void 0) cached = cache.get(name);
			if (cached !== void 0) {
				if (cached) inject(cached, key);
				else drop();
				return;
			}
			const ticket = ++seq;
			fetchKey(`${ENDPOINT}${encodeURIComponent(name)}.svg`, key).then((svg) => {
				if (ticket !== seq) return;
				if (svg) inject(svg, key);
				else drop();
			});
			return;
		}
		if (!src) {
			drop();
			return;
		}
		if (!isAllowedIconSrc(src)) {
			drop();
			return;
		}
		const key = `src:${src}`;
		const cached = cache.get(key);
		if (cached !== void 0) {
			if (cached) inject(cached, key);
			else drop();
			return;
		}
		const ticket = ++seq;
		fetchKey(src, key).then((svg) => {
			if (ticket !== seq) return;
			if (svg) inject(svg, key);
			else drop();
		});
	};
	const mo = new MutationObserver(render);
	mo.observe(node, {
		attributes: true,
		attributeFilter: [
			"name",
			"src",
			"label",
			"spin"
		]
	});
	render();
	return { destroy: () => {
		mo.disconnect();
		spinDispose?.();
		spinDispose = null;
	} };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/dismiss-self.ts
const dismissed = /* @__PURE__ */ new WeakSet();
/** True after a successful dismissSelf (or if the node is already detached). */
function isDismissed(el) {
	return dismissed.has(el) || !el.isConnected;
}
/**
* Emit a composed dismiss event then remove the element. Idempotent: a second
* call on the same node is a no-op.
*/
function dismissSelf(el, eventName = "sail-toast-dismiss") {
	if (dismissed.has(el)) return false;
	dismissed.add(el);
	el.dispatchEvent(new CustomEvent(eventName, {
		bubbles: true,
		composed: true
	}));
	el.remove();
	return true;
}

//#endregion
//#region ../../packages/sail/runtime/src/actions/auto-dismiss.ts
function readDuration(el) {
	const raw = el.getAttribute("duration");
	if (raw == null || raw === "") return 5e3;
	const n = Number(raw);
	return Number.isFinite(n) ? n : 5e3;
}
/**
* Auto-dismiss after `duration` ms (attr). 0 / negative = sticky (no timer).
* Pause while hovered or focused so a reader is never cut off mid-read.
*/
const autoDismiss = (node) => {
	const el = node;
	const duration = readDuration(el);
	let timer;
	const fire = () => {
		if (isDismissed(el)) return;
		if (timer) {
			clearTimeout(timer);
			timer = void 0;
		}
		dismissSelf(el);
	};
	const start = () => {
		if (isDismissed(el) || duration <= 0) return;
		if (timer) clearTimeout(timer);
		timer = setTimeout(fire, duration);
	};
	const pause = () => {
		if (timer) clearTimeout(timer);
		timer = void 0;
	};
	el.addEventListener("pointerenter", pause);
	el.addEventListener("pointerleave", start);
	el.addEventListener("focusin", pause);
	el.addEventListener("focusout", start);
	start();
	return { destroy: () => {
		pause();
		el.removeEventListener("pointerenter", pause);
		el.removeEventListener("pointerleave", start);
		el.removeEventListener("focusin", pause);
		el.removeEventListener("focusout", start);
	} };
};

//#endregion
//#region ../../packages/sail/runtime/src/actions/dismiss-control.ts
const CLOSE_SEL = "[data-toast-close], [data-dismiss], button[aria-label=\"Dismiss\"]";
function isToastHost(el) {
	return el.hasAttribute("duration") || el.getAttribute("data-sail-host") === "toast";
}
/**
* Click on a descendant close control dismisses the host: composed dismiss
* event, then `el.remove()`. Toast → `sail-toast-dismiss` (no dismissible
* gate). Alert / other → `sail-dismiss`, only while `[dismissible]`.
*/
const dismissControl = (node) => {
	const el = node;
	const onClick = (e) => {
		if (isDismissed(el)) return;
		const close = e.target?.closest(CLOSE_SEL);
		if (!close || !el.contains(close)) return;
		const toast = isToastHost(el);
		if (!toast && !el.hasAttribute("dismissible")) return;
		dismissSelf(el, toast ? "sail-toast-dismiss" : "sail-dismiss");
	};
	el.addEventListener("click", onClick);
	return { destroy: () => {
		el.removeEventListener("click", onClick);
	} };
};

//#endregion
//#region ../../packages/sail/tokens/src/generated/themes-meta.ts
const THEME_META = [
	{
		id: "8bit",
		dark: true,
		modes: ["dark"],
		defaultMode: "dark"
	},
	{
		id: "brut",
		dark: false,
		modes: ["light"],
		defaultMode: "light"
	},
	{
		id: "console",
		dark: true,
		modes: ["dark"],
		defaultMode: "dark"
	},
	{
		id: "contrast",
		dark: true,
		modes: ["dark"],
		defaultMode: "dark"
	},
	{
		id: "glass",
		dark: false,
		modes: ["light"],
		defaultMode: "light"
	},
	{
		id: "halo",
		dark: false,
		modes: ["light"],
		defaultMode: "light"
	},
	{
		id: "my-theme-s99n",
		dark: true,
		modes: ["dark"],
		defaultMode: "dark"
	},
	{
		id: "my-theme-vbtk",
		dark: false,
		modes: ["light"],
		defaultMode: "light"
	},
	{
		id: "neutral",
		dark: false,
		modes: ["light", "dark"],
		defaultMode: "light"
	},
	{
		id: "paper",
		dark: false,
		modes: ["light"],
		defaultMode: "light"
	},
	{
		id: "sail",
		dark: false,
		modes: ["light", "dark"],
		defaultMode: "light"
	},
	{
		id: "ship",
		dark: false,
		modes: ["light", "dark"],
		defaultMode: "light"
	},
	{
		id: "studio",
		dark: true,
		modes: ["light", "dark"],
		defaultMode: "dark"
	},
	{
		id: "synthwave",
		dark: true,
		modes: ["dark"],
		defaultMode: "dark"
	}
];
const BY_ID = Object.fromEntries(THEME_META.map((t) => [t.id, t]));
function themeMeta(id) {
	return BY_ID[id];
}

//#endregion
//#region ../../packages/sail/tokens/src/themes.ts
/** Hand labels + swatches keyed by generated ThemeId. Meta (modes/dark) is generated. */
const THEME_PRESENTATION = {
	sail: {
		label: "Sail",
		tier: "core",
		swatches: [
			"oklch(0.985 0.008 250)",
			"oklch(1 0 0)",
			"oklch(0.58 0.18 245)",
			"oklch(0.62 0.14 195)"
		]
	},
	ship: {
		label: "SHIP",
		tier: "catalog",
		swatches: [
			"oklch(0.99 0.001 106.423)",
			"oklch(1 0 0)",
			"oklch(0.67 0.16 58)",
			"oklch(0.61 0.23 303)"
		]
	},
	neutral: {
		label: "Neutral",
		tier: "core",
		swatches: [
			"oklch(0.985 0.002 264)",
			"oklch(1 0 0)",
			"oklch(0.27 0.020 264)",
			"oklch(0.55 0.090 255)"
		]
	},
	paper: {
		label: "Paper",
		tier: "catalog",
		swatches: [
			"oklch(0.97 0.014 80)",
			"oklch(0.99 0.010 80)",
			"oklch(0.55 0.14 35)",
			"oklch(0.48 0.09 55)"
		]
	},
	halo: {
		label: "Halo (Soft)",
		tier: "catalog",
		swatches: [
			"oklch(0.92 0.008 230)",
			"oklch(0.92 0.008 230)",
			"oklch(0.62 0.14 50)",
			"oklch(0.55 0.05 230)"
		]
	},
	console: {
		label: "Console",
		tier: "catalog",
		swatches: [
			"oklch(0.13 0.008 220)",
			"oklch(0.16 0.010 220)",
			"oklch(0.78 0.20 145)",
			"oklch(0.78 0.16 195)"
		]
	},
	contrast: {
		label: "High Contrast",
		tier: "catalog",
		swatches: [
			"oklch(0.09 0 0)",
			"oklch(0.15 0 0)",
			"oklch(0.87 0.18 85)",
			"oklch(0.99 0 0)"
		]
	},
	glass: {
		label: "Glass",
		tier: "catalog",
		swatches: [
			"oklch(0.95 0.04 230)",
			"oklch(0.99 0.005 240)",
			"oklch(0.62 0.16 50)",
			"oklch(0.62 0.14 280)"
		]
	},
	brut: {
		label: "Brutalist",
		tier: "catalog",
		swatches: [
			"oklch(0.97 0 0)",
			"oklch(1 0 0)",
			"oklch(0.14 0 0)",
			"oklch(0.62 0.24 28)"
		]
	},
	"8bit": {
		label: "8-Bit",
		tier: "catalog",
		swatches: [
			"oklch(0.18 0.05 265)",
			"oklch(0.24 0.06 265)",
			"oklch(0.72 0.21 25)",
			"oklch(0.82 0.20 95)"
		]
	},
	synthwave: {
		label: "Synthwave",
		tier: "catalog",
		swatches: [
			"oklch(0.16 0.06 285)",
			"oklch(0.22 0.08 285)",
			"oklch(0.65 0.28 340)",
			"oklch(0.75 0.22 190)"
		]
	}
};
function buildThemes() {
	return THEME_META.map((meta) => {
		const pres = THEME_PRESENTATION[meta.id] ?? {
			label: meta.id.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" "),
			tier: "catalog",
			swatches: [
				"oklch(0.95 0 0)",
				"oklch(1 0 0)",
				"oklch(0.5 0.1 250)",
				"oklch(0.55 0.1 280)"
			]
		};
		const modes = meta.modes.length > 0 ? meta.modes : ["light", "dark"];
		return {
			id: meta.id,
			label: pres.label,
			dark: meta.dark,
			tier: pres.tier,
			swatches: pres.swatches,
			modes: [...modes],
			defaultMode: meta.defaultMode ?? "light"
		};
	});
}
const THEMES = buildThemes();
const THEME_BY_ID = Object.fromEntries(THEMES.map((t) => [t.id, t]));

//#endregion
//#region ../../packages/sail/runtime/src/actions/menu.ts
const host = (name) => `[data-sail-host="${name}"]`;
const isOpen = (el) => {
	try {
		return el.matches(":popover-open");
	} catch {
		return false;
	}
};
/**
* Internal: L1 `<dropdown>` / `<context-menu>` closed behavior (not a public `use:`).
* Authors compose `<dropdown>` / `<context-menu>` + `<menu-item>`, not `use:menu`.
*
* Wires role=menu / menuitem, roving focus (ArrowUp/Down, Home/End), Escape to
* close, focuses the first item on open, and closes after activating an item.
* Positioning is `use:anchor`'s job; open/close and light-dismiss are the
* native popover's.
*/
const menu = (node) => {
	const root = node;
	const itemSel = host("menu-item");
	const itemsOf = () => Array.from(root.querySelectorAll(itemSel)).filter((i) => !i.hasAttribute("disabled"));
	if (!root.hasAttribute("role")) root.setAttribute("role", "menu");
	const sync = () => {
		for (const i of root.querySelectorAll(itemSel)) {
			i.setAttribute("role", "menuitem");
			if (!i.hasAttribute("tabindex")) i.setAttribute("tabindex", "-1");
			if (i.hasAttribute("disabled")) i.setAttribute("aria-disabled", "true");
		}
	};
	const close = () => {
		try {
			root.hidePopover?.();
		} catch {}
	};
	const onKeydown = (e) => {
		const list = itemsOf();
		if (!list.length) return;
		const cur = list.indexOf(document.activeElement);
		let j = -1;
		if (e.key === "ArrowDown") j = cur < 0 ? 0 : (cur + 1) % list.length;
		else if (e.key === "ArrowUp") j = cur < 0 ? list.length - 1 : (cur - 1 + list.length) % list.length;
		else if (e.key === "Home") j = 0;
		else if (e.key === "End") j = list.length - 1;
		else if (e.key === "Escape") {
			close();
			return;
		}
		if (j >= 0) {
			e.preventDefault();
			list[j].focus();
		}
	};
	const onClick = (e) => {
		const item = e.target?.closest(itemSel);
		if (!item || !root.contains(item)) return;
		if (item.hasAttribute("disabled")) {
			e.preventDefault();
			e.stopPropagation();
			return;
		}
		root.dispatchEvent(new CustomEvent("sail-menu-select", {
			detail: { value: item.getAttribute("value") ?? item.textContent?.trim() ?? "" },
			bubbles: true,
			composed: true
		}));
		close();
	};
	const onToggle = () => {
		if (isOpen(root)) itemsOf()[0]?.focus();
	};
	root.addEventListener("keydown", onKeydown);
	root.addEventListener("click", onClick);
	root.addEventListener("toggle", onToggle);
	const mo = new MutationObserver(() => sync());
	mo.observe(root, {
		childList: true,
		subtree: true
	});
	sync();
	return { destroy: () => {
		root.removeEventListener("keydown", onKeydown);
		root.removeEventListener("click", onClick);
		root.removeEventListener("toggle", onToggle);
		mo.disconnect();
	} };
};

//#endregion
//#region src/edit/store.ts
let state$2 = emptyState$2();
function emptyState$2() {
	return {
		projectRoot: "",
		filter: "",
		files: [],
		activePath: null,
		view: "source",
		docs: /* @__PURE__ */ new Map(),
		modelRev: 0,
		selectionNodeId: null,
		sourceScrollLine: null,
		sourceScrollCol: null,
		sourceScrollNonce: 0,
		hoverTip: {
			nodeId: "",
			label: "",
			visible: false
		},
		threads: [],
		autoFixMode: "off",
		error: null,
		loading: false
	};
}

//#endregion
//#region src/checks/deeplink.ts
const DEFAULTS = {
	vscode: "vscode://file/{path}:{line}:{col}",
	cursor: "cursor://file/{path}:{line}:{col}",
	zed: "zed://file/{path}:{line}:{col}"
};
function defaultEditorSetting(scheme = "vscode") {
	const s = scheme.toLowerCase();
	return {
		scheme: s,
		template: DEFAULTS[s] ?? DEFAULTS.vscode
	};
}

//#endregion
//#region src/checks/store.ts
let state$1 = emptyState$1();
function emptyState$1() {
	return {
		projectRoot: "",
		findings: [],
		selectedId: null,
		ruleFilter: null,
		fileFilter: null,
		autoFixMode: "off",
		editor: defaultEditorSetting("vscode"),
		layoutBothViewportsRan: false,
		errorCount: 0,
		warningCount: 0,
		statusNote: "All clear",
		loading: false,
		error: null,
		sourceByPath: /* @__PURE__ */ new Map(),
		autoFixAttributed: "project setting",
		reviewOpen: false,
		reviewItems: [],
		reviewNote: "",
		lastRunAt: null,
		lastCheckedPath: null,
		lastLayoutChecked: false,
		attributions: [],
		collapsedRules: []
	};
}

//#endregion
//#region src/tasks/store.ts
let state = emptyState();
function emptyState() {
	return {
		projectRoot: "",
		threads: [],
		selectedId: null,
		selected: null,
		openFilter: "open",
		actorFilter: "all",
		driftFilter: "all",
		replyDraft: "",
		loading: false,
		error: null,
		commitPath: null,
		lastBatchCount: 0
	};
}

//#endregion
//#region src/shell/shell-rail.ts
/** Multi-app mount base (e.g. `/_studio`) so rail links stay in-app. */
function appRouteBase() {
	if (typeof document === "undefined") return "";
	const raw = document.querySelector("meta[name=\"deck-app-route\"]")?.getAttribute("content")?.trim();
	if (!raw || raw === "/") return "";
	return raw.replace(/\/$/, "");
}

//#endregion
//#region src/theme-session/theme-confirm.ts
let confirmFn = (message) => {
	if (typeof window === "undefined" || typeof window.confirm !== "function") return true;
	return window.confirm(message);
};
function themeConfirm(message) {
	return confirmFn(message);
}
function confirmDeleteTheme(name) {
	return themeConfirm(`Delete project theme “${name}”? This removes the theme JSON from the project and cannot be undone.`);
}
function confirmAddToProject(sourceName, newId) {
	return themeConfirm(`Add “${sourceName}” to the project as “${newId}”?`);
}
/** New theme from home — not “Add catalog pack” wording. */
function confirmNewTheme(sourceName, newId) {
	return themeConfirm(`Create new project theme “${newId}” based on “${sourceName}”?`);
}
function confirmRemixTheme(sourceName, newId) {
	return themeConfirm(`Remix “${sourceName}” into a new project theme “${newId}”?`);
}

//#endregion
//#region src/theme-session/theme-paint.ts
/**
* Theme paint helpers — mode tables, display names, location id.
* Pure functions used by createThemeSession (no store side effects).
*/
const SAFE_PAINT_FALLBACK = "studio";
function modesForPaintId(paintId) {
	const meta = themeMeta(paintId);
	const reg = THEMES.find((t) => t.id === paintId);
	const modes = (meta?.modes?.length ? meta.modes : reg?.modes?.length ? reg.modes : ["light", "dark"]).filter((m) => m === "light" || m === "dark");
	const list = modes.length ? modes : ["light"];
	const defaultMode = (meta?.defaultMode ?? reg?.defaultMode ?? list[0]) === "dark" && list.includes("dark") ? "dark" : list.includes("light") ? "light" : list[0];
	const dual = list.includes("light") && list.includes("dark");
	return {
		modes: list,
		defaultMode,
		dual,
		modesLabel: dual ? "Light · Dark" : list[0] === "dark" ? "Dark only" : "Light only"
	};
}
/**
* Registered catalog id for interim table paint (when applyScope has no draft doc).
* Prefer: exact id → forkedFrom (store) → my- strip exact match → safe studio + warning.
* Never substring-heuristic matches (silent wrong parent).
*/
function resolvePaintId(id, forkedFrom) {
	if (!id) return {
		paintId: SAFE_PAINT_FALLBACK,
		warning: "No theme id for paint preview; using studio table."
	};
	if (THEMES.some((t) => t.id === id)) return { paintId: id };
	if (forkedFrom && THEMES.some((t) => t.id === forkedFrom)) return { paintId: forkedFrom };
	const stripped = id.replace(/^my-/, "").replace(/-theme-[a-z0-9]+$/i, "");
	if (stripped && THEMES.some((t) => t.id === stripped)) return { paintId: stripped };
	const stampStrip = id.replace(/^my-/, "").replace(/-[a-z0-9]{3,6}$/i, "");
	if (stampStrip && stampStrip !== stripped && THEMES.some((t) => t.id === stampStrip)) return { paintId: stampStrip };
	return {
		paintId: SAFE_PAINT_FALLBACK,
		warning: `No registered paint table for “${id}”; preview uses ${SAFE_PAINT_FALLBACK}.`
	};
}

//#endregion
//#region src/themes-surface/themes-surface-vm.ts
/**
* Themes surface VM — project home master–detail (Variant B).
* Reuses @ship/studio/design store: loadThemes / openTheme / addThemeToProject.
*
* Paint mode is per-theme (from theme meta), not the studio shell mode.
* Light-only themes (paper, glass, halo, brut, …) have no dark CSS tables —
* forcing data-mode=dark makes preview fall through to :root (studio).
*
* Membership model = project file on disk (catalog vs project).
* Delete + Add to project require confirm. UI never says "fork".
* No fake enable-off switch; no session-default star theater.
*/
const THEME_BLURBS = {
	studio: "Warm product chrome for day-to-day building.",
	ship: "Cool SHIP defaults — slate surfaces and amber primary.",
	sail: "Sail brand — the default identity for new apps.",
	neutral: "Quiet grays. Lets content and one accent do the work.",
	paper: "Print-inspired light surfaces and editorial type feel.",
	halo: "Soft luminous surfaces with gentle depth.",
	console: "Terminal-dark phosphor energy for dense tooling.",
	contrast: "High-contrast accessibility-first pairings.",
	glass: "Translucent panels and soft refraction cues.",
	brut: "Brutalist edges — hard borders, strong type.",
	"8bit": "Retro game-adjacent palette and hard pixels.",
	synthwave: "Neon dusk — magenta, cyan, night sky."
};
function titleCase(id) {
	return id.split(/[-_]/).filter(Boolean).map((p) => p.charAt(0).toUpperCase() + p.slice(1)).join(" ") || id;
}
/** Human label + blurb + paint id for a theme id (registered or project copy). */
function describeTheme(id, forkedFrom) {
	const reg = THEMES.find((t) => t.id === id);
	const resolved = resolvePaintId(id, forkedFrom);
	const previewId = resolved.paintId;
	const parent = THEMES.find((t) => t.id === previewId);
	if (reg) return {
		name: reg.label,
		blurb: THEME_BLURBS[id] ?? `${reg.label} — full token bundle from the Sail catalog.`,
		previewId: id,
		modeInfo: modesForPaintId(id)
	};
	return {
		name: titleCase(id),
		blurb: parent ? `Project theme based on ${parent.label}. Open to edit Color, Shape, and other facets; live draft paint uses applyScope when available.` : "Project theme. Open it to edit DTCG tokens in Themes.",
		previewId,
		modeInfo: modesForPaintId(previewId),
		paintWarning: resolved.warning
	};
}
function createThemesSurface() {
	const projectRows = signal([]);
	const catalogRows = signal([]);
	const hasProjectRows = signal(false);
	const hasCatalogRows = signal(false);
	const hasAnyThemes = signal(false);
	const selectedId = signal("");
	const hasSelected = signal(false);
	const previewTitle = signal("Select a theme");
	const previewDesc = signal("");
	const previewMeta = signal("");
	const previewStatusLabel = signal("");
	const previewStatusTone = signal("muted");
	/** True when selected theme is in the project (file on disk). */
	const previewInProject = signal(false);
	const previewIsCatalog = signal(false);
	const previewThemeId = signal("studio");
	const previewPaintId = signal("studio");
	/** Paint mode for the selected theme (clamped to that theme’s tables). */
	const previewModeId = signal("dark");
	const previewModesLabel = signal("");
	const canToggleMode = signal(false);
	const modeLightOn = signal(false);
	const modeDarkOn = signal(false);
	/** Ephemeral feedback — fixed toast overlay (not an in-flow status band). */
	const showToast = signal(false);
	const toastMessage = signal("");
	const toastTone = signal("muted");
	const toastItems = signal([]);
	let toastSeq = 0;
	const themesError = signal("");
	const hasThemesError = signal(false);
	const loadingLabel = signal("");
	const isLoading = signal(false);
	const projectCountLabel = signal("0");
	const catalogCountLabel = signal("0");
	const menuTargetId = signal("");
	/**
	* User-chosen mode when theme supports both. Keyed by paint id.
	* Single-mode themes ignore this and always use their only table.
	*/
	const modeByPaintId = /* @__PURE__ */ new Map();
	/** Avoid re-toasting the same paint fallback on every sync. */
	let paintWarnForId = "";
	const resolvePaintMode = (paintId) => {
		const info = modesForPaintId(paintId);
		if (!info.dual) return info.defaultMode;
		return modeByPaintId.get(paintId) ?? info.defaultMode;
	};
	const mapRow = (t, sel, open) => {
		const inProject = !t.catalog;
		const statusLabel = inProject ? "" : "Catalog";
		const statusTone = "muted";
		const rowFork = open?.forkedFrom && open.themeId === t.id ? open.forkedFrom : null;
		const d = describeTheme(t.id, rowFork);
		const paintMode = resolvePaintMode(d.previewId);
		const meta = inProject ? d.modeInfo.modesLabel : `Catalog · ${d.modeInfo.modesLabel}`;
		return {
			id: t.id,
			name: d.name,
			meta,
			blurb: d.blurb,
			statusLabel,
			statusTone,
			selected: sel === t.id,
			inProject,
			isCatalog: t.catalog,
			catalog: t.catalog,
			previewId: d.previewId,
			paintMode,
			modesLabel: d.modeInfo.modesLabel,
			menuId: `theme-menu-${t.id}`,
			key: t.id
		};
	};
	const allRows = () => [...projectRows(), ...catalogRows()];
	const sync = () => {
		const ds = getDesignState();
		const sel = selectedId() || ds.themeId || ds.projectThemes[0]?.id || ds.catalogThemes[0]?.id || "";
		if (sel && selectedId() !== sel) selectedId.set(sel);
		const open = {
			themeId: ds.themeId || "",
			forkedFrom: ds.forkedFrom
		};
		const proj = ds.projectThemes.map((t) => mapRow(t, sel, open));
		const projIds = new Set(ds.projectThemes.map((t) => t.id));
		const cat = ds.catalogThemes.filter((t) => !projIds.has(t.id)).map((t) => mapRow(t, sel, open));
		projectRows.set(proj);
		catalogRows.set(cat);
		hasProjectRows.set(proj.length > 0);
		hasCatalogRows.set(cat.length > 0);
		hasAnyThemes.set(proj.length + cat.length > 0);
		projectCountLabel.set(String(proj.length));
		catalogCountLabel.set(String(cat.length));
		const row = allRows().find((r) => r.id === sel) ?? null;
		hasSelected.set(!!row);
		if (row) {
			const selectedFork = open.forkedFrom && row.id === open.themeId ? open.forkedFrom : null;
			const resolved = resolvePaintId(row.id, selectedFork);
			const previewId = resolved.paintId;
			const info = modesForPaintId(previewId);
			const mode = resolvePaintMode(previewId);
			previewTitle.set(row.name);
			previewDesc.set(row.blurb);
			previewMeta.set(row.meta);
			previewStatusLabel.set(row.statusLabel);
			previewStatusTone.set(row.statusTone);
			previewInProject.set(row.inProject);
			previewIsCatalog.set(row.isCatalog);
			previewThemeId.set(row.id);
			previewPaintId.set(previewId);
			previewModeId.set(mode);
			previewModesLabel.set(info.modesLabel);
			canToggleMode.set(info.dual);
			modeLightOn.set(mode === "light");
			modeDarkOn.set(mode === "dark");
			if (resolved.warning && paintWarnForId !== row.id) {
				paintWarnForId = row.id;
				setNote(resolved.warning, "warning");
			}
		} else {
			previewTitle.set("Select a theme");
			previewDesc.set("Choose a theme on the left to preview its surfaces, type, and controls.");
			previewMeta.set("");
			previewStatusLabel.set("");
			previewStatusTone.set("muted");
			previewInProject.set(false);
			previewIsCatalog.set(false);
			previewThemeId.set(ds.themeId || "studio");
			previewPaintId.set(ds.themeId || "studio");
			previewModeId.set("dark");
			previewModesLabel.set("");
			canToggleMode.set(false);
			modeLightOn.set(false);
			modeDarkOn.set(false);
		}
		const err = ds.error || ds.lastWriteError;
		themesError.set(err ?? "");
		hasThemesError.set(!!err);
		isLoading.set(ds.loading);
		loadingLabel.set(ds.loading ? "Loading themes…" : "");
	};
	const setNote = (msg, tone = "muted") => {
		if (!msg) {
			showToast.set(false);
			toastMessage.set("");
			toastTone.set("muted");
			toastItems.set([]);
			return;
		}
		toastSeq += 1;
		const item = {
			key: `toast-${toastSeq}`,
			message: msg,
			tone
		};
		toastMessage.set(msg);
		toastTone.set(tone);
		toastItems.set([item]);
		showToast.set(true);
	};
	const onToastDismiss = () => {
		showToast.set(false);
		toastMessage.set("");
		toastItems.set([]);
	};
	const selectTheme = (id) => {
		if (!id) return;
		selectedId.set(id);
		sync();
	};
	const applyPreviewMode = (m) => {
		const paintId = previewPaintId();
		const info = modesForPaintId(paintId);
		if (!info.modes.includes(m)) {
			setNote(`${previewTitle()} only ships a ${info.modesLabel.toLowerCase()} table.`);
			return;
		}
		modeByPaintId.set(paintId, m);
		sync();
	};
	/** Switch checked = dark; unchecked = light (matches facet chrome). */
	const onPreviewModeSwitch = (e) => {
		const checked = e.target.checked;
		applyPreviewMode(checked ? "dark" : "light");
	};
	const onListClick = (e) => {
		const el = e.target?.closest?.("[theme-id]");
		if (!el?.dataset.themeId) return;
		const ctrl = e.target?.closest?.("switch, [role=\"switch\"], button, menu-item, [sail-host]=\"menu-item\"], a, input, [popover]");
		if (ctrl && ctrl !== el) return;
		selectTheme(el.dataset.themeId);
	};
	const onMenuOpen = (e) => {
		e.stopPropagation();
		const id = (e.target?.closest?.("[theme-id]"))?.dataset.themeId;
		if (id) {
			menuTargetId.set(id);
			selectTheme(id);
		}
	};
	/** Catalog → project file via confirm + addThemeToProject. */
	const addCatalogToProject = (id) => {
		const row = allRows().find((r) => r.id === id);
		if (!row || row.inProject) return;
		const newId = id.startsWith("my-") ? id : `my-${id}`;
		const srcName = describeTheme(id).name;
		if (!confirmAddToProject(srcName, newId)) {
			setNote("Add to project cancelled.");
			return;
		}
		setNote(`Adding ${srcName} to the project…`);
		addThemeToProject(newId, id).then((ok) => {
			if (ok) {
				selectedId.set(newId);
				setNote(`${describeTheme(newId).name} is in the project.`, "success");
			} else setNote(getDesignState().lastWriteError || "Could not add that theme to the project.", "destructive");
			sync();
		});
	};
	/** List-row Add to project (catalog only). */
	const onAddToProject = (e) => {
		e.stopPropagation();
		const id = (e.target?.closest?.("[theme-id]"))?.dataset.themeId;
		if (!id) return;
		selectTheme(id);
		addCatalogToProject(id);
	};
	const onOpenTheme = (id) => {
		const tid = id || selectedId();
		if (!tid) return;
		openTheme(tid).then(() => {
			const err = getDesignState().error;
			if (err) {
				setNote(err, "destructive");
				selectTheme(tid);
				sync();
				return;
			}
			selectTheme(tid);
			sync();
			navigate(`${appRouteBase()}/themes/${encodeURIComponent(tid)}`);
		});
	};
	const onPreviewOpen = () => onOpenTheme();
	const onPreviewAddToProject = () => {
		const id = selectedId();
		if (!id) return;
		addCatalogToProject(id);
	};
	const onNewTheme = () => {
		const src = selectedId() || getDesignState().themeId || getDesignState().catalogThemes[0]?.id || "studio";
		const newId = `my-theme-${Date.now().toString(36).slice(-4)}`;
		const srcName = describeTheme(src).name;
		if (!confirmNewTheme(srcName, newId)) {
			setNote("New theme cancelled.");
			return;
		}
		setNote("Creating project theme…");
		addThemeToProject(newId, src).then((ok) => {
			if (ok) {
				selectedId.set(newId);
				setNote(`Created ${describeTheme(newId).name} from ${srcName}.`, "success");
			} else setNote(getDesignState().lastWriteError || "Could not create theme.", "destructive");
			sync();
		});
	};
	const onRowMenu = (e) => {
		const cmd = e.detail?.value || "";
		const id = menuTargetId() || selectedId();
		if (!cmd || !id) return;
		selectTheme(id);
		if (cmd === "open") onOpenTheme(id);
		else if (cmd === "enable" || cmd === "add") addCatalogToProject(id);
		else if (cmd === "remix" || cmd === "duplicate") {
			const newId = `my-${id}-${Date.now().toString(36).slice(-4)}`.replace(/[^a-z0-9-]/g, "-").replace(/-+/g, "-");
			const srcName = describeTheme(id).name;
			if (!confirmRemixTheme(srcName, newId)) {
				setNote("Remix cancelled.");
				return;
			}
			addThemeToProject(newId, id).then((ok) => {
				if (ok) {
					setNote(`Remixed as ${describeTheme(newId).name}`, "success");
					selectedId.set(newId);
				} else setNote(getDesignState().lastWriteError || "Remix failed", "destructive");
				sync();
			});
		} else if (cmd === "delete") {
			if (allRows().find((r) => r.id === id)?.isCatalog) {
				setNote("Catalog themes cannot be deleted.", "warning");
				return;
			}
			if (!confirmDeleteTheme(describeTheme(id).name)) {
				setNote("Delete cancelled.");
				return;
			}
			deleteTheme(id).then((ok) => {
				if (ok) {
					if (selectedId() === id) selectedId.set("");
					setNote("Theme deleted.", "success");
				} else setNote(getDesignState().lastWriteError || "Could not delete theme.", "destructive");
				sync();
			});
		}
	};
	const onRetryThemes = () => {
		loadThemes().then(() => {
			setNote("Theme list reloaded.");
			sync();
		});
	};
	sync();
	mountAmbient(() => {
		const unsub = subscribeDesign(() => sync());
		const ds = getDesignState();
		if (ds.projectThemes.length === 0 && ds.catalogThemes.length === 0 && !ds.loading) loadThemes().then(() => sync());
		return () => unsub();
	});
	return {
		projectRows,
		catalogRows,
		hasProjectRows,
		hasCatalogRows,
		hasAnyThemes,
		selectedId,
		hasSelected,
		previewTitle,
		previewDesc,
		previewMeta,
		previewStatusLabel,
		previewStatusTone,
		previewInProject,
		/** @deprecated alias — project membership, not an enable flag */
		previewEnabled: previewInProject,
		previewIsCatalog,
		previewThemeId,
		previewPaintId,
		previewModeId,
		previewModesLabel,
		canToggleMode,
		modeLightOn,
		modeDarkOn,
		showToast,
		toastMessage,
		toastTone,
		toastItems,
		themesError,
		hasThemesError,
		loadingLabel,
		isLoading,
		projectCountLabel,
		catalogCountLabel,
		onListClick,
		onMenuOpen,
		onAddToProject,
		onPreviewOpen,
		onPreviewAddToProject,
		/** @deprecated alias for template migration */
		onPreviewEnable: onPreviewAddToProject,
		onPreviewModeSwitch,
		onNewTheme,
		onRowMenu,
		onRetryThemes,
		onToastDismiss
	};
}

//#endregion
//#region src/themes-surface/themes-surface.sail.controller.ts
var themes_surface_sail_controller_default = controller(() => {
	const vm = createThemesSurface();
	return {
		projectRows: vm.projectRows,
		catalogRows: vm.catalogRows,
		hasProjectRows: vm.hasProjectRows,
		hasCatalogRows: vm.hasCatalogRows,
		hasAnyThemes: vm.hasAnyThemes,
		hasSelected: vm.hasSelected,
		previewTitle: vm.previewTitle,
		previewDesc: vm.previewDesc,
		previewMeta: vm.previewMeta,
		previewStatusLabel: vm.previewStatusLabel,
		previewStatusTone: vm.previewStatusTone,
		previewInProject: vm.previewInProject,
		previewIsCatalog: vm.previewIsCatalog,
		previewThemeId: vm.previewThemeId,
		previewPaintId: vm.previewPaintId,
		previewModeId: vm.previewModeId,
		previewModesLabel: vm.previewModesLabel,
		canToggleMode: vm.canToggleMode,
		modeLightOn: vm.modeLightOn,
		modeDarkOn: vm.modeDarkOn,
		showToast: vm.showToast,
		toastMessage: vm.toastMessage,
		toastTone: vm.toastTone,
		toastItems: vm.toastItems,
		themesError: vm.themesError,
		hasThemesError: vm.hasThemesError,
		loadingLabel: vm.loadingLabel,
		isLoading: vm.isLoading,
		projectCountLabel: vm.projectCountLabel,
		catalogCountLabel: vm.catalogCountLabel,
		onListClick: (e) => vm.onListClick(e),
		onMenuOpen: (e) => vm.onMenuOpen(e),
		onAddToProject: (e) => vm.onAddToProject(e),
		onPreviewOpen: () => vm.onPreviewOpen(),
		onPreviewAddToProject: () => vm.onPreviewAddToProject(),
		onPreviewModeSwitch: (e) => vm.onPreviewModeSwitch(e),
		onNewTheme: () => vm.onNewTheme(),
		onRowMenu: (e) => vm.onRowMenu(e),
		onRetryThemes: () => vm.onRetryThemes(),
		onToastDismiss: () => vm.onToastDismiss()
	};
});

//#endregion
//#region src/themes-surface/themes-surface.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0$1 = /*@__PURE__*/ __tpl("<div themes-header=\"\" class=\"sail-row sail-g-4 sail-ai-center sail-jc-between sail-nowrap sx-shrink0 sx-padX6 sx-padY4 sx-borderBWidth1 sx-borderBbordersubtle sx-bgsurface1\"><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><p class=\"sx-fgforeground sx-fontSizelg sx-fontWeightsemibold sx-trackingtight\">\n        \n        Themes\n      </p><p class=\"sx-fgmutedforeground sx-fontSizesm sx-leadingsnug\">\n        \n        Project themes live on disk. Add catalog themes to the project, then open one to edit.\n      </p></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap sx-shrink0\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" new-theme=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n        New theme\n      </button></div></div>");
const __tpl1 = /*@__PURE__*/ __tpl("<div themes-error=\"\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sx-shrink0 sx-padX6 sx-padY2 sx-borderBWidth1 sx-borderBborder\"><p class=\"sx-grow1 sx-minW0 sx-fgdestructive sx-fontSizesm sx-truncate\"> </p><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" retry-themes=\"\" tone=\"default\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm\">\n        Retry\n      </button></div>");
const __tpl2 = /*@__PURE__*/ __tpl("<div themes-loading=\"\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sx-shrink0 sx-padX6 sx-padY2\"><span data-sail-host=\"spinner\" role=\"status\" aria-label=\"Loading\" size=\"sm\" tone=\"default\" class=\"sail-spinner sx-displayinlineblock sx-alignmiddle sx-sizeiconsizemd sx-fgspinnerforeground sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-tonedefault-fgspinnerforeground sx-toneprimary-fgprimary sx-toneinfo-fginfo sx-tonesuccess-fgsuccess sx-tonewarning-fgwarning sx-tonedestructive-fgdestructive sx-toneaccent-fgaccent sx-centeredtrue-displayblock sx-centeredtrue-my8 sx-centeredtrue-mxauto\"><span aria-hidden=\"true\" class=\"sx-displayblock sx-sizefull sx-borderWidthcontrolstroke sx-bordersolid sx-borderspinnertrack sx-borderRcurrentColor sx-radiusfull sx-animatespin\">\n    </span></span><p class=\"sx-fgmutedforeground sx-fontSizesm\"> </p></div>");
const __tpl3 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightbold sx-textCaseuppercase sx-trackingwide\">\n          \n          Library\n        </p>");
const __tpl4 = /*@__PURE__*/ __tpl("<div data-sail-host=\"empty-state\" role=\"note\" aria-live=\"off\" variant=\"placeholder\" size=\"sm\" themes-empty=\"\" tone=\"default\" class=\"sx-wfull sail-empty-state sail-col sail-g-3 sail-ai-stretch sail-jc-center sx-fontFamilybody sx-fgemptystateforeground sx-textAligncenter sx-borderWidth1 sx-borderemptystateborder sx-radiuslg sx-bgemptystate sx-shadowxs sx-padY16 sx-padX4 sx-sizemd-padY16 sx-sizemd-padX4 sx-sizesm-padY7 sx-sizesm-padX4 sx-sizelg-padY24 sx-sizelg-padX6 sx-variantdefault-textAligncenter sx-variantdefault-borderWidth1 sx-variantdefault-borderemptystateborder sx-variantdefault-radiuslg sx-variantdefault-bgemptystate sx-variantdefault-shadowxs sx-variantinline-textAlignstart sx-variantinline-borderWidth1 sx-variantinline-borderdashed sx-variantinline-borderemptystateborder sx-variantinline-radiusmd sx-variantinline-bgmuted sx-variantinline-shadownone sx-variantinline-padY3 sx-variantinline-padX4 sx-variantplaceholder-textAligncenter sx-variantplaceholder-borderWidth0 sx-variantplaceholder-bgtransparent sx-variantplaceholder-radiusnone sx-variantplaceholder-shadownone sx-variantplaceholder-padY6 sx-variantconnecting-textAligncenter sx-variantconnecting-borderWidth0 sx-variantconnecting-bgtransparent sx-variantconnecting-radiusnone sx-variantconnecting-shadownone sx-variantconnecting-padY6 sx-forcedcolors-borderCanvasText\"><div aria-hidden=\"true\" class=\"sail-row-i sail-g-3 sail-ai-center sail-jc-center sail-nowrap sx-shrink1 sx-selfcenter sx-radiuslg sx-bgmuted sx-fgmutedforeground sx-size12 sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-toneprimary-bgmuted sx-toneprimary-fgprimary sx-toneinfo-bgmuted sx-toneinfo-fginfo sx-tonesuccess-bgmuted sx-tonesuccess-fgsuccess sx-tonewarning-bgmuted sx-tonewarning-fgwarning sx-tonedestructive-bgmuted sx-tonedestructive-fgdestructive sx-toneaccent-bgmuted sx-toneaccent-fgaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-empty-displaynone sx-forcedcolors-bgCanvas sx-forcedcolors-fgCanvasText\"></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0 sx-selfstretch\"><div class=\"sx-fontFamilyheading sx-fontWeightsemibold sx-fgforeground sx-leadingtight sx-fontSizemd sx-empty-displaynone\"><strong slot=\"title\">No themes loaded</strong></div><div class=\"sx-leadingloose sx-fgmutedforeground sx-fontSizesm sx-maxWpanelchwmd sx-empty-displaynone\">\n                \n                Retry, or create a project theme with New theme.\n              </div></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-center sx-selfcenter sx-empty-displaynone\"></div></div>");
const __tpl5 = /*@__PURE__*/ __tpl("<p list-group=\"project\" class=\"sx-padX2 sx-padT1 sx-padB1 sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n              \n              In project\n            </p>");
const __tpl6 = /*@__PURE__*/ __tpl("<div theme-row=\"\" theme-scope=\"project\" class=\"sail-row sail-g-3 sail-ai-center sail-jc-start sail-nowrap sx-pad2 sx-radiusmd sx-cursorpointer sx-borderWidth1 sx-bordertransparent sx-hover-bghoverbg sx-current-bgselectedbg sx-current-borderselectedborder\"><div theme-swatches=\"\" aria-hidden=\"true\" class=\"sail-row sail-g-0 sail-ai-center sail-jc-start sail-nowrap sx-shrink0 sx-w14 sx-h7 sx-minW14 sx-overflowhidden sx-radiussm sx-borderWidth1 sx-borderbordersubtle\"><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgbackground\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgsurface2\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgprimary\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgaccent\">\n                    </div></div><div class=\"sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minW0\"><p class=\"sx-fontSizesm sx-fontWeightsemibold sx-truncate\"> </p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-truncate\"> </p></div><div class=\"sail-row sail-g-1 sail-ai-center sail-jc-start sail-nowrap sx-shrink0\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" tone=\"muted\" theme-menu=\"\" aria-label=\"Theme menu\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-tonemutedvariantghost-fgmutedforeground\"><span data-sail-host=\"icon\" name=\"ellipsis\" size=\"sm\" class=\"sail-icon sx-shrink0 sx-displayinlineblock sx-alignmiddle sx-fgcurrentColor sx-sizeiconsizemd sx-glyphfill sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-sizexl-sizeiconsize2xl\"></span></button><div popover=\"\" data-sail-host=\"dropdown\" placement=\"bottom-end\" class=\"sail-dropdown sx-positionfixed sx-insetauto sx-m0 sx-leftbindanchorLeft sx-topbindanchorTop sx-wmax sx-pad15 sx-fgdropdownforeground sx-bgdropdown sx-borderWidth1 sx-borderborder sx-radiusmd sx-shadowmd sx-zpopover\"><button type=\"button\" data-sail-host=\"menu-item\" value=\"open\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Open</button><button type=\"button\" data-sail-host=\"menu-item\" value=\"remix\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Remix</button><button type=\"button\" data-sail-host=\"menu-item\" value=\"delete\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Delete</button></div></div></div>");
const __tpl7 = /*@__PURE__*/ __tpl("<p list-group=\"catalog\" class=\"sx-padX2 sx-padT4 sx-padB1 sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n              \n              Catalog\n            </p>");
const __tpl8 = /*@__PURE__*/ __tpl("<div theme-row=\"\" theme-scope=\"catalog\" class=\"sail-row sail-g-3 sail-ai-center sail-jc-start sail-nowrap sx-pad2 sx-radiusmd sx-cursorpointer sx-borderWidth1 sx-bordertransparent sx-hover-bghoverbg sx-current-bgselectedbg sx-current-borderselectedborder\"><div theme-swatches=\"\" aria-hidden=\"true\" class=\"sail-row sail-g-0 sail-ai-center sail-jc-start sail-nowrap sx-shrink0 sx-w14 sx-h7 sx-minW14 sx-overflowhidden sx-radiussm sx-borderWidth1 sx-borderbordersubtle\"><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgbackground\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgsurface2\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgprimary\">\n                    </div><div class=\"sail-center sail-g-0 sx-grow1 sx-minW0 sx-hfull sx-bgaccent\">\n                    </div></div><div class=\"sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minW0\"><p class=\"sx-fontSizesm sx-fontWeightsemibold sx-truncate\"> </p><p class=\"sx-fgmutedforeground sx-fontSizexs sx-truncate\"> </p></div><span data-sail-host=\"badge\" tone=\"muted\" size=\"sm\" variant=\"outline\" pill=\"\" class=\"sx-shrink0 sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Catalog</span><div class=\"sail-row sail-g-1 sail-ai-center sail-jc-start sail-nowrap sx-shrink0\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"outline\" tone=\"primary\" theme-add-project=\"\" aria-label=\"Add to project\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantoutline-borderforeground sx-variantoutline-fgforeground sx-variantoutlinedisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantoutline-borderprimary sx-toneprimaryvariantoutline-fgprimary\">\n                    Add\n                  </button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" tone=\"muted\" theme-menu=\"\" aria-label=\"Theme menu\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-tonemutedvariantghost-fgmutedforeground\"><span data-sail-host=\"icon\" name=\"ellipsis\" size=\"sm\" class=\"sail-icon sx-shrink0 sx-displayinlineblock sx-alignmiddle sx-fgcurrentColor sx-sizeiconsizemd sx-glyphfill sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-sizexl-sizeiconsize2xl\"></span></button><div popover=\"\" data-sail-host=\"dropdown\" placement=\"bottom-end\" class=\"sail-dropdown sx-positionfixed sx-insetauto sx-m0 sx-leftbindanchorLeft sx-topbindanchorTop sx-wmax sx-pad15 sx-fgdropdownforeground sx-bgdropdown sx-borderWidth1 sx-borderborder sx-radiusmd sx-shadowmd sx-zpopover\"><button type=\"button\" data-sail-host=\"menu-item\" value=\"enable\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Add to project</button><button type=\"button\" data-sail-host=\"menu-item\" value=\"open\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Open</button><button type=\"button\" data-sail-host=\"menu-item\" value=\"duplicate\" class=\"sail-menu-item sx-displayflex sx-gap25 sx-padY2 sx-padX3 sx-wfull sx-borderWidth0 sx-textAlignstart sx-fontFamilybody sx-fontSizesm sx-leadingtight sx-fgmenuitemforeground sx-bgtransparent sx-radiussm sx-cursorpointer sx-whitespacenowrap sx-hover-bgmenuitemhoverbg sx-focus-bgmenuitemhoverbg sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlinering sx-disabled-opacity50 sx-disabled-cursornotallowed\">Duplicate</button></div></div></div>");
const __tpl9 = /*@__PURE__*/ __tpl("<div data-sail-host=\"empty-state\" role=\"note\" aria-live=\"off\" variant=\"placeholder\" size=\"sm\" themes-preview-empty=\"\" tone=\"default\" class=\"sx-wfull sail-empty-state sail-col sail-g-3 sail-ai-stretch sail-jc-center sx-fontFamilybody sx-fgemptystateforeground sx-textAligncenter sx-borderWidth1 sx-borderemptystateborder sx-radiuslg sx-bgemptystate sx-shadowxs sx-padY16 sx-padX4 sx-sizemd-padY16 sx-sizemd-padX4 sx-sizesm-padY7 sx-sizesm-padX4 sx-sizelg-padY24 sx-sizelg-padX6 sx-variantdefault-textAligncenter sx-variantdefault-borderWidth1 sx-variantdefault-borderemptystateborder sx-variantdefault-radiuslg sx-variantdefault-bgemptystate sx-variantdefault-shadowxs sx-variantinline-textAlignstart sx-variantinline-borderWidth1 sx-variantinline-borderdashed sx-variantinline-borderemptystateborder sx-variantinline-radiusmd sx-variantinline-bgmuted sx-variantinline-shadownone sx-variantinline-padY3 sx-variantinline-padX4 sx-variantplaceholder-textAligncenter sx-variantplaceholder-borderWidth0 sx-variantplaceholder-bgtransparent sx-variantplaceholder-radiusnone sx-variantplaceholder-shadownone sx-variantplaceholder-padY6 sx-variantconnecting-textAligncenter sx-variantconnecting-borderWidth0 sx-variantconnecting-bgtransparent sx-variantconnecting-radiusnone sx-variantconnecting-shadownone sx-variantconnecting-padY6 sx-forcedcolors-borderCanvasText\"><div aria-hidden=\"true\" class=\"sail-row-i sail-g-3 sail-ai-center sail-jc-center sail-nowrap sx-shrink1 sx-selfcenter sx-radiuslg sx-bgmuted sx-fgmutedforeground sx-size12 sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-toneprimary-bgmuted sx-toneprimary-fgprimary sx-toneinfo-bgmuted sx-toneinfo-fginfo sx-tonesuccess-bgmuted sx-tonesuccess-fgsuccess sx-tonewarning-bgmuted sx-tonewarning-fgwarning sx-tonedestructive-bgmuted sx-tonedestructive-fgdestructive sx-toneaccent-bgmuted sx-toneaccent-fgaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-empty-displaynone sx-forcedcolors-bgCanvas sx-forcedcolors-fgCanvasText\"></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0 sx-selfstretch\"><div class=\"sx-fontFamilyheading sx-fontWeightsemibold sx-fgforeground sx-leadingtight sx-fontSizemd sx-empty-displaynone\"><strong slot=\"title\">Nothing selected</strong></div><div class=\"sx-leadingloose sx-fgmutedforeground sx-fontSizesm sx-maxWpanelchwmd sx-empty-displaynone\">\n            \n            Pick a theme in the library to preview its surfaces and controls.\n          </div></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-center sx-selfcenter sx-empty-displaynone\"></div></div>");
const __tpl10 = /*@__PURE__*/ __tpl("<p class=\"sx-fontFamilydisplay sx-fontSize2xl sx-fontWeightmedium sx-trackingtight\"> </p>");
const __tpl11 = /*@__PURE__*/ __tpl("<span data-sail-host=\"badge\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\"> </span>");
const __tpl12 = /*@__PURE__*/ __tpl("<span data-sail-host=\"badge\" tone=\"muted\" size=\"sm\" variant=\"outline\" pill=\"\" modes-chip=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\"> </span>");
const __tpl13 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizesm sx-leadingrelaxed\"> </p>");
const __tpl14 = /*@__PURE__*/ __tpl("<p class=\"sx-fgmutedforeground sx-fontSizexs\"> </p>");
const __tpl15 = /*@__PURE__*/ __tpl("<div preview-mode=\"\" dual-mode=\"\" aria-label=\"Theme mode\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start sail-nowrap\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold\">\n                  Mode</p><span data-sail-host=\"icon\" name=\"sun\" size=\"sm\" aria-label=\"Light\" class=\"sx-shrink0 sail-icon sx-displayinlineblock sx-alignmiddle sx-fgcurrentColor sx-sizeiconsizemd sx-glyphfill sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-sizexl-sizeiconsize2xl sx-fgmutedforeground\">\n                  </span><input type=\"checkbox\" data-sail-host=\"switch\" role=\"switch\" size=\"sm\" tone=\"muted\" mode-switch=\"\" class=\"sail-switch sx-shrink0 sx-appearancenone sx-positionrelative sx-boxSizingborder sx-displayinlineblock sx-alignmiddle sx-m0 sx-cursorpointer sx-bgnone sx-bgswitch sx-borderWidth1 sx-borderswitchborder sx-radiusfull sx-wcontrolhsm sx-hiconsizemd sx-tapHighlightnone sx-transitioncolors sx-durationfast sx-easeout sx-sizemd-wcontrolhsm sx-sizemd-hiconsizemd sx-sizesm-wcontrolhxs sx-sizesm-hiconsizesm sx-sizelg-w7 sx-sizelg-hiconsizelg sx-tonedefault-bgswitch sx-toneprimary-bgswitch sx-toneinfo-bgswitch sx-tonesuccess-bgswitch sx-tonewarning-bgswitch sx-tonedestructive-bgswitch sx-toneaccent-bgswitch sx-tonemuted-bgswitch sx-hover-bghoverbg sx-checked-bgprimary sx-checked-borderselectedborder sx-checked-shadowxs sx-focus-outlinesolid sx-focus-outline2 sx-focus-outlineprimary sx-focus-outlineOffsetringoffset sx-disabled-cursornotallowed sx-disabled-bgswitch sx-disabled-borderborder sx-disabled-shadownone sx-before-contentempty sx-before-positionabsolute sx-before-top0 sx-before-bottom0 sx-before-myauto sx-before-leftpx sx-before-bgswitchthumb sx-before-radiusfull sx-before-shadowsm sx-before-size3 sx-before-transitionall sx-before-durationfast sx-before-easeout sx-sizemdbefore-size3 sx-sizesmbefore-size25 sx-sizelgbefore-sizeiconsizesm sx-checkedbefore-leftauto sx-checkedbefore-rightpx sx-checkedbefore-bgprimaryforeground sx-disabledbefore-shadowxs sx-tonedefaultchecked-bgprimary sx-tonedefaultfocus-outlineprimary sx-toneprimarychecked-bgprimary sx-toneprimaryfocus-outlineprimary sx-toneinfochecked-bginfo sx-toneinfofocus-outlineinfo sx-tonesuccesschecked-bgsuccess sx-tonesuccessfocus-outlinesuccess sx-tonewarningchecked-bgwarning sx-tonewarningfocus-outlinewarning sx-tonedestructivechecked-bgdestructive sx-tonedestructivefocus-outlinedestructive sx-toneaccentchecked-bgaccent sx-toneaccentfocus-outlineaccent sx-tonemutedchecked-bgmutedforeground sx-tonemutedfocus-outlinemutedforeground sx-tonedefaultcheckedbefore-bgprimaryforeground sx-toneprimarycheckedbefore-bgprimaryforeground sx-toneinfocheckedbefore-bginfoforeground sx-tonesuccesscheckedbefore-bgsuccessforeground sx-tonewarningcheckedbefore-bgwarningforeground sx-tonedestructivecheckedbefore-bgdestructiveforeground sx-toneaccentcheckedbefore-bgaccentforeground sx-tonemutedcheckedbefore-bgbackground sx-motionreducebefore-transitionnone sx-forcedcolors-borderCanvasText sx-forcedcolorschecked-bgHighlight sx-forcedcolorsfocus-outlineCanvasText\"><span data-sail-host=\"icon\" name=\"moon\" size=\"sm\" aria-label=\"Dark\" class=\"sx-shrink0 sail-icon sx-displayinlineblock sx-alignmiddle sx-fgcurrentColor sx-sizeiconsizemd sx-glyphfill sx-sizemd-sizeiconsizemd sx-sizexs-sizeiconsize2xs sx-sizesm-sizeiconsizesm sx-sizelg-sizeiconsizexl sx-sizexl-sizeiconsize2xl sx-fgmutedforeground\">\n                  </span></div>");
const __tpl16 = /*@__PURE__*/ __tpl("<button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" preview-open=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n                  Open\n                </button>");
const __tpl17 = /*@__PURE__*/ __tpl("<button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" preview-add-project=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">\n                  Add to project\n                </button>");
const __tpl18 = /*@__PURE__*/ __tpl("<button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"outline\" tone=\"primary\" preview-open=\"\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantoutline-borderforeground sx-variantoutline-fgforeground sx-variantoutlinedisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantoutline-borderprimary sx-toneprimaryvariantoutline-fgprimary\">\n                  Open\n                </button>");
const __tpl19 = /*@__PURE__*/ __tpl("<div color-story=\"\" class=\"sail-col sail-g-3 sail-ai-stretch\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n              \n              Color story\n            </p><div color-story-grid=\"\" class=\"sail-row sail-g-2 sail-ai-center sail-jc-start\"><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><div class=\"sail-center sail-g-0 sx-h16 sx-radiusmd sx-bgprimary sx-borderWidth1 sx-borderbordersubtle\">\n                  </div><p class=\"sx-fontSizexs sx-fontWeightmedium\">\n                  Primary</p></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><div class=\"sail-center sail-g-0 sx-h16 sx-radiusmd sx-bgsecondary sx-borderWidth1 sx-borderbordersubtle\">\n                  </div><p class=\"sx-fontSizexs sx-fontWeightmedium\">\n                  Secondary</p></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><div class=\"sail-center sail-g-0 sx-h16 sx-radiusmd sx-bgaccent sx-borderWidth1 sx-borderbordersubtle\">\n                  </div><p class=\"sx-fontSizexs sx-fontWeightmedium\">\n                  Accent</p></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><div class=\"sail-center sail-g-0 sx-h16 sx-radiusmd sx-bgbackground sx-borderWidth1 sx-borderbordersubtle\">\n                  </div><p class=\"sx-fontSizexs sx-fontWeightmedium\">\n                  Background</p></div><div class=\"sail-col sail-g-1 sail-ai-stretch sx-grow1 sx-minW0\"><div class=\"sail-center sail-g-0 sx-h16 sx-radiusmd sx-bgsurface1 sx-borderWidth1 sx-borderbordersubtle\">\n                  </div><p class=\"sx-fontSizexs sx-fontWeightmedium\">\n                  Surface</p></div></div></div>");
const __tpl20 = /*@__PURE__*/ __tpl("<div type-story=\"\" class=\"sail-col sail-g-2 sail-ai-stretch sx-pad4 sx-radiuslg sx-borderWidth1 sx-borderbordersubtle sx-bgsurface1 sx-fgforeground\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n              \n              Type · corners\n            </p><p class=\"sx-fontFamilydisplay sx-fontSizexl sx-fontWeightsemibold sx-trackingtight\">\n              \n              Display / heading sample\n            </p><p class=\"sx-fontFamilybody sx-fontSizesm sx-leadingrelaxed\">\n              \n              Body — the quick brown fox jumps over the lazy dog. 0123456789\n            </p><p class=\"sx-fontFamilymono sx-fontSizexs\">\n              \n              mono — const theme = \"preview\"\n            </p><div radius-chips=\"\" class=\"sail-row sail-g-2 sail-ai-end sail-jc-start\"><div class=\"sail-col sail-g-1 sail-ai-center\"><div aria-hidden=\"true\" class=\"sail-center sail-g-0 sx-w8 sx-h8 sx-bgtransparent sx-borderWidth2 sx-borderprimary sx-radiussm\">\n                  </div><p class=\"sx-fgmutedforeground sx-fontSizexs\">\n                  sm</p></div><div class=\"sail-col sail-g-1 sail-ai-center\"><div aria-hidden=\"true\" class=\"sail-center sail-g-0 sx-w8 sx-h8 sx-bgtransparent sx-borderWidth2 sx-borderprimary sx-radiusmd\">\n                  </div><p class=\"sx-fgmutedforeground sx-fontSizexs\">\n                  md</p></div><div class=\"sail-col sail-g-1 sail-ai-center\"><div aria-hidden=\"true\" class=\"sail-center sail-g-0 sx-w8 sx-h8 sx-bgtransparent sx-borderWidth2 sx-borderprimary sx-radiuslg\">\n                  </div><p class=\"sx-fgmutedforeground sx-fontSizexs\">\n                  lg</p></div><div class=\"sail-col sail-g-1 sail-ai-center\"><div aria-hidden=\"true\" class=\"sail-center sail-g-0 sx-w8 sx-h8 sx-bgtransparent sx-borderWidth2 sx-borderprimary sx-radiusxl\">\n                  </div><p class=\"sx-fgmutedforeground sx-fontSizexs\">\n                  xl</p></div></div></div>");
const __tpl21 = /*@__PURE__*/ __tpl("<div preview-chrome=\"\" class=\"sail-col sail-g-2 sail-ai-stretch\"><p class=\"sx-fgmutedforeground sx-fontSizexs sx-fontWeightsemibold sx-textCaseuppercase sx-trackingwide\">\n              \n              Live chrome\n            </p><div preview-specimen=\"\" class=\"sail-col sail-g-0 sail-ai-stretch sx-overflowhidden sx-minW0 sx-radiuslg sx-borderWidth1 sx-borderborder sx-shadowmd sx-bgbackground sx-fgforeground\"><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-between sail-nowrap sx-padX4 sx-h9 sx-bgsurface1 sx-borderBWidth1 sx-borderBbordersubtle\"><p class=\"sx-fontSizesm sx-fontWeightsemibold sx-truncate\"> </p><span data-sail-host=\"badge\" tone=\"primary\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Preview</span></div><div class=\"sail-col sail-g-4 sail-ai-stretch sx-pad5 sx-bgbackground\"><div data-sail-host=\"card\" tone=\"default\" variant=\"default\" padding=\"md\" accent-bar=\"off\" class=\"sail-card sx-positionrelative sx-isolate sx-overflowclip sx-radiuslg sx-border1border sx-fgcardforeground sx-shadowcardrest sx-transitioncolors sx-durationfast sx-easeout sx-bgcard sx-padY4 sx-padX5 sx-backdropcardbackdropfilter sx-borderImagecornerimage sx-variantdefault-bgcard sx-variantoutline-bgtransparent sx-paddingmd-padY4 sx-paddingmd-padX5 sx-paddingnone-pad0 sx-paddingsm-padY2 sx-paddingsm-padX3 sx-paddinglg-padY5 sx-paddinglg-padX6 sx-interactivetrue-cursorpointer sx-interactivetrue-tapHighlightnone sx-interactivetruehover-borderborderstrong sx-interactivetruehover-shadowhover sx-interactivetruehover-translateYpx sx-interactivetrueactive-shadowcardrest sx-interactivetrueactive-translateY0 sx-interactivetrueactive-durationinstant sx-interactivetruemotionreduce-translateY0 sx-interactivetruefocus-outline2 sx-interactivetrueforcedcolorsfocus-outlineCanvasText sx-filltrue-bgmuted sx-filltrue-borderborder sx-toneprimary-borderprimary sx-toneinfo-borderinfo sx-tonesuccess-bordersuccess sx-tonewarning-borderwarning sx-tonedestructive-borderdestructive sx-toneaccent-borderaccent sx-tonemuted-borderborder sx-statusrailtrue-borderS3borderstrong sx-statusrailtrue-padS4 sx-paddingsmstatusrailtrue-padS2 sx-paddingmdstatusrailtrue-padS4 sx-paddinglgstatusrailtrue-padS5 sx-paddingnonestatusrailtrue-padS0 sx-accentbarstaticbefore-contentempty sx-accentbarstaticbefore-positionabsolute sx-accentbarstaticbefore-left0 sx-accentbarstaticbefore-right0 sx-accentbarstaticbefore-top0 sx-accentbarstaticbefore-hhalf sx-accentbarstaticbefore-bgImageprimaryaccent sx-accentbarstaticbefore-opacity100 sx-accentbarstaticbefore-radiusTlg sx-accentbarstaticbefore-pointerEventsnone sx-accentbarstaticbefore-transitionopacity sx-accentbarstaticbefore-durationmedium sx-accentbarstaticbefore-easeout sx-accentbarhoverbefore-contentempty sx-accentbarhoverbefore-positionabsolute sx-accentbarhoverbefore-left0 sx-accentbarhoverbefore-right0 sx-accentbarhoverbefore-top0 sx-accentbarhoverbefore-hhalf sx-accentbarhoverbefore-bgImageprimaryaccent sx-accentbarhoverbefore-opacity0 sx-accentbarhoverbefore-radiusTlg sx-accentbarhoverbefore-pointerEventsnone sx-accentbarhoverbefore-transitionopacity sx-accentbarhoverbefore-durationmedium sx-accentbarhoverbefore-easeout sx-accentbarhoverhoverbefore-opacity100 sx-pad4\"><div class=\"sx-mt4 sx-mx5 sx-mb3 sx-padX5 sx-padY4 sx-borderB1bordersubtle sx-fontFamilyheading sx-empty-displaynone\"></div><div class=\"sail-col sail-g-3 sail-ai-stretch\"><p class=\"sx-fontSizemd sx-fontWeightsemibold sx-fontFamilyheading\">\n                      Sample card</p><p class=\"sx-fgmutedforeground sx-fontSizesm sx-leadingrelaxed sx-fontFamilybody\">\n                      \n                      Controls, badges, and fields read this theme’s token table.\n                    </p><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start\"><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"primary\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantsolid-bgprimary sx-toneprimaryvariantsolid-borderprimary sx-toneprimaryvariantsolid-fgprimaryforeground\">Primary</button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"solid\" tone=\"accent\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantsolid-bgforeground sx-variantsolid-borderforeground sx-variantsolid-fgbackground sx-variantsolid-shadowxs sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneaccentvariantsolid-bgaccent sx-toneaccentvariantsolid-borderaccent sx-toneaccentvariantsolid-fgaccentforeground\">Accent</button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"outline\" tone=\"primary\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantoutline-borderforeground sx-variantoutline-fgforeground sx-variantoutlinedisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-toneprimaryvariantoutline-borderprimary sx-toneprimaryvariantoutline-fgprimary\">Outline</button><button data-sail-host=\"button\" type=\"button\" size=\"sm\" variant=\"ghost\" tone=\"muted\" class=\"sail-button sx-displayinlineflex sx-appearancenone sx-fontWeightmedium sx-leadingnone sx-radiusmd sx-borderWidthcontrolstroke sx-bordersolid sx-bordertransparent sx-borderImagecornerimage sx-isolate sx-positionrelative sx-controlFrame sx-selectnone sx-whitespacenowrap sx-cursorpointer sx-hover-controlLift sx-active-press sx-disabled-cursornotallowed sx-disabled-opacity50 sx-variantghost-bgtransparent sx-variantghost-fgforeground sx-variantghostdisabled-bgtransparent sx-sizesm-minHcontrolhsm sx-sizesm-padX25 sx-sizesm-fontSizesm sx-tonemutedvariantghost-fgmutedforeground\">Ghost</button></div><div class=\"sail-row sail-g-2 sail-ai-center sail-jc-start\"><span data-sail-host=\"badge\" tone=\"primary\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Primary</span><span data-sail-host=\"badge\" tone=\"success\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Success</span><span data-sail-host=\"badge\" tone=\"warning\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Warning</span><span data-sail-host=\"badge\" tone=\"destructive\" size=\"sm\" variant=\"solid\" pill=\"\" class=\"sail-badge sx-displayinlineflex sx-fontFamilybody sx-fontWeightsemibold sx-leadingnone sx-whitespacenowrap sx-borderWidth1 sx-radiussm sx-minHcontrolh2xs sx-padX2 sx-padYpx sx-fontSizexs sx-variantsoft-bgmuted sx-variantsoft-fgmutedforeground sx-variantsoft-borderborder sx-variantsolid-bgprimary sx-variantsolid-fgprimaryforeground sx-variantsolid-borderprimary sx-variantsolid-shadowxs sx-variantoutline-bgtransparent sx-variantoutline-fgforeground sx-variantoutline-borderborder sx-sizemd-minHcontrolh2xs sx-sizemd-padX2 sx-sizemd-fontSizexs sx-sizesm-minH4 sx-sizesm-padX15 sx-sizesm-fontSizexs sx-sizelg-minHcontrolhxs sx-sizelg-padX25 sx-sizelg-fontSizexs sx-tonedefault-bgmuted sx-tonedefault-fgmutedforeground sx-tonedefault-borderborder sx-toneprimary-bgprimary sx-toneprimary-fgprimaryforeground sx-toneprimary-borderprimary sx-toneinfo-bginfo sx-toneinfo-fginfoforeground sx-toneinfo-borderinfo sx-tonesuccess-bgsuccess sx-tonesuccess-fgsuccessforeground sx-tonesuccess-bordersuccess sx-tonewarning-bgwarning sx-tonewarning-fgwarningforeground sx-tonewarning-borderwarning sx-tonedestructive-bgdestructive sx-tonedestructive-fgdestructiveforeground sx-tonedestructive-borderdestructive sx-toneaccent-bgaccent sx-toneaccent-fgaccentforeground sx-toneaccent-borderaccent sx-tonemuted-bgmuted sx-tonemuted-fgmutedforeground sx-tonemuted-borderborder sx-pilltrue-radiusfull sx-tonedefaultvariantsoft-bgmuted sx-tonedefaultvariantsoft-fgmutedforeground sx-tonedefaultvariantsoft-borderborder sx-hover-opacity90 sx-focus-outline2\">Danger</span></div><input data-sail-host=\"input\" type=\"search\" size=\"sm\" placeholder=\"Search field\" aria-label=\"Sample search\" tone=\"default\" class=\"sail-input sx-typesearch-displayblock sx-typesearch-minHcontrolhmd sx-typesearch-padY15 sx-typesearch-padX3 sx-typesearch-fontSizebase sx-typesearch-fgforeground sx-typesearch-bgbackground sx-typesearch-borderWidth1 sx-typesearch-borderborder sx-typesearch-radiusmd sx-typesearch-shadowxs sx-typesearch-wfull sx-typesearchdisabled-cursornotallowed sx-typesearchdisabled-opacity50 sx-typesearchsizexs-minHcontrolh2xs sx-typesearchsizexs-padX2 sx-typesearchsizexs-fontSizexs sx-typesearchsizesm-minHcontrolhsm sx-typesearchsizesm-padX25 sx-typesearchsizesm-fontSizesm sx-typesearchsizemd-minHcontrolhmd sx-typesearchsizemd-padX3 sx-typesearchsizemd-fontSizebase sx-typesearchsizelg-minHcontrolhlg sx-typesearchsizelg-padX4 sx-typesearchsizelg-fontSizemd sx-invalidtrue-borderdestructive\"></div><div class=\"sx-mt3 sx-mx5 sx-mb4 sx-padX5 sx-padY4 sx-borderT1bordersubtle sx-empty-displaynone\"></div></div></div></div></div>");
const __tpl22 = /*@__PURE__*/ __tpl("<div data-sail-host=\"toast\" role=\"status\" aria-live=\"polite\" themes-toast=\"\" class=\"sail-toast sail-row sail-g-3 sail-ai-start sail-jc-start sx-fontFamilybody sx-fontSizesm sx-fgtoastforeground sx-bgtoast sx-radiusmd sx-shadowmd sx-padY3 sx-padX4 sx-borderWidth1 sx-bordertoastborder sx-borderSWidth3 sx-borderSforeground sx-pointerEventsauto sx-tonedefault-borderSforeground sx-toneprimary-borderSprimary sx-toneinfo-borderSinfo sx-tonesuccess-borderSsuccess sx-tonewarning-borderSwarning sx-tonedestructive-borderSdestructive sx-toneaccent-borderSaccent sx-tonemuted-borderSmutedforeground\"> <button data-toast-close=\"\" type=\"button\" anatomy=\"\" aria-label=\"Dismiss\" class=\"sx-shrink0 sx-grow0 sx-msauto sx-fgmutedforeground sx-bgtransparent sx-borderWidth0 sx-radiussm sx-cursorpointer sx-hover-fgforeground sx-hover-bgmuted\">\n    ✕</button></div>");
delegateEvents(["change", "click"]);
const __default = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	if (themes_surface_sail_controller_default && typeof themes_surface_sail_controller_default.setup === "function") {
		ctx.scope = $scope;
		const __r = themes_surface_sail_controller_default.setup(ctx);
		if (__r && typeof __r === "object") Object.assign($scope, __r);
	}
	const __wireBoundary_b1 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.hasThemesError()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag5 = document.createDocumentFragment();
			const __c6 = [];
			const __t7 = document.createTextNode("\n    ");
			__frag5.appendChild(__t7);
			const __e8 = __tpl1();
			const __e9 = __e8.firstChild;
			const __t10 = __e9.firstChild;
			markHydrateText(__t10, "b2");
			__c6.push(bindText(__t10, () => $scope.themesError()));
			const __e11 = __e9.nextSibling;
			__e11.$$click = (function() {
				const __h = $scope.onRetryThemes;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag5.appendChild(__e8);
			const __t12 = document.createTextNode("\n  ");
			__frag5.appendChild(__t12);
			return {
				nodes: Array.from(__frag5.childNodes),
				cleanup: () => {
					for (const c of __c6) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b4 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.isLoading()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag14 = document.createDocumentFragment();
			const __c15 = [];
			const __t16 = document.createTextNode("\n    ");
			__frag14.appendChild(__t16);
			const __e17 = __tpl2();
			const __t19 = __e17.firstChild.nextSibling.firstChild;
			markHydrateText(__t19, "b5");
			__c15.push(bindText(__t19, () => $scope.loadingLabel()));
			__frag14.appendChild(__e17);
			const __t20 = document.createTextNode("\n  ");
			__frag14.appendChild(__t20);
			return {
				nodes: Array.from(__frag14.childNodes),
				cleanup: () => {
					for (const c of __c15) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b10 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.hasThemesError()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag37 = document.createDocumentFragment();
			const __c38 = [];
			const __t39 = document.createTextNode("\n              ");
			__frag37.appendChild(__t39);
			const __e40 = __tpl4();
			runAction(__e40, variantLive, void 0);
			__frag37.appendChild(__e40);
			const __t41 = document.createTextNode("\n            ");
			__frag37.appendChild(__t41);
			return {
				nodes: Array.from(__frag37.childNodes),
				cleanup: () => {
					for (const c of __c38) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b9 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.hasAnyThemes()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag33 = document.createDocumentFragment();
			const __c34 = [];
			const __t35 = document.createTextNode("\n            ");
			__frag33.appendChild(__t35);
			const __ifAnchor36 = document.createComment("if");
			__frag33.appendChild(__ifAnchor36);
			__c34.push(__wireBoundary_b10(__ifAnchor36));
			const __t42 = document.createTextNode("\n          ");
			__frag33.appendChild(__t42);
			return {
				nodes: Array.from(__frag33.childNodes),
				cleanup: () => {
					for (const c of __c34) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b12 = (anchor) => bindFor({
		anchor,
		items: () => $scope.projectRows(),
		key: (row, __i) => row.key,
		render: (row, __i) => {
			const __childScope50 = Object.create($scope);
			__childScope50.row = row;
			__childScope50.__i = __i;
			return (($scope) => {
				const __frag51 = document.createDocumentFragment();
				const __c52 = [];
				const __t53 = document.createTextNode("\n              ");
				__frag51.appendChild(__t53);
				const __e54 = __tpl6();
				{
					const __av55 = $scope.row.id;
					if (__av55 == null || __av55 === false || __av55 === "") __e54.removeAttribute("theme-id");
					else __e54.setAttribute("theme-id", __av55 === true ? "" : String(__av55));
				}
				{
					const __av56 = $scope.row.selected ? "true" : "false";
					if (__av56 == null || __av56 === false || __av56 === "") __e54.removeAttribute("selected");
					else __e54.setAttribute("selected", __av56 === true ? "" : String(__av56));
					if (__av56 == null || __av56 === false || __av56 === "") __e54.removeAttribute("aria-selected");
					else __e54.setAttribute("aria-selected", __av56 === true ? "true" : String(__av56));
				}
				{
					const __av57 = $scope.row.selected ? "true" : null;
					if (__av57 == null || __av57 === false || __av57 === "") __e54.removeAttribute("aria-current");
					else __e54.setAttribute("aria-current", __av57 === true ? "" : String(__av57));
				}
				const __e58 = __e54.firstChild;
				{
					const __av59 = $scope.row.previewId;
					if (__av59 == null || __av59 === false || __av59 === "") __e58.removeAttribute("theme");
					else __e58.setAttribute("theme", __av59 === true ? "" : String(__av59));
				}
				{
					const __av60 = $scope.row.paintMode;
					if (__av60 == null || __av60 === false || __av60 === "") __e58.removeAttribute("mode");
					else __e58.setAttribute("mode", __av60 === true ? "" : String(__av60));
				}
				const __e61 = __e58.nextSibling;
				const __e62 = __e61.firstChild;
				const __t63 = __e62.firstChild;
				markHydrateText(__t63, "b18");
				__t63.textContent = $scope.row.name;
				const __t65 = __e62.nextSibling.firstChild;
				markHydrateText(__t65, "b19");
				__t65.textContent = $scope.row.meta;
				const __e67 = __e61.nextSibling.firstChild;
				{
					const __av68 = $scope.row.menuId;
					if (__av68 == null || __av68 === false || __av68 === "") __e67.removeAttribute("popovertarget");
					else __e67.setAttribute("popovertarget", __av68 === true ? "" : String(__av68));
				}
				{
					const __av69 = $scope.row.id;
					if (__av69 == null || __av69 === false || __av69 === "") __e67.removeAttribute("theme-id");
					else __e67.setAttribute("theme-id", __av69 === true ? "" : String(__av69));
				}
				__e67.$$click = (function() {
					const __h = $scope.onMenuOpen;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})();
				const __e70 = __e67.firstChild;
				runAction(__e70, icon, void 0);
				const __e71 = __e67.nextSibling;
				runAction(__e71, anchor, void 0);
				runAction(__e71, menu, void 0);
				{
					const __av72 = $scope.row.menuId;
					if (__av72 == null || __av72 === false || __av72 === "") __e71.removeAttribute("id");
					else __e71.setAttribute("id", __av72 === true ? "" : String(__av72));
				}
				__c52.push(on(__e71, "sail-menu-select", (function() {
					const __h = $scope.onRowMenu;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})(), { modifiers: [] }));
				__frag51.appendChild(__e54);
				const __t73 = document.createTextNode("\n            ");
				__frag51.appendChild(__t73);
				return {
					nodes: Array.from(__frag51.childNodes),
					cleanup: () => {
						for (const c of __c52) try {
							c();
						} catch (e) {}
					},
					update: (__new74, __newI75) => {
						const __old = $scope.row;
						$scope.row = __new74;
						$scope.__i = __newI75;
						if (__old.id !== $scope.row.id) {
							{
								const __av55 = $scope.row.id;
								if (__av55 == null || __av55 === false || __av55 === "") __e54.removeAttribute("theme-id");
								else __e54.setAttribute("theme-id", __av55 === true ? "" : String(__av55));
							}
							{
								const __av69 = $scope.row.id;
								if (__av69 == null || __av69 === false || __av69 === "") __e67.removeAttribute("theme-id");
								else __e67.setAttribute("theme-id", __av69 === true ? "" : String(__av69));
							}
						}
						if (__old.selected !== $scope.row.selected) {
							{
								const __av56 = $scope.row.selected ? "true" : "false";
								if (__av56 == null || __av56 === false || __av56 === "") __e54.removeAttribute("selected");
								else __e54.setAttribute("selected", __av56 === true ? "" : String(__av56));
								if (__av56 == null || __av56 === false || __av56 === "") __e54.removeAttribute("aria-selected");
								else __e54.setAttribute("aria-selected", __av56 === true ? "true" : String(__av56));
							}
							{
								const __av57 = $scope.row.selected ? "true" : null;
								if (__av57 == null || __av57 === false || __av57 === "") __e54.removeAttribute("aria-current");
								else __e54.setAttribute("aria-current", __av57 === true ? "" : String(__av57));
							}
						}
						if (__old.previewId !== $scope.row.previewId) {
							const __av59 = $scope.row.previewId;
							if (__av59 == null || __av59 === false || __av59 === "") __e58.removeAttribute("theme");
							else __e58.setAttribute("theme", __av59 === true ? "" : String(__av59));
						}
						if (__old.paintMode !== $scope.row.paintMode) {
							const __av60 = $scope.row.paintMode;
							if (__av60 == null || __av60 === false || __av60 === "") __e58.removeAttribute("mode");
							else __e58.setAttribute("mode", __av60 === true ? "" : String(__av60));
						}
						if (__old.name !== $scope.row.name) __t63.textContent = $scope.row.name;
						if (__old.meta !== $scope.row.meta) __t65.textContent = $scope.row.meta;
						if (__old.menuId !== $scope.row.menuId) {
							{
								const __av68 = $scope.row.menuId;
								if (__av68 == null || __av68 === false || __av68 === "") __e67.removeAttribute("popovertarget");
								else __e67.setAttribute("popovertarget", __av68 === true ? "" : String(__av68));
							}
							{
								const __av72 = $scope.row.menuId;
								if (__av72 == null || __av72 === false || __av72 === "") __e71.removeAttribute("id");
								else __e71.setAttribute("id", __av72 === true ? "" : String(__av72));
							}
						}
					}
				};
			})(__childScope50);
		}
	});
	const __wireBoundary_b11 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.hasProjectRows()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag44 = document.createDocumentFragment();
			const __c45 = [];
			const __t46 = document.createTextNode("\n            ");
			__frag44.appendChild(__t46);
			const __e47 = __tpl5();
			__frag44.appendChild(__e47);
			const __t48 = document.createTextNode("\n            ");
			__frag44.appendChild(__t48);
			const __forAnchor49 = document.createComment("for");
			__frag44.appendChild(__forAnchor49);
			__c45.push(__wireBoundary_b12(__forAnchor49));
			const __t76 = document.createTextNode("\n          ");
			__frag44.appendChild(__t76);
			return {
				nodes: Array.from(__frag44.childNodes),
				cleanup: () => {
					for (const c of __c45) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b26 = (anchor) => bindFor({
		anchor,
		items: () => $scope.catalogRows(),
		key: (row, __i) => row.key,
		render: (row, __i) => {
			const __childScope84 = Object.create($scope);
			__childScope84.row = row;
			__childScope84.__i = __i;
			return (($scope) => {
				const __frag85 = document.createDocumentFragment();
				const __c86 = [];
				const __t87 = document.createTextNode("\n              ");
				__frag85.appendChild(__t87);
				const __e88 = __tpl8();
				{
					const __av89 = $scope.row.id;
					if (__av89 == null || __av89 === false || __av89 === "") __e88.removeAttribute("theme-id");
					else __e88.setAttribute("theme-id", __av89 === true ? "" : String(__av89));
				}
				{
					const __av90 = $scope.row.selected ? "true" : "false";
					if (__av90 == null || __av90 === false || __av90 === "") __e88.removeAttribute("selected");
					else __e88.setAttribute("selected", __av90 === true ? "" : String(__av90));
					if (__av90 == null || __av90 === false || __av90 === "") __e88.removeAttribute("aria-selected");
					else __e88.setAttribute("aria-selected", __av90 === true ? "true" : String(__av90));
				}
				{
					const __av91 = $scope.row.selected ? "true" : null;
					if (__av91 == null || __av91 === false || __av91 === "") __e88.removeAttribute("aria-current");
					else __e88.setAttribute("aria-current", __av91 === true ? "" : String(__av91));
				}
				const __e92 = __e88.firstChild;
				{
					const __av93 = $scope.row.previewId;
					if (__av93 == null || __av93 === false || __av93 === "") __e92.removeAttribute("theme");
					else __e92.setAttribute("theme", __av93 === true ? "" : String(__av93));
				}
				{
					const __av94 = $scope.row.paintMode;
					if (__av94 == null || __av94 === false || __av94 === "") __e92.removeAttribute("mode");
					else __e92.setAttribute("mode", __av94 === true ? "" : String(__av94));
				}
				const __e95 = __e92.nextSibling;
				const __e96 = __e95.firstChild;
				const __t97 = __e96.firstChild;
				markHydrateText(__t97, "b32");
				__t97.textContent = $scope.row.name;
				const __t99 = __e96.nextSibling.firstChild;
				markHydrateText(__t99, "b33");
				__t99.textContent = $scope.row.meta;
				const __e101 = __e95.nextSibling.nextSibling.firstChild;
				{
					const __av102 = $scope.row.id;
					if (__av102 == null || __av102 === false || __av102 === "") __e101.removeAttribute("theme-id");
					else __e101.setAttribute("theme-id", __av102 === true ? "" : String(__av102));
				}
				__e101.$$click = (function() {
					const __h = $scope.onAddToProject;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})();
				const __e103 = __e101.nextSibling;
				{
					const __av104 = $scope.row.menuId;
					if (__av104 == null || __av104 === false || __av104 === "") __e103.removeAttribute("popovertarget");
					else __e103.setAttribute("popovertarget", __av104 === true ? "" : String(__av104));
				}
				{
					const __av105 = $scope.row.id;
					if (__av105 == null || __av105 === false || __av105 === "") __e103.removeAttribute("theme-id");
					else __e103.setAttribute("theme-id", __av105 === true ? "" : String(__av105));
				}
				__e103.$$click = (function() {
					const __h = $scope.onMenuOpen;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})();
				const __e106 = __e103.firstChild;
				runAction(__e106, icon, void 0);
				const __e107 = __e103.nextSibling;
				runAction(__e107, anchor, void 0);
				runAction(__e107, menu, void 0);
				{
					const __av108 = $scope.row.menuId;
					if (__av108 == null || __av108 === false || __av108 === "") __e107.removeAttribute("id");
					else __e107.setAttribute("id", __av108 === true ? "" : String(__av108));
				}
				__c86.push(on(__e107, "sail-menu-select", (function() {
					const __h = $scope.onRowMenu;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})(), { modifiers: [] }));
				__frag85.appendChild(__e88);
				const __t109 = document.createTextNode("\n            ");
				__frag85.appendChild(__t109);
				return {
					nodes: Array.from(__frag85.childNodes),
					cleanup: () => {
						for (const c of __c86) try {
							c();
						} catch (e) {}
					},
					update: (__new110, __newI111) => {
						const __old = $scope.row;
						$scope.row = __new110;
						$scope.__i = __newI111;
						if (__old.id !== $scope.row.id) {
							{
								const __av89 = $scope.row.id;
								if (__av89 == null || __av89 === false || __av89 === "") __e88.removeAttribute("theme-id");
								else __e88.setAttribute("theme-id", __av89 === true ? "" : String(__av89));
							}
							{
								const __av102 = $scope.row.id;
								if (__av102 == null || __av102 === false || __av102 === "") __e101.removeAttribute("theme-id");
								else __e101.setAttribute("theme-id", __av102 === true ? "" : String(__av102));
							}
							{
								const __av105 = $scope.row.id;
								if (__av105 == null || __av105 === false || __av105 === "") __e103.removeAttribute("theme-id");
								else __e103.setAttribute("theme-id", __av105 === true ? "" : String(__av105));
							}
						}
						if (__old.selected !== $scope.row.selected) {
							{
								const __av90 = $scope.row.selected ? "true" : "false";
								if (__av90 == null || __av90 === false || __av90 === "") __e88.removeAttribute("selected");
								else __e88.setAttribute("selected", __av90 === true ? "" : String(__av90));
								if (__av90 == null || __av90 === false || __av90 === "") __e88.removeAttribute("aria-selected");
								else __e88.setAttribute("aria-selected", __av90 === true ? "true" : String(__av90));
							}
							{
								const __av91 = $scope.row.selected ? "true" : null;
								if (__av91 == null || __av91 === false || __av91 === "") __e88.removeAttribute("aria-current");
								else __e88.setAttribute("aria-current", __av91 === true ? "" : String(__av91));
							}
						}
						if (__old.previewId !== $scope.row.previewId) {
							const __av93 = $scope.row.previewId;
							if (__av93 == null || __av93 === false || __av93 === "") __e92.removeAttribute("theme");
							else __e92.setAttribute("theme", __av93 === true ? "" : String(__av93));
						}
						if (__old.paintMode !== $scope.row.paintMode) {
							const __av94 = $scope.row.paintMode;
							if (__av94 == null || __av94 === false || __av94 === "") __e92.removeAttribute("mode");
							else __e92.setAttribute("mode", __av94 === true ? "" : String(__av94));
						}
						if (__old.name !== $scope.row.name) __t97.textContent = $scope.row.name;
						if (__old.meta !== $scope.row.meta) __t99.textContent = $scope.row.meta;
						if (__old.menuId !== $scope.row.menuId) {
							{
								const __av104 = $scope.row.menuId;
								if (__av104 == null || __av104 === false || __av104 === "") __e103.removeAttribute("popovertarget");
								else __e103.setAttribute("popovertarget", __av104 === true ? "" : String(__av104));
							}
							{
								const __av108 = $scope.row.menuId;
								if (__av108 == null || __av108 === false || __av108 === "") __e107.removeAttribute("id");
								else __e107.setAttribute("id", __av108 === true ? "" : String(__av108));
							}
						}
					}
				};
			})(__childScope84);
		}
	});
	const __wireBoundary_b25 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.hasCatalogRows()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag78 = document.createDocumentFragment();
			const __c79 = [];
			const __t80 = document.createTextNode("\n            ");
			__frag78.appendChild(__t80);
			const __e81 = __tpl7();
			__frag78.appendChild(__e81);
			const __t82 = document.createTextNode("\n            ");
			__frag78.appendChild(__t82);
			const __forAnchor83 = document.createComment("for");
			__frag78.appendChild(__forAnchor83);
			__c79.push(__wireBoundary_b26(__forAnchor83));
			const __t112 = document.createTextNode("\n          ");
			__frag78.appendChild(__t112);
			return {
				nodes: Array.from(__frag78.childNodes),
				cleanup: () => {
					for (const c of __c79) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b41 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.hasSelected()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag116 = document.createDocumentFragment();
			const __c117 = [];
			const __t118 = document.createTextNode("\n          ");
			__frag116.appendChild(__t118);
			const __e119 = __tpl9();
			runAction(__e119, variantLive, void 0);
			__frag116.appendChild(__e119);
			const __t120 = document.createTextNode("\n        ");
			__frag116.appendChild(__t120);
			return {
				nodes: Array.from(__frag116.childNodes),
				cleanup: () => {
					for (const c of __c117) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b44 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.previewStatusLabel()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag130 = document.createDocumentFragment();
			const __c131 = [];
			const __t132 = document.createTextNode("\n                ");
			__frag130.appendChild(__t132);
			const __e133 = __tpl11();
			__c131.push(bindAttr(__e133, "tone", () => $scope.previewStatusTone()));
			const __t134 = __e133.firstChild;
			markHydrateText(__t134, "b46");
			__c131.push(bindText(__t134, () => $scope.previewStatusLabel()));
			__frag130.appendChild(__e133);
			const __t135 = document.createTextNode("\n              ");
			__frag130.appendChild(__t135);
			return {
				nodes: Array.from(__frag130.childNodes),
				cleanup: () => {
					for (const c of __c131) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b50 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.canToggleMode()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag143 = document.createDocumentFragment();
			const __c144 = [];
			const __t145 = document.createTextNode("\n              ");
			__frag143.appendChild(__t145);
			const __e146 = __tpl15();
			const __e147 = __e146.firstChild.nextSibling;
			runAction(__e147, icon, void 0);
			const __e148 = __e147.nextSibling;
			__c144.push(bindProp(__e148, "checked", () => $scope.modeDarkOn()));
			__c144.push(bindAttr(__e148, "aria-label", () => $scope.modeDarkOn() ? "Switch to light mode" : "Switch to dark mode"));
			__e148.$$change = (function() {
				const __h = $scope.onPreviewModeSwitch;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			const __e149 = __e148.nextSibling;
			runAction(__e149, icon, void 0);
			__frag143.appendChild(__e146);
			const __t150 = document.createTextNode("\n            ");
			__frag143.appendChild(__t150);
			return {
				nodes: Array.from(__frag143.childNodes),
				cleanup: () => {
					for (const c of __c144) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b54 = (anchor) => bindIf({
		anchor,
		test: () => {
			if (!$scope.canToggleMode()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag152 = document.createDocumentFragment();
			const __c153 = [];
			const __t154 = document.createTextNode("\n              ");
			__frag152.appendChild(__t154);
			const __e155 = document.createElement("p");
			__e155.setAttribute("mode-fixed", "");
			__e155.setAttribute("class", "sx-fgmutedforeground sx-fontSizexs");
			const __t156 = document.createTextNode("\n                \n                This theme ships ");
			__e155.appendChild(__t156);
			const __t157 = document.createTextNode("");
			markHydrateText(__t157, "b55");
			__e155.appendChild(__t157);
			__c153.push(bindText(__t157, () => $scope.previewModesLabel().toLowerCase()));
			const __t158 = document.createTextNode(" — preview uses that table.\n              ");
			__e155.appendChild(__t158);
			__frag152.appendChild(__e155);
			const __t159 = document.createTextNode("\n            ");
			__frag152.appendChild(__t159);
			return {
				nodes: Array.from(__frag152.childNodes),
				cleanup: () => {
					for (const c of __c153) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b56 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.previewInProject()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag162 = document.createDocumentFragment();
			const __c163 = [];
			const __t164 = document.createTextNode("\n                ");
			__frag162.appendChild(__t164);
			const __e165 = __tpl16();
			__e165.$$click = (function() {
				const __h = $scope.onPreviewOpen;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag162.appendChild(__e165);
			const __t166 = document.createTextNode("\n              ");
			__frag162.appendChild(__t166);
			return {
				nodes: Array.from(__frag162.childNodes),
				cleanup: () => {
					for (const c of __c163) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b58 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.previewIsCatalog()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag168 = document.createDocumentFragment();
			const __c169 = [];
			const __t170 = document.createTextNode("\n                ");
			__frag168.appendChild(__t170);
			const __e171 = __tpl17();
			__e171.$$click = (function() {
				const __h = $scope.onPreviewAddToProject;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag168.appendChild(__e171);
			const __t172 = document.createTextNode("\n                ");
			__frag168.appendChild(__t172);
			const __e173 = __tpl18();
			__e173.$$click = (function() {
				const __h = $scope.onPreviewOpen;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__frag168.appendChild(__e173);
			const __t174 = document.createTextNode("\n              ");
			__frag168.appendChild(__t174);
			return {
				nodes: Array.from(__frag168.childNodes),
				cleanup: () => {
					for (const c of __c169) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b42 = (anchor) => bindIf({
		anchor,
		test: () => {
			if ($scope.hasSelected()) return 0;
			return -1;
		},
		branches: [() => {
			const __frag122 = document.createDocumentFragment();
			const __c123 = [];
			const __t124 = document.createTextNode("\n          ");
			__frag122.appendChild(__t124);
			const __e125 = document.createElement("div");
			__e125.setAttribute("class", "sail-col sail-g-3 sail-ai-stretch");
			__e125.setAttribute("preview-head", "");
			const __e126 = document.createElement("div");
			__e126.setAttribute("class", "sail-row sail-g-2 sail-ai-center sail-jc-start");
			const __e127 = __tpl10();
			const __t128 = __e127.firstChild;
			markHydrateText(__t128, "b43");
			__c123.push(bindText(__t128, () => $scope.previewTitle()));
			__e126.appendChild(__e127);
			const __ifAnchor129 = document.createComment("if");
			__e126.appendChild(__ifAnchor129);
			__c123.push(__wireBoundary_b44(__ifAnchor129));
			const __e136 = __tpl12();
			const __t137 = __e136.firstChild;
			markHydrateText(__t137, "b47");
			__c123.push(bindText(__t137, () => $scope.previewModesLabel()));
			__e126.appendChild(__e136);
			__e125.appendChild(__e126);
			const __e138 = __tpl13();
			const __t139 = __e138.firstChild;
			markHydrateText(__t139, "b48");
			__c123.push(bindText(__t139, () => $scope.previewDesc()));
			__e125.appendChild(__e138);
			const __e140 = __tpl14();
			const __t141 = __e140.firstChild;
			markHydrateText(__t141, "b49");
			__c123.push(bindText(__t141, () => $scope.previewMeta()));
			__e125.appendChild(__e140);
			const __ifAnchor142 = document.createComment("if");
			__e125.appendChild(__ifAnchor142);
			__c123.push(__wireBoundary_b50(__ifAnchor142));
			const __ifAnchor151 = document.createComment("if");
			__e125.appendChild(__ifAnchor151);
			__c123.push(__wireBoundary_b54(__ifAnchor151));
			const __e160 = document.createElement("div");
			__e160.setAttribute("class", "sail-row sail-g-2 sail-ai-center sail-jc-start");
			const __ifAnchor161 = document.createComment("if");
			__e160.appendChild(__ifAnchor161);
			__c123.push(__wireBoundary_b56(__ifAnchor161));
			const __ifAnchor167 = document.createComment("if");
			__e160.appendChild(__ifAnchor167);
			__c123.push(__wireBoundary_b58(__ifAnchor167));
			__e125.appendChild(__e160);
			__frag122.appendChild(__e125);
			const __t175 = document.createTextNode("\n\n          ");
			__frag122.appendChild(__t175);
			const __t176 = document.createTextNode("\n          ");
			__frag122.appendChild(__t176);
			const __e177 = __tpl19();
			const __e178 = __e177.firstChild.nextSibling;
			__c123.push(bindAttr(__e178, "theme", () => $scope.previewPaintId()));
			__c123.push(bindAttr(__e178, "mode", () => $scope.previewModeId()));
			__frag122.appendChild(__e177);
			const __t179 = document.createTextNode("\n\n          ");
			__frag122.appendChild(__t179);
			const __t180 = document.createTextNode("\n          ");
			__frag122.appendChild(__t180);
			const __e181 = __tpl20();
			__c123.push(bindAttr(__e181, "theme", () => $scope.previewPaintId()));
			__c123.push(bindAttr(__e181, "mode", () => $scope.previewModeId()));
			__frag122.appendChild(__e181);
			const __t182 = document.createTextNode("\n\n          ");
			__frag122.appendChild(__t182);
			const __t183 = document.createTextNode("\n          ");
			__frag122.appendChild(__t183);
			const __e184 = __tpl21();
			const __e185 = __e184.firstChild.nextSibling;
			__c123.push(bindAttr(__e185, "theme", () => $scope.previewPaintId()));
			__c123.push(bindAttr(__e185, "mode", () => $scope.previewModeId()));
			const __t188 = __e185.firstChild.firstChild.firstChild;
			markHydrateText(__t188, "b67");
			__c123.push(bindText(__t188, () => $scope.previewTitle()));
			__frag122.appendChild(__e184);
			const __t189 = document.createTextNode("\n        ");
			__frag122.appendChild(__t189);
			return {
				nodes: Array.from(__frag122.childNodes),
				cleanup: () => {
					for (const c of __c123) try {
						c();
					} catch (e) {}
				}
			};
		}]
	});
	const __wireBoundary_b68 = (anchor) => bindFor({
		anchor,
		items: () => $scope.toastItems(),
		key: (t, __i) => t.key,
		render: (t, __i) => {
			const __childScope192 = Object.create($scope);
			__childScope192.t = t;
			__childScope192.__i = __i;
			return (($scope) => {
				const __frag193 = document.createDocumentFragment();
				const __c194 = [];
				const __t195 = document.createTextNode("\n      ");
				__frag193.appendChild(__t195);
				const __e196 = __tpl22();
				runAction(__e196, autoDismiss, void 0);
				runAction(__e196, dismissControl, void 0);
				{
					const __av197 = $scope.t.tone;
					if (__av197 == null || __av197 === false || __av197 === "") __e196.removeAttribute("tone");
					else __e196.setAttribute("tone", __av197 === true ? "" : String(__av197));
				}
				__c194.push(on(__e196, "sail-toast-dismiss", (function() {
					const __h = $scope.onToastDismiss;
					return typeof __h === "function" ? __h.bind(void 0) : (() => {});
				})(), { modifiers: [] }));
				const __t198 = __e196.firstChild;
				markHydrateText(__t198, "b71");
				__t198.textContent = $scope.t.message;
				__frag193.appendChild(__e196);
				const __t199 = document.createTextNode("\n    ");
				__frag193.appendChild(__t199);
				return {
					nodes: Array.from(__frag193.childNodes),
					cleanup: () => {
						for (const c of __c194) try {
							c();
						} catch (e) {}
					},
					update: (__new200, __newI201) => {
						const __old = $scope.t;
						$scope.t = __new200;
						$scope.__i = __newI201;
						if (__old.tone !== $scope.t.tone) {
							const __av197 = $scope.t.tone;
							if (__av197 == null || __av197 === false || __av197 === "") __e196.removeAttribute("tone");
							else __e196.setAttribute("tone", __av197 === true ? "" : String(__av197));
						}
						if (__old.message !== $scope.t.message) __t198.textContent = $scope.t.message;
					}
				};
			})(__childScope192);
		}
	});
	return Object.assign({}, $scope, {
		__mount(container) {
			const __cleanups = [];
			const __root = container.shadowRoot ?? container;
			const __e0 = document.createElement("div");
			__e0.setAttribute("class", "sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minH0 sx-hfull sx-minW0 sx-overflowhidden sx-bgbackground");
			__e0.setAttribute("surface", "themes");
			__e0.setAttribute("themes-surface", "");
			__e0.classList.add("sail-root");
			const __e1 = __tpl0$1();
			const __e3 = __e1.firstChild.nextSibling.firstChild;
			__e3.$$click = (function() {
				const __h = $scope.onNewTheme;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e0.appendChild(__e1);
			const __ifAnchor4 = document.createComment("if");
			__e0.appendChild(__ifAnchor4);
			__cleanups.push(__wireBoundary_b1(__ifAnchor4));
			const __ifAnchor13 = document.createComment("if");
			__e0.appendChild(__ifAnchor13);
			__cleanups.push(__wireBoundary_b4(__ifAnchor13));
			const __e21 = document.createElement("div");
			__e21.setAttribute("class", "sail-row sail-g-0 sail-ai-stretch sail-jc-start sail-nowrap sx-grow1 sx-minH0 sx-basis0 sx-hfull sx-minW0 sx-overflowhidden");
			__e21.setAttribute("themes-workbench", "");
			__e21.setAttribute("themes-split", "");
			const __e22 = document.createElement("div");
			__e22.setAttribute("class", "sail-col sail-g-0 sail-ai-stretch sx-grow1 sx-minH0 sx-minWpanelminwmd sx-maxWpanelmaxwmd sx-basis0 sx-borderRWidth1 sx-borderRbordersubtle sx-bgsurface1");
			__e22.setAttribute("themes-list-pane", "");
			const __e23 = document.createElement("div");
			__e23.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch sx-shrink0 sx-padX4 sx-padT4 sx-padB3 sx-borderBWidth1 sx-borderBbordersubtle");
			__e23.setAttribute("themes-list-head", "");
			const __e24 = __tpl3();
			__e23.appendChild(__e24);
			const __e25 = document.createElement("p");
			__e25.setAttribute("class", "sx-fgmutedforeground sx-fontSizexs");
			const __t26 = document.createTextNode("");
			markHydrateText(__t26, "b6");
			__e25.appendChild(__t26);
			__cleanups.push(bindText(__t26, () => $scope.projectCountLabel()));
			const __t27 = document.createTextNode(" in project · ");
			__e25.appendChild(__t27);
			const __t28 = document.createTextNode("");
			markHydrateText(__t28, "b7");
			__e25.appendChild(__t28);
			__cleanups.push(bindText(__t28, () => $scope.catalogCountLabel()));
			const __t29 = document.createTextNode(" catalog\n        ");
			__e25.appendChild(__t29);
			__e23.appendChild(__e25);
			__e22.appendChild(__e23);
			const __e30 = document.createElement("div");
			__e30.setAttribute("class", "sx-grow1 sail-scroll-area sx-overscrollcontain sx-scrollbarThin sx-scrollbarThumbscrollareathumb sx-scrollbarTracktransparent sx-orientationvertical-overflowXhidden sx-orientationvertical-overflowYauto sx-orientationhorizontal-overflowXauto sx-orientationhorizontal-overflowYhidden sx-orientationboth-overflowauto sx-hover-scrollbarThumbscrollareathumbhover sx-minH0 sx-basis0 sx-hfull");
			__e30.setAttribute("data-sail-host", "scroll-area");
			__e30.setAttribute("themes-list", "");
			__e30.$$click = (function() {
				const __h = $scope.onListClick;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})();
			__e30.setAttribute("orientation", "vertical");
			const __e31 = document.createElement("div");
			__e31.setAttribute("class", "sail-col sail-g-1 sail-ai-stretch sx-pad3");
			const __ifAnchor32 = document.createComment("if");
			__e31.appendChild(__ifAnchor32);
			__cleanups.push(__wireBoundary_b9(__ifAnchor32));
			const __ifAnchor43 = document.createComment("if");
			__e31.appendChild(__ifAnchor43);
			__cleanups.push(__wireBoundary_b11(__ifAnchor43));
			const __ifAnchor77 = document.createComment("if");
			__e31.appendChild(__ifAnchor77);
			__cleanups.push(__wireBoundary_b25(__ifAnchor77));
			__e30.appendChild(__e31);
			__e22.appendChild(__e30);
			__e21.appendChild(__e22);
			const __e113 = document.createElement("div");
			__e113.setAttribute("class", "sx-grow1 sx-minW0 sail-scroll-area sx-overscrollcontain sx-scrollbarThin sx-scrollbarThumbscrollareathumb sx-scrollbarTracktransparent sx-orientationvertical-overflowXhidden sx-orientationvertical-overflowYauto sx-orientationhorizontal-overflowXauto sx-orientationhorizontal-overflowYhidden sx-orientationboth-overflowauto sx-hover-scrollbarThumbscrollareathumbhover sx-minH0 sx-basis0 sx-hfull sx-bgbackground");
			__e113.setAttribute("data-sail-host", "scroll-area");
			__e113.setAttribute("themes-preview", "");
			__e113.setAttribute("theme-preview-pane", "");
			__e113.setAttribute("orientation", "vertical");
			const __e114 = document.createElement("div");
			__e114.setAttribute("class", "sail-col sail-g-6 sail-ai-stretch sx-padX8 sx-padT6 sx-padB10 sx-maxWpanelmaxwlg");
			const __ifAnchor115 = document.createComment("if");
			__e114.appendChild(__ifAnchor115);
			__cleanups.push(__wireBoundary_b41(__ifAnchor115));
			const __ifAnchor121 = document.createComment("if");
			__e114.appendChild(__ifAnchor121);
			__cleanups.push(__wireBoundary_b42(__ifAnchor121));
			__e113.appendChild(__e114);
			__e21.appendChild(__e113);
			__e0.appendChild(__e21);
			const __e190 = document.createElement("div");
			__e190.setAttribute("class", "sail-toast-region sail-col sail-g-2 sail-ai-stretch sx-positionfixed sx-zztoast sx-pad4 sx-pointerEventsnone sx-top0 sx-rightauto sx-bottomauto sx-left0 sx-wpanelwtoast sx-positiontopstart-top0 sx-positiontopstart-rightauto sx-positiontopstart-bottomauto sx-positiontopstart-left0 sx-positiontopend-top0 sx-positiontopend-right0 sx-positiontopend-bottomauto sx-positiontopend-leftauto sx-positionbottomstart-topauto sx-positionbottomstart-rightauto sx-positionbottomstart-bottom0 sx-positionbottomstart-left0 sx-positionbottomstart-flexDircolumnreverse sx-positionbottomend-topauto sx-positionbottomend-right0 sx-positionbottomend-bottom0 sx-positionbottomend-leftauto sx-positionbottomend-flexDircolumnreverse");
			__e190.setAttribute("data-sail-host", "toast-region");
			__e190.setAttribute("position", "bottom-end");
			__e190.setAttribute("themes-toast-region", "");
			const __forAnchor191 = document.createComment("for");
			__e190.appendChild(__forAnchor191);
			__cleanups.push(__wireBoundary_b68(__forAnchor191));
			__e0.appendChild(__e190);
			__root.appendChild(__e0);
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		},
		hydrate(root, ctx) {
			const __cleanups = [];
			const __t = collectTextOpens(root);
			const __m = collectElementMarkers(root);
			if (__m["b0"]) __cleanups.push(hydrateOn(__m["b0"], "click", (function() {
				const __h = $scope.onNewTheme;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b1"]) {
				const __t = __m["b1"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b1(__anchor));
			}
			if (__m["b4"]) {
				const __t = __m["b4"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b4(__anchor));
			}
			if (__t["b6"]) __cleanups.push(hydrateText(__t["b6"], () => $scope.projectCountLabel()));
			if (__t["b7"]) __cleanups.push(hydrateText(__t["b7"], () => $scope.catalogCountLabel()));
			if (__m["b8"]) __cleanups.push(hydrateOn(__m["b8"], "click", (function() {
				const __h = $scope.onListClick;
				return typeof __h === "function" ? __h.bind(void 0) : (() => {});
			})(), []));
			if (__m["b9"]) {
				const __t = __m["b9"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b9(__anchor));
			}
			if (__m["b11"]) {
				const __t = __m["b11"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b11(__anchor));
			}
			if (__m["b25"]) {
				const __t = __m["b25"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b25(__anchor));
			}
			if (__m["b41"]) {
				const __t = __m["b41"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b41(__anchor));
			}
			if (__m["b42"]) {
				const __t = __m["b42"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b42(__anchor));
			}
			if (__m["b68"]) {
				const __t = __m["b68"];
				const __doc = __t.ownerDocument ?? document;
				const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
				if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
				__cleanups.push(__wireBoundary_b68(__anchor));
			}
			return () => {
				for (const c of __cleanups) try {
					c();
				} catch (e) {}
			};
		}
	});
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});
component("themes-surface", { styles: ".sx-minH0{min-height:0}.sx-hfull{height:100%}.sx-minW0{min-width:0}.sx-overflowhidden{overflow:hidden}.sx-bgbackground{background-color:var(--background)}.sx-padX6{padding-inline:var(--space-6)}.sx-padY4{padding-block:var(--space-4)}.sx-borderBWidth1{border-bottom-width:1px}.sx-borderBbordersubtle{border-bottom-color:var(--border-subtle)}.sx-bgsurface1{background-color:var(--surface-1)}.sx-fgforeground{color:var(--foreground)}.sx-fontSizelg{font-size:var(--text-lg)}.sx-fontWeightsemibold{font-weight:var(--font-weight-semibold)}.sx-trackingtight{letter-spacing:var(--tracking-tight)}.sx-fgmutedforeground{color:var(--muted-foreground)}.sx-fontSizesm{font-size:var(--text-sm)}.sx-leadingsnug{line-height:var(--leading-snug)}.sx-padY2{padding-block:var(--space-2)}.sx-borderBborder{border-bottom-color:var(--border)}.sx-fgdestructive{color:var(--destructive)}.sx-truncate{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.sx-basis0{flex-basis:0}.sx-minWpanelminwmd{min-width:var(--panel-min-w-md)}.sx-maxWpanelmaxwmd{max-width:var(--panel-max-w-md)}.sx-borderRWidth1{border-right-width:1px}.sx-borderRbordersubtle{border-right-color:var(--border-subtle)}.sx-padX4{padding-inline:var(--space-4)}.sx-padT4{padding-top:var(--space-4)}.sx-padB3{padding-bottom:var(--space-3)}.sx-fontSizexs{font-size:var(--text-xs)}.sx-fontWeightbold{font-weight:var(--font-weight-bold)}.sx-textCaseuppercase{text-transform:uppercase}.sx-trackingwide{letter-spacing:var(--tracking-wide)}.sx-pad3{padding:var(--space-3)}.sx-padX2{padding-inline:var(--space-2)}.sx-padT1{padding-top:var(--space-1)}.sx-padB1{padding-bottom:var(--space-1)}.sx-pad2{padding:var(--space-2)}.sx-radiusmd{border-radius:var(--radius-md)}.sx-cursorpointer{cursor:pointer}.sx-borderWidth1{border-width:1px}.sx-bordertransparent{border-color:transparent}.sx-hover-bghoverbg:hover{background-color:var(--hover-bg)}.sx-current-bgselectedbg[aria-current]:not([aria-current='false']){background-color:var(--selected-bg)}.sx-current-borderselectedborder[aria-current]:not([aria-current='false']){border-color:var(--selected-border)}.sx-w14{width:var(--space-14)}.sx-h7{height:var(--space-7)}.sx-minW14{min-width:var(--space-14)}.sx-radiussm{border-radius:var(--radius-sm)}.sx-borderbordersubtle{border-color:var(--border-subtle)}.sx-bgsurface2{background-color:var(--surface-2)}.sx-bgprimary{background-color:var(--primary)}.sx-bgaccent{background-color:var(--accent)}.sx-padX8{padding-inline:var(--space-8)}.sx-padT6{padding-top:var(--space-6)}.sx-padB10{padding-bottom:var(--space-10)}.sx-maxWpanelmaxwlg{max-width:var(--panel-max-w-lg)}.sx-fontFamilydisplay{font-family:var(--display-font)}.sx-fontSize2xl{font-size:var(--text-2xl)}.sx-fontWeightmedium{font-weight:var(--font-weight-medium)}.sx-leadingrelaxed{line-height:var(--leading-relaxed)}.sx-h16{height:var(--space-16)}.sx-bgsecondary{background-color:var(--secondary)}.sx-pad4{padding:var(--space-4)}.sx-radiuslg{border-radius:var(--radius-lg)}.sx-fontSizexl{font-size:var(--text-xl)}.sx-fontFamilybody{font-family:var(--body-font)}.sx-fontFamilymono{font-family:var(--mono-font)}.sx-w8{width:var(--space-8)}.sx-h8{height:var(--space-8)}.sx-bgtransparent{background-color:transparent}.sx-borderWidth2{border-width:2px}.sx-borderprimary{border-color:var(--primary)}.sx-radiusxl{border-radius:var(--radius-xl)}.sx-borderborder{border-color:var(--border)}.sx-shadowmd{box-shadow:var(--shadow-md)}.sx-h9{height:var(--space-9)}.sx-pad5{padding:var(--space-5)}.sx-fontSizemd{font-size:var(--text-md)}.sx-fontFamilyheading{font-family:var(--heading-font)}.sx-grow1{flex-grow:1}.sx-shrink0{flex-shrink:0}.sx-wfull{width:100%}.sail-col{display:flex;flex-direction:column;min-width:0}.sail-col>*{min-width:0}.sail-col[popover]:not(:popover-open){display:none!important}.sail-col[popover]:popover-open{display:flex;flex-direction:column;min-width:0}.sail-g-0{gap:0}.sail-ai-stretch{align-items:stretch}.sail-row{display:flex;flex-wrap:wrap;min-width:0}.sail-row>*{min-width:0}.sail-row[popover]:not(:popover-open){display:none!important}.sail-row[popover]:popover-open{display:flex;flex-wrap:wrap;min-width:0}.sail-g-4{gap:var(--space-4)}.sail-ai-center{align-items:center}.sail-jc-between{justify-content:space-between}.sail-row.sail-nowrap{flex-wrap:nowrap}.sail-g-1{gap:var(--space-1)}.sail-g-2{gap:var(--space-2)}.sail-jc-start{justify-content:flex-start}.sail-g-3{gap:var(--space-3)}.sail-center{display:grid;place-items:center;place-content:center;width:100%}.sail-g-6{gap:var(--space-6)}.sail-ai-end{align-items:flex-end}" }, __default);

//#endregion
//#region routes/themes/page.sail
/*! _compilerVersion {"cargo":"0.1.0","git":"c11a87c8","dirty":true,"builtAt":"2026-08-05T21:58:16Z"} */
const __tpl0 = /*@__PURE__*/ __tpl("<themes-surface class=\"sx-grow1 sx-minH0 sx-hfull sx-minW0 sx-overflowhidden\">\n  </themes-surface>");
var page_default = Object.assign(component((ctx) => {
	const { route, navigate, providers } = ctx;
	const $scope = Object.create(null);
	$scope.route = route;
	$scope.navigate = navigate;
	for (const __k of Object.keys(ctx)) {
		if (__k === "route" || __k === "navigate" || __k === "scope" || __k === "stateKeys" || __k === "host" || __k === "providers" || __k === "props") continue;
		$scope[__k] = ctx[__k];
	}
	if (providers && typeof providers === "object") Object.assign($scope, providers);
	return Object.assign({}, $scope, { __mount(container) {
		const __cleanups = [];
		const __root = container.shadowRoot ?? container;
		const __suiStylesKey = "37f1aea1";
		const __suiStyles = "@layer sail {\n.sx-minH0{min-height:0}.sx-hfull{height:100%}.sx-minW0{min-width:0}.sx-overflowhidden{overflow:hidden}.sx-grow1{flex-grow:1}\n}";
		{
			let __existing = null;
			let __n = container.firstChild;
			while (__n) {
				if (__n.nodeType === 1 && __n.tagName === "STYLE" && __n.getAttribute("data-sail-style") === __suiStylesKey) {
					__existing = __n;
					break;
				}
				__n = __n.nextSibling;
			}
			if (!__existing) {
				const __el = document.createElement("style");
				__el.setAttribute("data-sail-style", __suiStylesKey);
				__el.textContent = __suiStyles;
				container.insertBefore(__el, container.firstChild);
			}
		}
		const __e0 = __tpl0();
		__root.appendChild(__e0);
		return () => {
			for (const c of __cleanups) try {
				c();
			} catch (e) {}
		};
	} });
}), {
	_compilerVersion: {
		"cargo": "0.1.0",
		"git": "c11a87c8",
		"dirty": true,
		"builtAt": "2026-08-05T21:58:16Z"
	},
	clients: []
});

//#endregion
export { page_default as default };