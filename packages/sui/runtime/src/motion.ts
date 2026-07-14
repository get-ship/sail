// Motion runtime — the single authority for every animation the framework
// owns. `motion:` directives compile to a single `__sui_motion_attach(el, cfg)`
// call; the runtime creates the animations and decides whether to honor
// `prefers-reduced-motion`. No CSS kill switch owns this path — the CSS rule
// covers transitions the runtime doesn't (button hover lift, range thumb,
// table row hover), and stays independently in `styles/index.ts`.
//
// The runtime is a thin wrapper over motion.dev's vanilla `motion` package.
// The compiler does no AOT keyframe bake: it validates a closed preset
// catalogue and emits the resolved config (property, preset, kind, the
// preset's physics, the duration token, etc.), and the runtime maps each
// config to an `animate()` call. Three reasons:
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

import { setTransitionRunner } from './transition-runner'
import type { Disposer } from './bind'
import { animate, stagger as motionStagger, initPrefersReducedMotion } from 'motion'

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

/** Reduced-motion is the runtime's single authority. The runtime
 *  consults `matchMedia('(prefers-reduced-motion: reduce)')` on EVERY
 *  attach and on every interactive trigger — per-call, not cached —
 *  so a mid-session preference flip is honored at the next event. A
 *  cached value would freeze the answer to the first install. */
function prefersReducedMotion(): boolean {
  if (typeof matchMedia === 'undefined') return false
  return matchMedia('(prefers-reduced-motion: reduce)').matches
}

// The Motion library's `initPrefersReducedMotion()` is also a single
// matchMedia subscription — we call it once at module load so the
// `delay: stagger(...)` factory inside Motion sees the same value.
// Idempotent: Motion's own listener guard (`hasReducedMotionListener`)
// collapses repeated calls.
try { initPrefersReducedMotion() } catch { /* SSR */ }

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

/** Apply a single property's authored rest pose to `el` (inline style). */
function applyRestStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  const value = REST_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity':
      el.style.opacity = String(value)
      return
    case 'scale':
      el.style.scale = String(value)
      return
    case 'translate-y':
      el.style.translate = `0 ${value}px`
      return
    case 'translate-x':
      el.style.translate = `${value}px 0`
      return
    case 'rotate':
      el.style.rotate = `${value}deg`
      return
  }
}

function applyFinalStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  const value = FINAL_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity':
      el.style.opacity = String(value)
      return
    case 'scale':
      el.style.scale = String(value)
      return
    case 'translate-y':
      el.style.translate = '0 0'
      return
    case 'translate-x':
      el.style.translate = '0 0'
      return
    case 'rotate':
      el.style.rotate = '0deg'
      return
  }
}

/** Build the actual motion.dev keyframes for a property. The `bounce`
 *  preset on `translate-y`/`translate-x` expands the 0..1 contract into
 *  a pixel value measured from the element's own box — `-elHeight` for
 *  `translate-y`, `-elWidth` for `translate-x`. Every other preset uses
 *  the literal from/to the compiler resolved. */
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
    case 'opacity':
      return { opacity: [from, to] }
    case 'scale':
      return { scale: [from, to] }
    case 'translate-y':
      return { y: [from, to] }
    case 'translate-x':
      return { x: [from, to] }
    case 'rotate':
      return { rotate: [from, to] }
  }
}

/** Build the `Transition` shape Motion expects from a `MotionConfig`. */
function buildTransition(
  cfg: MotionConfig,
  index: number,
  total: number,
  staggerSec: number | undefined,
): Record<string, unknown> {
  const t: Record<string, unknown> = {}
  if (cfg.kind === 'spring') {
    t.type = 'spring'
    if (cfg.stiffness != null) t.stiffness = cfg.stiffness
    if (cfg.damping != null) t.damping = cfg.damping
    if (cfg.mass != null) t.mass = cfg.mass
    if (cfg.restSpeed != null) t.restSpeed = cfg.restSpeed
    if (cfg.restDelta != null) t.restDelta = cfg.restDelta
  } else {
    t.type = 'tween'
    if (cfg.duration != null) t.duration = cfg.duration
    if (cfg.easing) t.ease = cfg.easing
  }
  if (cfg.iterations != null) t.repeat = cfg.iterations
  if (cfg.direction != null) t.repeatType = cfg.direction
  if (staggerSec != null && total > 1) {
    // Per-sibling offset: `motion:loop` reads the wave as it goes out,
    // so the first motion in the array is the "earlier" sibling.
    t.delay = motionStagger(staggerSec)(index, total)
  }
  return t
}

/** Build an element-level `animate()` call from the configs. Returns the
 *  list of controls so the disposer can cancel them. */
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
      // Motion can throw if the element rejects animation (e.g. detached
      // node, very old browser). Fall back to the final pose.
      applyFinalStyle(el, cfg)
    }
  }
  return controls
}

/** DOM events that drive an interaction trigger (enter, leave). */
const TRIGGER_EVENTS: Record<string, [string, string, string?]> = {
  hover: ['mouseenter', 'mouseleave'],
  press: ['pointerdown', 'pointerup', 'pointercancel'],
  focus: ['focus', 'blur'],
  'focus-visible': ['focusin', 'focusout'],
}

/** Focus-source tracker for `:focus-visible` (see also: framework action). */
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

/** Public: called by the compiler's emitted JS for every `motion:` directive. */
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
    const controls = createAnimations(el, config)
    return () => { for (const c of controls) c.cancel() }
  }

  if (branch === 'enter' || branch === 'exit') {
    installLifecycleRunner()
    if (branch === 'enter') {
      if (reduced) {
        for (const m of config.motions) applyFinalStyle(el, m)
        return () => {}
      }
      const controls = createAnimations(el, config)
      return () => { for (const c of controls) c.cancel() }
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
    const controls = createAnimations(el, config)
    return () => { for (const c of controls) c.cancel() }
  }

  // Interaction triggers (hover, press, focus, focus-visible).
  const events = TRIGGER_EVENTS[config.trigger]!
  if (config.trigger === 'focus-visible') trackFocusSource()
  const [enterEvent, leaveEvent, cancelEvent] = events
  // The element rests at its authored identity until the first event —
  // create the controls now and cancel them so motion.dev never applies
  // the departed pose.
  const initial = createAnimations(el, config)
  for (const c of initial) c.cancel()
  let live = initial
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


// ── LIFECYCLE ──────────────────────────────────────────────────────────────
// `if.ts` / `for.ts` call `runLeave(nodes)`: return a promise and they hold
// the nodes in the DOM until it settles; return null and they tear down
// synchronously. The capability is real (CSS cannot hold an element after
// it leaves the tree) and the seam survives the `motion:` rewrite — backed
// by the same runtime.

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

export type { Disposer }
