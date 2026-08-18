// Motion runtime — the single authority for every animation the framework
// owns. Style motion (`style.enter` / `.exit` / `.hover` / `.loop` / …)
// compiles to a single `__sail_motion_attach(el, cfg)` call; the runtime
// creates the animations and decides whether to honor
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
// Under `prefers-reduced-motion: reduce`, the runtime computes the FINAL
// keyframe value and writes it as inline style — same behavior motion.dev's
// `reduceMotion: true` would produce, but the option is inert on the vanilla
// `animate(element, ...)` path, so we short-circuit at the wrapper boundary.
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

import { setTransitionRunner } from '@ship/sail-runtime/internal'
import { effect, untrack } from '@ship/sail-runtime'
import type { Disposer } from '@ship/sail-runtime'
import { animate } from 'motion/mini'
import {
  spring as motionSpring,
  generateLinearEasing,
  calcGeneratorDuration,
} from './motion-dom-used'

/** Per-property motion config the compiler emits. */
export interface MotionConfig {
  property: 'scale' | 'scale-x' | 'scale-y' | 'opacity' | 'translate-y' | 'translate-x' | 'rotate'
  preset: string
  kind: 'spring' | 'tween'
  stiffness?: number
  damping?: number
  mass?: number
  restSpeed?: number
  restDelta?: number
  /** A magnitude. A NUMBER is the channel's native unit (px for translate,
   *  deg for rotate, unitless for scale/opacity). A STRING carries its own
   *  unit and is used verbatim — which is how `'-12%'` becomes expressible,
   *  something the preset table could never say (the retired `slip` preset's
   *  own doc recorded "% is not expressible" and approximated it in px). */
  from?: number | string
  to?: number | string
  /** Duration in seconds (Motion's unit). The compiler converts from the
   *  milliseconds the author writes. */
  duration?: number
  delay?: number
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
  // Single-axis scale defaults to a fully-collapsed track (0) so a bare
  // `scale-x`/`scale-y` reads as a reveal/fill that grows to identity (1) —
  // mirroring how `translate-x` rests offset and settles to 0.
  'scale-x': 0,
  'scale-y': 0,
  'translate-y': 16,
  'translate-x': 16,
  rotate: -6,
}

const FINAL_VALUE: Record<MotionConfig['property'], number> = {
  opacity: 1,
  scale: 1,
  'scale-x': 1,
  'scale-y': 1,
  'translate-y': 0,
  'translate-x': 0,
  rotate: 0,
}

/** Does this magnitude already carry its own unit? `'120px'` and `'-12%'`
 *  do; `'120'`, `120` and `''` do not. */
function hasUnit(v: number | string | undefined): boolean {
  // NOT a type predicate: `'120'` is a string WITHOUT a unit, so narrowing
  // the negative branch to `number | undefined` would be a lie.
  return typeof v === 'string' && v.trim() !== '' && Number.isNaN(Number(v))
}

/** Render a magnitude into a CSS length/angle. A unit-bearing string wins
 *  verbatim; anything numeric gets the channel's native unit appended. This
 *  is the ONE place a unit is decided, so `translate-x: { to: 120 }` and
 *  `translate-x: { to: '120px' }` cannot drift apart. */
function withUnit(v: number | string | undefined, unit: string, fallback: number): string {
  if (hasUnit(v)) return (v as string).trim()
  const n = v === undefined || v === '' ? fallback : Number(v)
  return `${Number.isNaN(n) ? fallback : n}${unit}`
}

/** Render a magnitude for a UNITLESS channel (scale, opacity). A string that
 *  smuggled in a unit here would produce an invalid keyframe, so it is
 *  coerced back to a bare number. */
function withoutUnit(v: number | string | undefined, fallback: number): number {
  if (v === undefined || v === '') return fallback
  const n = typeof v === 'number' ? v : Number.parseFloat(v)
  return Number.isNaN(n) ? fallback : n
}

function isHTMLElement(el: Element): el is HTMLElement {
  return 'style' in el
}

function applyRestStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  // Honor authored `from`. Pulse/press start at 1 — pinning them to the
  // fade rest (opacity 0) flashes a visible loop/button out of existence
  // for the detached-mount rAF, then the real keyframes pop it back.
  switch (cfg.property) {
    case 'opacity':
      el.style.opacity = String(withoutUnit(cfg.from, REST_VALUE.opacity))
      return
    case 'scale':
      el.style.scale = String(withoutUnit(cfg.from, REST_VALUE.scale))
      return
    // Single-axis: the OTHER axis is pinned at identity (1), so `scale-x`
    // stretches horizontally only and `scale-y` vertically only.
    case 'scale-x':
      el.style.scale = `${withoutUnit(cfg.from, REST_VALUE['scale-x'])} 1`
      return
    case 'scale-y':
      el.style.scale = `1 ${withoutUnit(cfg.from, REST_VALUE['scale-y'])}`
      return
    case 'translate-y':
      el.style.translate = `0 ${withUnit(cfg.from, 'px', REST_VALUE['translate-y'])}`
      return
    case 'translate-x':
      el.style.translate = `${withUnit(cfg.from, 'px', REST_VALUE['translate-x'])} 0`
      return
    case 'rotate':
      el.style.rotate = withUnit(cfg.from, 'deg', REST_VALUE.rotate)
      return
  }
}

function applyFinalStyle(el: Element, cfg: MotionConfig): void {
  if (!isHTMLElement(el)) return
  const value = FINAL_VALUE[cfg.property]
  switch (cfg.property) {
    case 'opacity': el.style.opacity = String(value); return
    case 'scale': el.style.scale = String(value); return
    case 'scale-x': el.style.scale = `${value} 1`; return
    case 'scale-y': el.style.scale = `1 ${value}`; return
    case 'translate-y': el.style.translate = '0 0'; return
    case 'translate-x': el.style.translate = '0 0'; return
    case 'rotate': el.style.rotate = '0deg'; return
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
function applyReducedMotionSettle(el: Element, motions: MotionConfig[]): void {
  if (!isHTMLElement(el)) return
  for (const m of motions) applyFinalStyle(el, m)
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
function axisRange(cfg: MotionConfig, el: HTMLElement): [string, string] {
  if (cfg.preset === 'bounce') {
    if (cfg.property === 'translate-y') {
      const h = el.getBoundingClientRect().height || el.offsetHeight || 8
      return ['0px', `${-h}px`]
    }
    if (cfg.property === 'translate-x') {
      const w = el.getBoundingClientRect().width || el.offsetWidth || 8
      return ['0px', `${-w}px`]
    }
  }
  return [
    withUnit(cfg.from, 'px', REST_VALUE[cfg.property]),
    withUnit(cfg.to, 'px', FINAL_VALUE[cfg.property]),
  ]
}

function buildKeyframes(cfg: MotionConfig, el: HTMLElement): Record<string, Array<number | string>> {
  if (cfg.property === 'translate-y' || cfg.property === 'translate-x') {
    const [from, to] = axisRange(cfg, el)
    // Single-axis case: the other component is pinned at 0 for both
    // keyframes — the same shape `applyRestStyle`/`applyFinalStyle`
    // already write to `el.style.translate`.
    return cfg.property === 'translate-y'
      ? { translate: [`0px ${from}`, `0px ${to}`] }
      : { translate: [`${from} 0px`, `${to} 0px`] }
  }
  const from = withoutUnit(cfg.from, REST_VALUE[cfg.property])
  const to = withoutUnit(cfg.to, FINAL_VALUE[cfg.property])
  switch (cfg.property) {
    case 'opacity': return { opacity: [from, to] }
    case 'scale': return { scale: [from, to] }
    // Single-axis scale: pin the untouched axis at identity (1) for both
    // keyframes, exactly as `translate-x`/`-y` pin their other component at 0.
    case 'scale-x': return { scale: [`${from} 1`, `${to} 1`] }
    case 'scale-y': return { scale: [`1 ${from}`, `1 ${to}`] }
    // CSS's native `rotate` property (unlike the `translate`/`scale`
    // shorthands) requires an angle, not a bare number — WAAPI rejects
    // `{ rotate: [-6, 0] }` with "Invalid keyframe value for property
    // rotate". Every other property in this switch is unitless by spec
    // (opacity, scale).
    case 'rotate': return {
      rotate: [
        withUnit(cfg.from, 'deg', REST_VALUE.rotate),
        withUnit(cfg.to, 'deg', FINAL_VALUE.rotate),
      ],
    }
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
function buildMergedTranslateKeyframes(
  xCfg: MotionConfig | undefined,
  yCfg: MotionConfig | undefined,
  el: HTMLElement,
): Record<string, Array<string>> {
  const [xFrom, xTo] = xCfg ? axisRange(xCfg, el) : ['0px', '0px']
  const [yFrom, yTo] = yCfg ? axisRange(yCfg, el) : ['0px', '0px']
  return { translate: [`${xFrom} ${yFrom}`, `${xTo} ${yTo}`] }
}

/** The full set of CSS properties this runtime's preset table can ever
 *  emit as a WAAPI keyframe key. `SILENCE IS THE BUG`: a keyframe
 *  property outside this set animates zero pixels/opacity/anything on
 *  an HTML element — WAAPI accepts (and ignores) unknown keyframe
 *  properties with no error, so a bug here would otherwise ship
 *  invisibly, exactly like the `x`/`y` defect this guards against. */
const ANIMATABLE_KEYFRAME_PROPERTIES = new Set(['opacity', 'scale', 'translate', 'rotate'])

function warnIfNotAnimatable(el: Element, keyframes: Record<string, Array<number | string>>): void {
  const supports = typeof CSS !== 'undefined' && typeof CSS.supports === 'function' ? CSS.supports : null
  for (const prop of Object.keys(keyframes)) {
    if (!ANIMATABLE_KEYFRAME_PROPERTIES.has(prop)) {
      console.warn(
        `[sail/motion] keyframe property "${prop}" is not one of this runtime's known-animatable ` +
        `properties (${[...ANIMATABLE_KEYFRAME_PROPERTIES].join(', ')}) — WAAPI accepts unknown ` +
        `keyframe properties silently, so this animation will run forever and move nothing.`,
        el,
      )
      continue
    }
    // Belt-and-suspenders: an allow-listed property with a malformed
    // value (e.g. a unitless `rotate` regression) is just as silent —
    // `CSS.supports` catches the value shape even when the property
    // name itself is fine.
    if (!supports) continue
    for (const value of keyframes[prop]!) {
      if (!supports(prop, String(value))) {
        console.warn(
          `[sail/motion] keyframe value ${JSON.stringify(value)} is not valid for CSS property ` +
          `"${prop}" — the animation will silently no-op.`,
          el,
        )
      }
    }
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
function warnIfOutOfVisibleRange(
  el: Element,
  cfg: MotionConfig,
  keyframes: Record<string, Array<number | string>>,
): void {
  if (cfg.property !== 'opacity') return
  for (const value of keyframes.opacity ?? []) {
    const n = typeof value === 'number' ? value : Number(value)
    if (Number.isFinite(n) && (n < 0 || n > 1)) {
      console.warn(
        `[sail/motion] opacity keyframe value ${JSON.stringify(value)} (from preset "${cfg.preset}") ` +
        `is outside the visible 0..1 range — the computed style clamps it, so this animation runs ` +
        `and moves nothing. "${cfg.preset}" looks like a preset authored for a different property ` +
        `(e.g. scale) — check the property this preset is attached to.`,
        el,
      )
      return
    }
  }
}

function buildTransition(
  cfg: MotionConfig,
  delaySec: number | undefined,
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
    const springArgs: Exclude<Parameters<typeof motionSpring>[0], number> = {
      keyframes: [0, 1],
    }
    if (cfg.stiffness != null) springArgs.stiffness = cfg.stiffness
    if (cfg.damping != null) springArgs.damping = cfg.damping
    if (cfg.mass != null) springArgs.mass = cfg.mass
    if (cfg.restSpeed != null) springArgs.restSpeed = cfg.restSpeed
    if (cfg.restDelta != null) springArgs.restDelta = cfg.restDelta
    const generator = motionSpring(springArgs)
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
  // Two delays, one field: the per-sibling stagger step (orchestration
  // BETWEEN elements) and the config's own `delay` (this animation's own
  // offset). They compose — a staggered list whose members each wait 200ms
  // should wait 200ms AND stagger.
  if (delaySec != null || cfg.delay != null) {
    t.delay = (delaySec ?? 0) + (cfg.delay ?? 0)
  }
  return t
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
const staggerIndices = new WeakMap<Element, Map<string, number>>()
function nextStaggerIndex(parent: Element, trigger: string): number {
  let byTrigger = staggerIndices.get(parent)
  if (!byTrigger) {
    byTrigger = new Map()
    staggerIndices.set(parent, byTrigger)
  }
  const index = byTrigger.get(trigger) ?? 0
  byTrigger.set(trigger, index + 1)
  return index
}

/** Resolved ONCE per `__sail_motion_attach` call (per element, per
 *  mount) — never recomputed on repeat triggers (hover in/out, replay)
 *  — so an element's sibling phase is fixed for its lifetime. Motion's
 *  own `stagger()` distance formula (`from: 0`, the default) is
 *  `duration * index` regardless of `total`, so no sibling count needs
 *  to be known in advance. */
function computeStaggerDelay(config: MotionAttachConfig, el: Element): number | undefined {
  if (config.stagger == null) return undefined
  const parent = el.parentElement
  if (!parent) return 0
  const index = nextStaggerIndex(parent, config.trigger)
  // motion-dom stagger(`duration`, { from: 0 }) is `duration * index`.
  return config.stagger * index
}

/** The subset of `AnimationPlaybackControls` this runtime relies on —
 *  `stop`/`cancel` to tear an animation down, `finished` to know when
 *  it settled on its own. The optional members are what the timeline
 *  triggers use: `attachTimeline` re-times the animation onto a
 *  `ViewTimeline`/`ScrollTimeline` (motion-dom's own seam for this),
 *  and `pause`/`play`/`complete` drive the no-timeline fallback. They
 *  are optional so a test double that only speaks `stop`/`cancel`
 *  still satisfies the type — the runtime degrades per-member. */
type AnimControls = {
  stop: () => void
  cancel: () => void
  finished: Promise<unknown>
  pause?: () => void
  play?: () => void
  complete?: () => void
  attachTimeline?: (timeline: {
    timeline?: AnimationTimeline | undefined
    rangeStart?: string
    rangeEnd?: string
    observe: (animation: unknown) => VoidFunction
  }) => VoidFunction
}

/** Resume a freshly-created set of animations at `timeMs` — used when a
 *  reactive config change re-creates an ambient loop and we do not want it
 *  to visibly snap back to the start. Best-effort by construction: a driver
 *  that does not expose a settable time simply plays from zero. */
function seekTo(el: HTMLElement, timeMs: number | null): void {
  if (timeMs == null) return
  try {
    for (const a of el.getAnimations?.() ?? []) {
      const total = (a.effect?.getTiming?.().duration as number | undefined) ?? 0
      a.currentTime = total > 0 ? timeMs % total : timeMs
    }
  } catch {
    // No getAnimations (jsdom) or a read-only time — play from the top.
  }
}

function createAnimations(
  el: HTMLElement,
  config: MotionAttachConfig,
  delaySec: number | undefined,
): AnimControls[] {
  const controls: AnimControls[] = []
  const total = config.motions.length
  const xIndex = config.motions.findIndex((m) => m.property === 'translate-x')
  const yIndex = config.motions.findIndex((m) => m.property === 'translate-y')
  const merged = new Set<number>()

  if (xIndex !== -1 && yIndex !== -1) {
    const xCfg = config.motions[xIndex]!
    const yCfg = config.motions[yIndex]!
    const keyframes = buildMergedTranslateKeyframes(xCfg, yCfg, el)
    warnIfNotAnimatable(el, keyframes)
    // The transition (duration/easing/iterations) is taken from
    // whichever axis is authored first — one WAAPI `translate` track
    // gets one transition, so mismatched x/y presets on the same
    // element resolve to the earlier motion's physics. Authoring the
    // same preset on both axes (the common case) makes this moot. The
    // sibling stagger delay is the same for every property on this
    // element regardless — it is not indexed by which axis leads.
    const leadIndex = Math.min(xIndex, yIndex)
    const transition = buildTransition(config.motions[leadIndex]!, delaySec)
    try {
      const c = animate(el, keyframes, transition as Parameters<typeof animate>[2])
      controls.push(c as unknown as AnimControls)
    } catch {
      applyFinalStyle(el, xCfg)
      applyFinalStyle(el, yCfg)
    }
    merged.add(xIndex)
    merged.add(yIndex)
  }

  for (let i = 0; i < total; i++) {
    if (merged.has(i)) continue
    const cfg = config.motions[i]!
    const keyframes = buildKeyframes(cfg, el)
    warnIfNotAnimatable(el, keyframes)
    warnIfOutOfVisibleRange(el, cfg, keyframes)
    const transition = buildTransition(cfg, delaySec)
    try {
      const c = animate(el, keyframes, transition as Parameters<typeof animate>[2])
      controls.push(c as unknown as AnimControls)
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
 *  The compiler wires Style motion during detached subtree construction
 *  (`el = __tpl(...); __sail_motion_attach(el, ...)`), so on first mount
 *  the element is not yet in the document. We pin it to its rest pose
 *  immediately (so it does not flash its final value for a frame) and
 *  create the real animations on the frame after it joins the document.
 *  Elements already connected at attach time animate synchronously,
 *  exactly as before. Environments without `requestAnimationFrame`
 *  (SSR, tests) fall through to the synchronous path. */
function createAnimationsWhenConnected(
  el: HTMLElement,
  config: MotionAttachConfig,
  delaySec: number | undefined,
  carriedTimeMs: number | null = null,
  afterCreate?: (controls: AnimControls[]) => Disposer,
): Disposer {
  if (el.isConnected || typeof requestAnimationFrame === 'undefined') {
    const controls = createAnimations(el, config, delaySec)
    seekTo(el, carriedTimeMs)
    const extra = afterCreate?.(controls)
    return () => {
      extra?.()
      for (const c of controls) c.cancel()
    }
  }
  for (const m of config.motions) applyRestStyle(el, m)
  let controls: AnimControls[] = []
  let extra: Disposer | undefined
  let disposed = false
  const start = (): void => {
    if (disposed) return
    if (!el.isConnected) { requestAnimationFrame(start); return }
    controls = createAnimations(el, config, delaySec)
    seekTo(el, carriedTimeMs)
    extra = afterCreate?.(controls)
  }
  requestAnimationFrame(start)
  return () => {
    disposed = true
    extra?.()
    for (const c of controls) c.cancel()
  }
}

// ── Timeline triggers — style.reveal / style.scroll ─────────────────
//
// PROGRESS-driven, not time-driven: the animation's position is a function
// of scroll, not of the clock. Where the platform has scroll timelines
// (Chromium), the SAME animation every other trigger bakes is simply
// re-timed onto a `ViewTimeline` (view: the element's own passage into the
// viewport) or a `ScrollTimeline` (scroll: the document's total scroll) —
// zero JS per frame, compositor-driven. Where it does not (Firefox, jsdom),
// `reveal` degrades from scrub-to-scroll to play-once-on-first-reveal via
// IntersectionObserver: a static end state is not a fallback for "animate
// as it enters the viewport", it reads as the feature being broken.
//
// HISTORY, because this has now been broken twice: the pre-motion.dev
// runtime carried exactly this wiring (`a.timeline = new ViewTimeline(...)`
// + the IO fallback). The motion.dev rewrite (8bb03de9) rewrote the
// branch as a bare `createAnimations()` — identical to `enter` — so every
// `style.reveal` element played its reveal ONCE at mount, below the fold,
// invisibly, and sat at its final pose forever. No error surfaces because
// a finished animation is indistinguishable from a correct one. If you are
// editing this branch: the contract is "hidden until the viewport says
// otherwise", and the degraded path below must stay LOUD.

interface ProgressTimelineCtors {
  ViewTimeline?: new (opts: { subject: Element }) => AnimationTimeline
  ScrollTimeline?: new (opts: { source: Element | null }) => AnimationTimeline
}

function makeProgressTimeline(kind: 'reveal' | 'scroll', el: Element): AnimationTimeline | undefined {
  const g = globalThis as ProgressTimelineCtors
  try {
    if (kind === 'reveal') {
      return g.ViewTimeline ? new g.ViewTimeline({ subject: el }) : undefined
    }
    if (typeof document === 'undefined') return undefined
    return g.ScrollTimeline ? new g.ScrollTimeline({ source: document.scrollingElement }) : undefined
  } catch {
    return undefined
  }
}

/** No scroll timeline available. `reveal` still has a correct meaning —
 *  play the reveal once, time-driven, the first time the element
 *  intersects the viewport. `scroll` (progress = container scroll) has
 *  no time-driven equivalent, so it lands on the END state — visible,
 *  not invisible — and SAYS SO: this runtime's history is silent
 *  degradations shipping as "works on my machine", so the degraded
 *  path is loud by policy. */
function observeProgressFallback(el: Element, kind: 'reveal' | 'scroll', c: AnimControls): Disposer {
  if (kind === 'reveal' && typeof IntersectionObserver === 'function') {
    c.pause?.()
    const io = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) {
          io.disconnect()
          c.play?.()
        }
      },
      { threshold: 0.15 },
    )
    io.observe(el)
    return () => io.disconnect()
  }
  console.warn(
    `[sail/motion] style.${kind} needs ${kind === 'reveal' ? 'ViewTimeline or IntersectionObserver' : 'ScrollTimeline'}, ` +
    `and this environment has neither — settling at the final pose instead of animating. ` +
    `The trigger is degraded, not silently dropped.`,
    el,
  )
  c.complete?.()
  return () => {}
}

/** Re-time a freshly created animation set onto its progress timeline.
 *  Runs synchronously after `createAnimations` (via the `afterCreate`
 *  hook), before the just-started time-driven animation can advance or
 *  finish. Returns the disposer for whatever the fallback installed.
 *
 *  REVEAL vs VIEWTIMELINE — measured failure (2026-08-09, Chromium):
 *  App shells nest the document in `overflow-y: auto` (docs rail, main
 *  chrome). ViewTimeline + `entry 0%…100%` (and `cover`) then report
 *  *negative* progress for the entire time the subject is on screen, so
 *  fill:both pins opacity/translate at the FROM keyframe (invisible)
 *  while the user is looking at it. Max opacity while `inView` with
 *  entry ranges: **0**. Progress only advances after the element has
 *  scrolled *past* the viewport — the opposite of "reveal on enter".
 *
 *  `style.reveal`'s author contract is "play when it enters the
 *  viewport", not scrub-linked scroll progress. IntersectionObserver
 *  play-once is the correct primary path for that contract in nested
 *  scrollports. ViewTimeline scrub stays available for `scroll` (page
 *  progress), which is not entry-gated the same way.
 */
function attachProgressTimeline(el: Element, kind: 'reveal' | 'scroll', controls: AnimControls[]): Disposer {
  if (kind === 'reveal') {
    const detachers: Disposer[] = []
    for (const c of controls) {
      detachers.push(observeProgressFallback(el, 'reveal', c))
    }
    return () => { for (const d of detachers) d() }
  }

  const timeline = makeProgressTimeline(kind, el)
  const detachers: Disposer[] = []
  for (const c of controls) {
    if (typeof c.attachTimeline === 'function') {
      detachers.push(
        c.attachTimeline({
          timeline,
          observe: () => observeProgressFallback(el, kind, c),
        }),
      )
    } else {
      detachers.push(observeProgressFallback(el, kind, c))
    }
  }
  return () => { for (const d of detachers) d() }
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

type Branch = 'loop' | 'enter' | 'exit' | 'reveal' | 'scroll' | 'event' | 'noop'

function classifyTrigger(trigger: string): Branch {
  if (trigger === 'loop') return 'loop'
  if (trigger === 'enter' || trigger === 'exit') return trigger
  if (trigger === 'reveal' || trigger === 'scroll') return trigger
  if (TRIGGER_EVENTS[trigger]) return 'event'
  return 'noop'
}

/** Attach Style motion (`style({ animate: { <trigger>: play(…) } })`).
 *
 *  The compiler emits a PLAIN OBJECT when every field of the config is a
 *  literal, and a THUNK when any field holds a signal. The thunk form is
 *  the reactive lane:
 *
 *      style({ animate: { loop: play(fade) } })
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
export function __sail_motion_attach(
  el: Element,
  config: MotionAttachConfig | (() => MotionAttachConfig),
): Disposer {
  if (typeof config !== 'function') return attachOnce(el, config)
  let inner: Disposer | null = null
  let carried: number | null = null
  const stop = effect(() => {
    // Reading the config inside the effect is what subscribes to every
    // signal any of its fields touch.
    const next = config()
    untrack(() => {
      if (inner) {
        carried = readPlaybackTime(el, next)
        inner()
        inner = null
      }
      inner = attachOnce(el, next, carried)
    })
  })
  return () => {
    stop()
    if (inner) inner()
    inner = null
  }
}

/** The playback position to carry across a re-creation, or `null` to start
 *  from zero. Only infinite loops carry — a finite animation whose target
 *  just changed should play the new one from the top. */
function readPlaybackTime(el: HTMLElement | Element, cfg: MotionAttachConfig): number | null {
  const infinite = cfg.motions.some((m) => m.iterations === Infinity) || cfg.trigger === 'loop'
  if (!infinite) return null
  try {
    const anims = (el as HTMLElement).getAnimations?.() ?? []
    for (const a of anims) {
      const t = a.currentTime
      if (typeof t === 'number' && Number.isFinite(t)) return t
    }
  } catch {
    // Environments without getAnimations (jsdom) simply restart.
  }
  return null
}

function attachOnce(
  el: Element,
  config: MotionAttachConfig,
  carriedTimeMs: number | null = null,
): Disposer {
  if (!isHTMLElement(el)) return () => {}
  const reduced = prefersReducedMotion()
  const branch = classifyTrigger(config.trigger)
  if (branch === 'noop') return () => {}
  // Resolved ONCE per attach (per element, per mount) — the sibling
  // phase index this consumes must not be recomputed on repeat
  // triggers (hover in/out) or it would keep advancing forever.
  const delaySec = computeStaggerDelay(config, el)

  if (branch === 'loop') {
    if (reduced) {
      applyReducedMotionSettle(el, config.motions)
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
    return createAnimationsWhenConnected(el, looped, delaySec, carriedTimeMs)
  }

  if (branch === 'enter' || branch === 'exit') {
    installLifecycleRunner()
    if (branch === 'enter') {
      if (reduced) {
        applyReducedMotionSettle(el, config.motions)
        return () => {}
      }
      return createAnimationsWhenConnected(el, config, delaySec)
    }
    // Exit motions do NOT run at mount — they are parked here and only
    // played by the lifecycle runner's `runLeave`, when the owning
    // {#if}/{#match} branch is actually torn down. (Previously this called
    // `createAnimations` eagerly, which played the "exit" animation the
    // moment the element mounted, and then `runLeave` just cancelled it and
    // snapped to the final pose — so real removal never held the node for
    // an animation, and the exit contract was silently a no-op.)
    if (reduced) return () => {}
    EXITS.set(el, { config, controls: [], delaySec })
    return () => {
      const entry = EXITS.get(el)
      EXITS.delete(el)
      if (entry) for (const c of entry.controls) c.cancel()
    }
  }

  if (branch === 'reveal' || branch === 'scroll') {
    if (reduced) {
      applyReducedMotionSettle(el, config.motions)
      return () => {}
    }
    // The same baked animation as every other trigger, re-timed onto a
    // progress timeline the moment it exists — never left time-driven,
    // or a below-the-fold reveal plays invisibly at mount and the
    // trigger is silently `enter`.
    return createAnimationsWhenConnected(el, config, delaySec, null, (controls) =>
      attachProgressTimeline(el, branch, controls),
    )
  }

  const events = TRIGGER_EVENTS[config.trigger]!
  if (config.trigger === 'focus-visible') trackFocusSource()
  const [enterEvent, leaveEvent, cancelEvent] = events
  // No mount-time priming. Creating (and immediately cancelling) the
  // interaction's animations here interrupts any OTHER motion that shares a
  // property on this element: `animate()` takes over the shared property, and
  // the cancel then drops it back to the element's inline base — clobbering,
  // e.g., a style.enter/style.exit opacity reveal so the element sticks
  // at opacity:0. The animations are created fresh on the first event, so the
  // priming was pure side effect. Start empty; `playForward` fills `live`.
  let live: Array<{ stop: () => void; cancel: () => void }> = []
  const playForward = () => {
    if (prefersReducedMotion()) return
    for (const c of live) c.cancel()
    live = createAnimations(el, config, delaySec)
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
  /** Parked at attach time; motions are only turned into real
   *  animations when `runLeave` actually fires. */
  config: MotionAttachConfig
  /** The in-flight exit animation's controls, if `runLeave` is
   *  currently playing one. Empty otherwise. */
  controls: AnimControls[]
  /** The sibling stagger delay resolved at attach time — exit shares
   *  the same fixed-per-element phase index as enter/loop. */
  delaySec: number | undefined
}

const EXITS = new WeakMap<Element, ExitEntry>()
let runnerInstalled = false

/** Upper bound on how long `runLeave` will hold a node in the DOM for its
 *  exit animation. A broken/never-settling animation (a bad easing curve,
 *  a preset with an unreachable rest state, a test double that never
 *  resolves `finished`) must not wedge removal forever — after this many
 *  ms the animation is cancelled, the final pose is snapped on, and
 *  removal proceeds regardless of playback state. 4000ms is generous
 *  headroom over the slowest shipped preset (`spring:deliberate`, well
 *  under 1s to rest) while still being a bounded, user-perceptible cap
 *  rather than an unbounded wait. */
const EXIT_TIMEOUT_MS = 4000

function installLifecycleRunner(): void {
  if (runnerInstalled) return
  runnerInstalled = true
  setTransitionRunner({
    runEnter: () => null,
    runLeave: (nodes: Iterable<Node>): Promise<void> | null => {
      const matched: Array<{ el: Element; entry: ExitEntry }> = []
      for (const n of nodes) {
        if (!(n instanceof Element)) continue
        const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll('*')].filter((e) => EXITS.has(e))
        for (const t of targets) {
          const entry = EXITS.get(t)
          if (entry) matched.push({ el: t, entry })
        }
      }
      if (matched.length === 0) return null
      if (prefersReducedMotion()) {
        for (const { el, entry } of matched) {
          for (const m of entry.config.motions) applyFinalStyle(el, m)
        }
        return null
      }
      const playing: Promise<unknown>[] = []
      for (const { el, entry } of matched) {
        const controls = createAnimations(el as HTMLElement, entry.config, entry.delaySec)
        entry.controls = controls
        for (const c of controls) playing.push(c.finished.catch(() => {}))
      }
      let timeoutId: ReturnType<typeof setTimeout>
      const settled = Promise.race([
        Promise.all(playing).then(() => {}),
        new Promise<void>((resolve) => { timeoutId = setTimeout(resolve, EXIT_TIMEOUT_MS) }),
      ])
      return settled.then(() => {
        clearTimeout(timeoutId)
        for (const { el, entry } of matched) {
          // Cancel unconditionally: a no-op on an already-finished
          // animation, and the guard that stops a timed-out one.
          for (const c of entry.controls) c.cancel()
          for (const m of entry.config.motions) applyFinalStyle(el, m)
          entry.controls = []
        }
      })
    },
    cancelTransitions: (node: Node): void => {
      if (!(node instanceof Element)) return
      const entry = EXITS.get(node)
      if (entry) {
        for (const c of entry.controls) c.cancel()
        entry.controls = []
      }
    },
  })
}

/** Test seam: exposes the resolved transition the runtime hands to
 *  motion's `animate()` so tests can assert on the easing / duration
 *  shape without playing the animation. NOT re-exported from
 *  `@ship/sail-runtime`. */
export function __buildTransitionForTest(
  cfg: MotionConfig,
  delaySec: number | undefined,
): Record<string, unknown> {
  return buildTransition(cfg, delaySec)
}

/** Test seam: exposes the resolved keyframes the runtime hands to
 *  motion's `animate()` so tests can assert every property lands as
 *  WAAPI-valid CSS (units where the spec requires them) without
 *  playing the animation. NOT re-exported from `@ship/sail-runtime`. */
export function __buildKeyframesForTest(
  cfg: MotionConfig,
  el: HTMLElement,
): Record<string, Array<number | string>> {
  return buildKeyframes(cfg, el)
}

/** Test seam: exposes the SILENCE-IS-THE-BUG guard directly so tests can
 *  assert its warning contract (names the property + the element)
 *  without having to reconstruct the historical `x`/`y` defect through
 *  a broken fixture. NOT re-exported from `@ship/sail-runtime`. */
export function __warnIfNotAnimatableForTest(
  el: Element,
  keyframes: Record<string, Array<number | string>>,
): void {
  warnIfNotAnimatable(el, keyframes)
}

/** Test seam: the bound `runLeave` uses to guarantee removal is never
 *  wedged by a stuck exit animation. NOT re-exported from
 *  `@ship/sail-runtime`. */
export const __EXIT_TIMEOUT_MS_FOR_TEST = EXIT_TIMEOUT_MS

export type { Disposer }
