// Motion runtime — the single authority for every animation the framework
// owns. `motion:` directives compile to a single `__sui_motion_attach(el, cfg)`
// call; the runtime creates the animations and decides whether to honor
// `prefers-reduced-motion`. No CSS kill switch owns this path — the CSS rule
// covers transitions the runtime doesn't (button hover lift, range thumb,
// table row hover), and stays independently in `styles/index.ts`.
//
// The runtime is a thin wrapper over motion.dev's vanilla `motion/mini`
// (WAAPI-only, ~3 kB) + `motion-dom` (the spring + stagger generators the
// mini wrapper does not export). The compiler does no AOT keyframe bake:
// it validates a closed preset catalogue and emits the resolved config
// (property, preset, kind, the preset's physics, the duration token, etc.),
// and the runtime maps each config to an `animate()` call. Three reasons:
//   1. inert-document `el.animate()` (the compiler's earlier shape) binds
//      the animation to a timeline that never advances, and a manual
//      `rehome` would have to be re-derived for every call site.
//   2. motion.dev owns one matchMedia subscription and one reduced-motion
//      decision per element — a CSS `* { animation: none !important }`
//      kill switch cannot reach WAAPI animations, so the previous runtime
//      already consulted matchMedia per event.
//   3. The catalog (a closed `animation-presets.json` the compiler and lint
//      both read) is the only surface that grows. AOT keyframes baked the
//      shape into JS, locking every preset's parameters to the compile.
//
// Animation lifecycle: the runtime creates an `AnimationPlaybackControls`
// per motion, parks them on the element, and the disposer cancels them.
// Reduced motion: when the matchMedia reports `reduce`, the runtime
// computes the FINAL keyframe value and writes it as inline style — same
// behavior motion.dev's `reduceMotion: true` would produce, but the
// option is inert on the vanilla `animate(element, ...)` path, so we
// short-circuit at the wrapper boundary.
//
// Springs: motion/mini's `animate` is a thin WAAPI wrapper. It does
// NOT accept a `type: 'spring'` STRING (motion's own NativeAnimation
// rejects it: `invariant(typeof options.type !== "string", ...)`). The
// supported path is to sample the spring's curve into a
// `linear(...)` easing string and hand THAT to motion/mini. The
// runtime builds the spring with motion-dom's `motionSpring({...})`,
// computes the rest duration with `calcGeneratorDuration`, and maps
// the normalized 0..1 progress to a WAAPI-compatible `linear(...)`
// string via `generateLinearEasing`. The compositor runs that as a
// native WAAPI animation — hardware-accelerated, no JS frame loop.
// This is the one place in the runtime where a spring is "really" a
// spring: a sampled curve the browser drives.

import { setTransitionRunner } from './transition-runner'
import type { Disposer } from './bind'
import { animate } from 'motion/mini'
import {
  spring as motionSpring,
  stagger as motionStagger,
  generateLinearEasing,
  calcGeneratorDuration,
} from 'motion-dom'

/** Per-property motion config the compiler emits. */
export interface MotionConfig {
  property: 'scale' | 'opacity' | 'translate-y' | 'translate-x' | 'rotate'
  preset: string
  kind: 'spring' | 'tween'
  stiffness?: number
  damping?: number
  mass?: number
  restSpeed?: number
  restDelta?: number
  from?: number
  to?: number
  /** Duration in seconds (Motion's unit). AOT-resolved from a token. */
  duration?: number
  easing?: string
  iterations?: number | 'infinity'
  direction?: 'normal' | 'reverse' | 'alternate' | 'mirror'
}

/** Per-call attach config the compiler emits. */
export interface MotionAttachConfig {
  trigger: string
  motions: MotionConfig[]
  /** Stagger step in seconds (Motion's unit). `0.15` = 150ms. */
  stagger?: number
}

let reducedMedia: MediaQueryList | null = null
function prefersReducedMotion(): boolean {
  if (typeof matchMedia === 'undefined') return false
  if (!reducedMedia) {
    const mql = matchMedia('(prefers-reduced-motion: reduce)')
    reducedMedia = mql
    if (typeof mql.addEventListener === 'function') {
      mql.addEventListener('change', () => { /* live flag re-read per call */ })
    }
  }
  return reducedMedia.matches
}

const REST_VALUE: Record<MotionConfig['property'], number> = {
  opacity: 0,
  scale: 0.96,
  'translate-y': 16,
  'translate-x': 16,
  rotate: -6,
}

const FINAL_VALUE: Record<MotionConfig['property'], number> = {
  opacity: 1,
  scale: 1,
  'translate-y': 0,
  'translate-x': 0,
  rotate: 0,
}

function isHTMLElement(el: Element): el is HTMLElement {
  return 'style' in el
}

function applyRestStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  const value = REST_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity': el.style.opacity = String(value); return
    case 'scale': el.style.scale = String(value); return
    case 'translate-y': el.style.translate = `0 ${value}px`; return
    case 'translate-x': el.style.translate = `${value}px 0`; return
    case 'rotate': el.style.rotate = `${value}deg`; return
  }
}

function applyFinalStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  const value = FINAL_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity': el.style.opacity = String(value); return
    case 'scale': el.style.scale = String(value); return
    case 'translate-y': el.style.translate = '0 0'; return
    case 'translate-x': el.style.translate = '0 0'; return
    case 'rotate': el.style.rotate = '0deg'; return
  }
}

function buildKeyframes(cfg: MotionConfig, el: HTMLElement): Record<string, number[]> {
  if (cfg.preset === 'bounce') {
    if (cfg.property === 'translate-y') {
      const h = el.getBoundingClientRect().height || el.offsetHeight || 8
      return { y: [0, -h] }
    }
    if (cfg.property === 'translate-x') {
      const w = el.getBoundingClientRect().width || el.offsetWidth || 8
      return { x: [0, -w] }
    }
  }
  const from = cfg.from ?? REST_VALUE[cfg.property]
  const to = cfg.to ?? FINAL_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity': return { opacity: [from, to] }
    case 'scale': return { scale: [from, to] }
    case 'translate-y': return { y: [from, to] }
    case 'translate-x': return { x: [from, to] }
    case 'rotate': return { rotate: [from, to] }
  }
}

function buildTransition(
  cfg: MotionConfig,
  index: number,
  total: number,
  staggerSec: number | undefined,
): Record<string, unknown> {
  const t: Record<string, unknown> = {}
  if (cfg.kind === 'spring') {
    // The spring's physics travel ON the generator, not on the transition
    // options. motion/mini's NativeAnimation rejects a string `type` and
    // does not consume a function `type` from mini's `animate()` path.
    // The supported contract: build the spring, compute its rest
    // duration, sample its curve into a 0..1 function, and hand that
    // to `generateLinearEasing` for a WAAPI-compatible `linear(...)`
    // string. The compositor runs that as a native WAAPI animation.
    const springArgs: Record<string, unknown> = { keyframes: [0, 1] }
    if (cfg.stiffness != null) springArgs.stiffness = cfg.stiffness
    if (cfg.damping != null) springArgs.damping = cfg.damping
    if (cfg.mass != null) springArgs.mass = cfg.mass
    if (cfg.restSpeed != null) springArgs.restSpeed = cfg.restSpeed
    if (cfg.restDelta != null) springArgs.restDelta = cfg.restDelta
    const generator = motionSpring(springArgs as Parameters<typeof motionSpring>[0])
    const durationMs = calcGeneratorDuration(generator)
    t.duration = durationMs / 1000
    t.ease = generateLinearEasing(
      (s: number) => generator.next(s * durationMs).value,
      durationMs,
    )
  } else {
    if (cfg.duration != null) t.duration = cfg.duration
    if (cfg.easing) t.ease = cfg.easing
  }
  if (cfg.iterations != null) t.repeat = cfg.iterations
  if (cfg.direction != null) {
    // Motion's `repeatType` vocabulary is not CSS `animation-direction`:
    // 'normal' → 'loop' (restart each iteration), 'alternate' → 'reverse'
    // (ping-pong). 'reverse'/'mirror' already match Motion's names. Passing
    // the raw CSS word made Motion fall back to its default and never
    // alternate — the bounce read as a one-way restart.
    t.repeatType =
      cfg.direction === 'normal'
        ? 'loop'
        : cfg.direction === 'alternate'
          ? 'reverse'
          : cfg.direction
  }
  if (staggerSec != null && total > 1) {
    t.delay = motionStagger(staggerSec)(index, total)
  }
  return t
}

function createAnimations(
  el: HTMLElement,
  config: MotionAttachConfig,
): Array<{ stop: () => void; cancel: () => void }> {
  const controls: Array<{ stop: () => void; cancel: () => void }> = []
  const stagger = config.stagger
  const total = config.motions.length
  for (let i = 0; i < total; i++) {
    const cfg = config.motions[i]!
    const keyframes = buildKeyframes(cfg, el)
    const transition = buildTransition(cfg, i, total, stagger)
    try {
      const c = animate(el, keyframes, transition as Parameters<typeof animate>[2])
      controls.push(c)
    } catch {
      applyFinalStyle(el, cfg)
    }
  }
  return controls
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
 *  The compiler wires `motion:` during detached subtree construction
 *  (`el = __tpl(...); __sui_motion_attach(el, ...)`), so on first mount
 *  the element is not yet in the document. We pin it to its rest pose
 *  immediately (so it does not flash its final value for a frame) and
 *  create the real animations on the frame after it joins the document.
 *  Elements already connected at attach time animate synchronously,
 *  exactly as before. Environments without `requestAnimationFrame`
 *  (SSR, tests) fall through to the synchronous path. */
function createAnimationsWhenConnected(
  el: HTMLElement,
  config: MotionAttachConfig,
): Disposer {
  if (el.isConnected || typeof requestAnimationFrame === 'undefined') {
    const controls = createAnimations(el, config)
    return () => { for (const c of controls) c.cancel() }
  }
  for (const m of config.motions) applyRestStyle(el, m)
  let controls: Array<{ stop: () => void; cancel: () => void }> = []
  let disposed = false
  const start = (): void => {
    if (disposed) return
    if (!el.isConnected) { requestAnimationFrame(start); return }
    controls = createAnimations(el, config)
  }
  requestAnimationFrame(start)
  return () => {
    disposed = true
    for (const c of controls) c.cancel()
  }
}

const TRIGGER_EVENTS: Record<string, [string, string, string?]> = {
  hover: ['mouseenter', 'mouseleave'],
  press: ['pointerdown', 'pointerup', 'pointercancel'],
  focus: ['focus', 'blur'],
  'focus-visible': ['focusin', 'focusout'],
}

let focusSource: 'keyboard' | 'mouse' | 'unknown' = 'unknown'
let focusTrackerInstalled = false
function trackFocusSource(): void {
  if (focusTrackerInstalled || typeof window === 'undefined') return
  focusTrackerInstalled = true
  window.addEventListener('keydown', () => { focusSource = 'keyboard' }, { capture: true })
  window.addEventListener('mousedown', () => { focusSource = 'mouse' }, { capture: true })
}

function isFocusVisible(ev: FocusEvent): boolean {
  const target = ev.target as Element | null
  if (target && typeof target.matches === 'function') {
    try { return target.matches(':focus-visible') } catch { /* not supported */ }
  }
  return focusSource === 'keyboard'
}

type Branch = 'loop' | 'enter' | 'exit' | 'view' | 'scroll' | 'event' | 'noop'

function classifyTrigger(trigger: string): Branch {
  if (trigger === 'loop') return 'loop'
  if (trigger === 'enter' || trigger === 'exit') return trigger
  if (trigger === 'view' || trigger === 'scroll') return trigger
  if (TRIGGER_EVENTS[trigger]) return 'event'
  return 'noop'
}

export function __sui_motion_attach(
  el: Element,
  config: MotionAttachConfig,
): Disposer {
  if (!isHTMLElement(el)) return () => {}
  const reduced = prefersReducedMotion()
  const branch = classifyTrigger(config.trigger)
  if (branch === 'noop') return () => {}

  if (branch === 'loop') {
    if (reduced) {
      for (const m of config.motions) applyFinalStyle(el, m)
      return () => {}
    }
    // `loop` means "play forever". A physics preset (`spring:bouncy`) is
    // reused for one-shot triggers too, so it carries no iteration count
    // of its own — the trigger supplies the repeat. Infinite + alternate
    // so the element travels back and forth (a bounce) rather than
    // snapping back to the start on every restart.
    const looped: MotionAttachConfig = {
      ...config,
      motions: config.motions.map((m) => ({
        ...m,
        iterations: m.iterations ?? Infinity,
        direction: m.direction ?? 'alternate',
      })),
    }
    return createAnimationsWhenConnected(el, looped)
  }

  if (branch === 'enter' || branch === 'exit') {
    installLifecycleRunner()
    if (branch === 'enter') {
      if (reduced) {
        for (const m of config.motions) applyFinalStyle(el, m)
        return () => {}
      }
      return createAnimationsWhenConnected(el, config)
    }
    if (reduced) return () => {}
    const controls = createAnimations(el, config)
    for (const m of config.motions) applyRestStyle(el, m)
    EXITS.set(el, { controls, motions: config.motions })
    return () => {
      EXITS.delete(el)
      for (const c of controls) c.cancel()
    }
  }

  if (branch === 'view' || branch === 'scroll') {
    if (reduced) {
      for (const m of config.motions) applyFinalStyle(el, m)
      return () => {}
    }
    return createAnimationsWhenConnected(el, config)
  }

  const events = TRIGGER_EVENTS[config.trigger]!
  if (config.trigger === 'focus-visible') trackFocusSource()
  const [enterEvent, leaveEvent, cancelEvent] = events
  // No mount-time priming. Creating (and immediately cancelling) the
  // interaction's animations here interrupts any OTHER motion that shares a
  // property on this element: `animate()` takes over the shared property, and
  // the cancel then drops it back to the element's inline base — clobbering,
  // e.g., a `motion:enter`/`motion:exit` opacity reveal so the element sticks
  // at opacity:0. The animations are created fresh on the first event, so the
  // priming was pure side effect. Start empty; `playForward` fills `live`.
  let live: Array<{ stop: () => void; cancel: () => void }> = []
  const playForward = () => {
    if (prefersReducedMotion()) return
    for (const c of live) c.cancel()
    live = createAnimations(el, config)
  }
  const onEnter = (ev: Event) => {
    if (config.trigger === 'focus-visible' && !isFocusVisible(ev as FocusEvent)) return
    playForward()
  }
  const onLeave = () => { playForward() }
  el.addEventListener(enterEvent, onEnter as EventListener)
  el.addEventListener(leaveEvent, onLeave as EventListener)
  if (cancelEvent) el.addEventListener(cancelEvent, onLeave as EventListener)
  return () => {
    el.removeEventListener(enterEvent, onEnter as EventListener)
    el.removeEventListener(leaveEvent, onLeave as EventListener)
    if (cancelEvent) el.removeEventListener(cancelEvent, onLeave as EventListener)
    for (const c of live) { try { c.cancel() } catch { /* ignore */ } }
  }
}

interface ExitEntry {
  controls: Array<{ stop: () => void; cancel: () => void }>
  motions: MotionConfig[]
}

const EXITS = new WeakMap<Element, ExitEntry>()
let runnerInstalled = false

function installLifecycleRunner(): void {
  if (runnerInstalled) return
  runnerInstalled = true
  setTransitionRunner({
    runEnter: () => null,
    runLeave: (nodes: Iterable<Node>): Promise<void> | null => {
      const playing: Promise<unknown>[] = []
      for (const n of nodes) {
        if (!(n instanceof Element)) continue
        const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll('*')].filter((e) => EXITS.has(e))
        for (const t of targets) {
          const entry = EXITS.get(t)
          if (!entry) continue
          for (const m of entry.motions) applyFinalStyle(t, m)
          for (const c of entry.controls) c.cancel()
        }
      }
      if (playing.length === 0) return null
      if (prefersReducedMotion()) return null
      return Promise.all(playing).then(() => {})
    },
    cancelTransitions: (node: Node): void => {
      if (!(node instanceof Element)) return
      const entry = EXITS.get(node)
      if (entry) for (const c of entry.controls) c.cancel()
    },
  })
}

/** Test seam: exposes the resolved transition the runtime hands to
 *  motion's `animate()` so tests can assert on the easing / duration
 *  shape without playing the animation. NOT re-exported from
 *  `@ship/sui-runtime`. */
export function __buildTransitionForTest(
  cfg: MotionConfig,
  index: number,
  total: number,
  staggerSec: number | undefined,
): Record<string, unknown> {
  return buildTransition(cfg, index, total, staggerSec)
}

export type { Disposer }
