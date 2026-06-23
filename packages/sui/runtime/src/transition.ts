// Enter / exit animation primitives.
//
// The ship-set compiler turns `transition:<name>={cfg}` (and the
// asymmetric `in:` / `out:` directives) into a `registerTransition`
// call on the element at construction time. The {#if} / {#for}
// runtimes then call `runEnter` after inserting nodes and `runLeave`
// before removing them (deferring DOM removal until the out-animation
// completes) — they reach those via the `transition-runner.ts`
// indirection, which holds stubs by default. This module's top-level
// `setTransitionRunner` call (bottom of file) swaps the stubs for
// the real impls as a side effect of being imported. Apps that don't
// touch transitions can tree-shake this whole module out of their
// bundle — only the ~80B stubs from transition-runner survive.
//
// Built-ins (`fade`, `slide`, `scale`) use the Web Animations API
// (`Element.animate`) so there's no JS interpolation cost. Custom
// transitions export an object matching the `Transition` interface.

import { setTransitionRunner } from './transition-runner'

/** Per-call configuration. Built-ins read `duration`/`easing`/`delay`
 *  plus their own extra options (e.g. `slide.axis`, `scale.from`). */
export interface TransitionConfig {
  duration?: number
  easing?: string
  delay?: number
  // Allow built-in-specific options through without losing type info
  // for casts inside the factory. Authors may extend the interface in
  // their own custom transitions.
  [key: string]: unknown
}

/** A transition factory. `in` is the enter phase; `out` is the exit
 *  phase. Either may be omitted — the runtime treats the missing side
 *  as a no-op. */
export interface Transition {
  in?: (node: Element, opts: TransitionConfig) => Animation | undefined
  out?: (node: Element, opts: TransitionConfig) => Animation | undefined
}

/** Which phase(s) a registration applies to. `transition:` is `both`;
 *  `in:` is `in`; `out:` is `out`. */
export type TransitionDir = 'both' | 'in' | 'out'

interface Registration {
  inT?: Transition
  outT?: Transition
  inOpts: TransitionConfig
  outOpts: TransitionConfig
  current?: Animation | undefined
}

const REG = Symbol('sui:transition')
const MARK = 'data-sui-transition'

function getReg(node: Element): Registration | undefined {
  return (node as unknown as { [REG]?: Registration })[REG]
}

function ensureReg(node: Element): Registration {
  const obj = node as unknown as { [REG]?: Registration }
  let r = obj[REG]
  if (!r) {
    r = { inOpts: {}, outOpts: {} }
    obj[REG] = r
  }
  return r
}

/** Attach a `Transition` factory to an element. Idempotent; called
 *  once per directive when the element is created (or hydrated). */
export function registerTransition(
  node: Element,
  transition: Transition,
  opts: TransitionConfig = {},
  dir: TransitionDir = 'both',
): void {
  const r = ensureReg(node)
  if (dir === 'in' || dir === 'both') {
    r.inT = transition
    r.inOpts = opts
  }
  if (dir === 'out' || dir === 'both') {
    r.outT = transition
    r.outOpts = opts
  }
  if (!node.hasAttribute(MARK)) node.setAttribute(MARK, '')
}

function reducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false
  }
  try {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches
  } catch {
    return false
  }
}

function applyReduced(o: TransitionConfig): TransitionConfig {
  if (!reducedMotion()) return o
  return { ...o, duration: 0, delay: 0 }
}

/** Run a single element's in-phase. Cancels any in-flight animation
 *  on the element so a re-mount during out picks up from current
 *  state. Returns the started `Animation`, or `undefined` when no
 *  in-factory is registered. */
export function runIn(node: Element): Animation | undefined {
  const r = getReg(node)
  if (!r?.inT?.in) return undefined
  r.current?.cancel()
  const anim = r.inT.in(node, applyReduced(r.inOpts))
  r.current = anim
  return anim
}

/** Run a single element's out-phase. Returns the started `Animation`,
 *  or `undefined` when no out-factory is registered (caller should
 *  remove the node immediately in that case). */
export function runOut(node: Element): Animation | undefined {
  const r = getReg(node)
  if (!r?.outT?.out) return undefined
  r.current?.cancel()
  const anim = r.outT.out(node, applyReduced(r.outOpts))
  r.current = anim
  return anim
}

/** Cancel the in-flight animation on `node` and its descendants. Used
 *  when a departing subtree is re-mounted before its out-transition
 *  finished — the next `runEnter` starts the in-phase fresh. */
function cancelTransitions(node: Node): void {
  if (node.nodeType !== 1) return
  const root = node as Element
  if (root.hasAttribute(MARK)) {
    const r = getReg(root)
    r?.current?.cancel()
    if (r) r.current = undefined
  }
  for (const el of root.querySelectorAll(`[${MARK}]`)) {
    const r = getReg(el)
    r?.current?.cancel()
    if (r) r.current = undefined
  }
}

function forEachRegisteredElement(
  nodes: Iterable<Node>,
  fn: (el: Element) => void,
): void {
  for (const n of nodes) {
    if (n.nodeType !== 1) continue
    const el = n as Element
    if (el.hasAttribute(MARK)) fn(el)
    for (const d of el.querySelectorAll(`[${MARK}]`)) fn(d)
  }
}

/** Run the in-phase on every registered element inside `nodes`
 *  (including the nodes themselves). Returns nothing — the caller
 *  doesn't need to await enter animations. */
function runEnter(nodes: Iterable<Node>): void {
  forEachRegisteredElement(nodes, el => { runIn(el) })
}

/** Run the out-phase on every registered element inside `nodes`.
 *  Returns a Promise that resolves when every started animation
 *  finishes (or is cancelled), or `null` when no out-transitions were
 *  started — in the `null` case the caller should remove the nodes
 *  immediately (matches the no-transition fast path). */
function runLeave(nodes: Iterable<Node>): Promise<void> | null {
  const promises: Promise<unknown>[] = []
  forEachRegisteredElement(nodes, el => {
    const a = runOut(el)
    if (a) promises.push(a.finished.catch(() => {}))
  })
  if (promises.length === 0) return null
  return Promise.all(promises).then(() => undefined)
}

const DEFAULT_DURATION = 200
const DEFAULT_EASING = 'cubic-bezier(0.4, 0, 0.2, 1)'
const DEFAULT_DELAY = 0

interface ResolvedTiming {
  duration: number
  easing: string
  delay: number
  fill: 'both'
}

function timing(o: TransitionConfig): ResolvedTiming {
  return {
    duration: typeof o.duration === 'number' ? o.duration : DEFAULT_DURATION,
    easing: typeof o.easing === 'string' ? o.easing : DEFAULT_EASING,
    delay: typeof o.delay === 'number' ? o.delay : DEFAULT_DELAY,
    fill: 'both',
  }
}

function animateSafe(node: Element, frames: Keyframe[], t: ResolvedTiming): Animation | undefined {
  // jsdom and very old browsers may not implement Element.animate.
  // Without it we can't drive a transition — return undefined and let
  // the caller treat this as "no transition" (remove immediately /
  // skip in-phase).
  const fn = (node as Element & { animate?: (frames: Keyframe[], opts: KeyframeAnimationOptions) => Animation }).animate
  if (typeof fn !== 'function') return undefined
  return fn.call(node, frames, t)
}

export const fade: Transition = {
  in: (node, o) => animateSafe(node, [{ opacity: 0 }, { opacity: 1 }], timing(o)),
  out: (node, o) => animateSafe(node, [{ opacity: 1 }, { opacity: 0 }], timing(o)),
}

export interface SlideConfig extends TransitionConfig {
  axis?: 'x' | 'y'
}

function slideFrames(axis: 'x' | 'y', dir: 'in' | 'out'): Keyframe[] {
  const off = axis === 'y' ? 'translateY(-100%)' : 'translateX(-100%)'
  const on = 'translate(0, 0)'
  return dir === 'in'
    ? [{ opacity: 0, transform: off }, { opacity: 1, transform: on }]
    : [{ opacity: 1, transform: on }, { opacity: 0, transform: off }]
}

export const slide: Transition = {
  in: (node, o) => {
    const axis = (o as SlideConfig).axis === 'x' ? 'x' : 'y'
    return animateSafe(node, slideFrames(axis, 'in'), timing(o))
  },
  out: (node, o) => {
    const axis = (o as SlideConfig).axis === 'x' ? 'x' : 'y'
    return animateSafe(node, slideFrames(axis, 'out'), timing(o))
  },
}

export interface ScaleConfig extends TransitionConfig {
  from?: number
}

function scaleFrames(from: number, dir: 'in' | 'out'): Keyframe[] {
  const off = { opacity: 0, transform: `scale(${from})` }
  const on = { opacity: 1, transform: 'scale(1)' }
  return dir === 'in' ? [off, on] : [on, off]
}

export const scale: Transition = {
  in: (node, o) => {
    const from = typeof (o as ScaleConfig).from === 'number' ? (o as ScaleConfig).from! : 0
    return animateSafe(node, scaleFrames(from, 'in'), timing(o))
  },
  out: (node, o) => {
    const from = typeof (o as ScaleConfig).from === 'number' ? (o as ScaleConfig).from! : 0
    return animateSafe(node, scaleFrames(from, 'out'), timing(o))
  },
}

// Side-effect register: swap transition-runner's stubs for the real
// impls now that this module is loaded. From this point bindFor and
// bindIf call the live runner (built-in fade/slide/scale + any
// user-registered transitions). Idempotent on re-import.
//
// The `/* @__PURE__ */` marker tells esbuild + bun that the call's
// SIDE EFFECT is droppable when the rest of the module's exports
// aren't consumed by the bundle. This is the lie that unlocks
// tree-shaking: when only stub-importers reach this file (so no
// `fade/slide/scale/registerTransition` is needed), the whole
// transition.ts disappears from the bundle — ~870B win on the
// Krausest bench. When a real consumer pulls anything from this
// file, the side-effect call lands as normal and the runner gets
// swapped to the real impls.
/* @__PURE__ */ setTransitionRunner({ runEnter, runLeave, cancelTransitions })
