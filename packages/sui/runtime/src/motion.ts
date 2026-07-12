// Motion runtime helpers — the consumer side of the compiler's
// AOT-baked `motion:` directive. The compiler emits
// `el.animate(keyframes, options)` for every `MotionProp`, then a
// `__sui_motion_attach(el, { trigger, enter, leave })` call that
// wires the resulting `Animation` objects to a DOM event or signal.
//
// Triggers supported (milestone 1):
//   - `hover`         — mouseenter / mouseleave
//   - `press`         — pointerdown / pointerup / pointercancel
//   - `focus`         — focus / blur
//   - `focus-visible` — focusin / focusout (with :focus-visible
//                       polyfill: skip if not keyboard-triggered)
//   - any other       — no-op; the call is recorded as a warning
//                       source for the future signal-triggers milestone.
//
// The `enter` and `leave` arrays are AOT-baked `Animation` objects
// — the per-property `el.animate(...)` results. Attach re-homes them
// onto the live document timeline (they are baked on template clones,
// whose node document is the <template>'s inert one — see `rehome`),
// cancels them so the element rests in its authored pose, then wires
// them to the trigger. The full per-trigger contract is the table in
// `__sui_motion_attach`.
import { setTransitionRunner } from './transition-runner'
import type { Disposer } from './bind'

/** AOT-baked WAAPI animation. */
export type BakedAnimation = Animation

/** Per-call config the compiler emits. The arrays may carry `null` holes:
 * the emitted `el.animate(...)` calls are guarded (`typeof el.animate ===
 * 'function' ? ... : null`) so environments without WAAPI — jsdom, SSR —
 * mount cleanly instead of crashing component setup. */
export interface MotionAttachConfig {
  trigger: string
  /** `stagger: '40ms'` — the delay STEP between siblings arriving together. */
  stagger?: number
  enter: Array<BakedAnimation | null>
  leave: Array<BakedAnimation | null>
}

/** WAAPI animations are programmatic — the runtime's CSS kill switch
 * (`* { animation: none !important }`) cannot stop them, so `motion:`
 * triggers consult the media query directly, per event (the preference
 * can change mid-session). */
function prefersReducedMotion(): boolean {
  return (
    typeof matchMedia === 'function' &&
    matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}

/** DOM events that drive a `motion:` trigger. `(enter, leave)`. */
const TRIGGER_EVENTS: Record<string, [string, string, string?]> = {
  hover: ['mouseenter', 'mouseleave'],
  press: ['pointerdown', 'pointerup', 'pointercancel'],
  focus: ['focus', 'blur'],
  'focus-visible': ['focusin', 'focusout'],
}

/** `true` if `el` matches `:focus-visible` semantics. Uses the
 * modern `:focus-visible` heuristic: focus came from a keyboard
 * interaction, not a mouse click. Without a real
 * `:focus-visible` selector (some older browsers), falls back to
 * checking that focus came from keyboard. */
function isFocusVisible(ev: FocusEvent): boolean {
  // Modern browsers expose `ev.target.matches(':focus-visible')`.
  const target = ev.target as Element | null
  if (target && typeof target.matches === 'function') {
    try {
      return target.matches(':focus-visible')
    } catch {
      // Selector not supported — fall through.
    }
  }
  // Fallback heuristic: tag the focus source. We track a flag on
  // the window for the first keydown vs first mousedown.
  return focusSource === 'keyboard'
}

// The focus-source tracker used to install two global listeners AT IMPORT TIME —
// a module-level side effect, which means (a) every app that so much as touches this
// module pays for two listeners it may never use, and (b) NO bundler can drop the
// module, because it cannot prove the import is pure. That is how `motion` ended up
// in a benchmark bundle that has no animations in it.
//
// It installs on first use instead: only `focus-visible` needs it.
let focusSource: 'keyboard' | 'mouse' | 'unknown' = 'unknown'
let focusTrackerInstalled = false

function trackFocusSource(): void {
  if (focusTrackerInstalled || typeof window === 'undefined') return
  focusTrackerInstalled = true
  window.addEventListener('keydown', () => {
    focusSource = 'keyboard'
  }, { capture: true })
  window.addEventListener('mousedown', () => {
    focusSource = 'mouse'
  }, { capture: true })
}

// The compiler bakes `el.animate(...)` during element CONSTRUCTION — before
// the node is inserted. Template clones (`__tpl`) are owned by the
// <template>'s inert contents document until insertion adopts them, and
// `Element.animate()` binds the Animation to the NODE DOCUMENT's default
// timeline at call time. An inert document's timeline never advances — its
// currentTime is permanently unresolved — so an animation minted there is a
// zombie: `play()` leaves it play-pending forever, holding the element at its
// FIRST keyframe (a resolved hold time beats inline styles in the animation
// cascade), while the rendering document's `getAnimations()` never lists it.
// Re-associate the effect with the rendering document's timeline before
// wiring any trigger. The Level-1 `new Animation(effect, timeline)`
// constructor re-parents the effect (the old animation is left effect-less);
// the Level-2 `animation.timeline` setter is NOT used here because engines
// without scroll-timeline support keep the attribute readonly.
function rehome(a: BakedAnimation): BakedAnimation {
  if (typeof document === 'undefined' || typeof Animation !== 'function') {
    return a
  }
  const timeline = document.timeline
  if (!timeline || a.timeline === timeline) return a
  const effect = a.effect
  if (!effect) return a
  a.cancel()
  return new Animation(effect, timeline)
}

/** Public: called by the compiler's emitted JS for every
 * `motion:` directive. Returns a disposer that cancels all
 * attached animations and removes the event listeners. */
export function __sui_motion_attach(
  el: Element,
  config: MotionAttachConfig,
): Disposer {
  const { trigger } = config
  // Re-home each animation onto the live document timeline exactly once.
  // `config.enter` and `config.leave` carry the SAME Animation objects
  // (the emit passes one baked array to both), and an effect can belong to
  // only one Animation — so the mapping must be memoized, or the second
  // array would strip the effect the first re-homing just claimed.
  const rehomed = new Map<BakedAnimation, BakedAnimation>()
  const fix = (a: BakedAnimation | null): BakedAnimation | null => {
    if (a == null) return null
    let r = rehomed.get(a)
    if (!r) {
      r = rehome(a)
      rehomed.set(a, r)
    }
    return r
  }
  // Drop the null holes from WAAPI-less environments; with nothing to
  // play, attach nothing (a disposer no-op keeps the call site uniform).
  const enter = config.enter.map(fix).filter((a): a is BakedAnimation => a != null)
  const leave = config.leave.map(fix).filter((a): a is BakedAnimation => a != null)
  if (enter.length === 0) return () => {}
  // CANCEL the enter animations until the first trigger event. Cancelled
  // means "no effect": the element rests in its authored pose. Paused would
  // apply the first keyframe (the departed pose — opacity 0, scale 0.96),
  // leaving every motion-decorated element half-vanished at rest.
  for (const a of enter) {
    a.cancel()
  }
  // Build the play/reverse actions. We play `enter` forward on the
  // enter event and reverse on the leave event.
  const playForward = () => {
    for (const a of enter) {
      a.play()
    }
  }
  const playReverse = () => {
    // For the leave phase, cancel each `enter` animation and
    // re-play the corresponding `leave` animation from the
    // start. `enter` and `leave` are AOT-paired by the compiler
    // (same length, same index).
    enter.forEach((a, i) => {
      const b = leave[i] ?? a
      try {
        a.cancel()
      } catch {
        // already finished
      }
      b.currentTime = 0
      b.play()
    })
  }
  // TRIGGER SEMANTICS — the contract each dispatch arm below implements.
  // "idle" = cancelled: no effect applied, the element rests in its authored
  // pose. Every animation is on the live document timeline by now (rehome).
  //
  //   trigger        at attach              plays when                       holds after
  //   ─────────────  ─────────────────────  ───────────────────────────────  ─────────────────────────
  //   hover          idle                   enter fwd on mouseenter;         active pose while
  //                                         leave from 0 on mouseleave       hovered; rest after
  //   press          idle                   enter fwd on pointerdown;        pressed pose while held;
  //                                         leave on pointerup/cancel        rest after release
  //   focus          idle                   enter fwd on focus;              focused pose while
  //                                         leave on blur                    focused; rest after
  //   focus-visible  idle                   enter fwd on keyboard focusin;   as focus (mouse focus
  //                                         leave on focusout                never triggers)
  //   enter          playing (immediately;  once, at attach — the element    arrived pose (the
  //                  stagger delays by      is already in the tree           animation's fill)
  //                  arrival order)
  //   exit           idle, parked in        when if/for hand the departing   element stays mounted
  //                  EXITS                  node to runLeave                 until settled, then gone
  //   view           driven by its          scrubbed by the element's own    whatever scroll position
  //                  ViewTimeline           viewport progress (no support:   dictates (fallback: ends
  //                                         play once on first reveal)       at arrived pose)
  //   scroll         driven by its          scrubbed by the container's      scroll position dictates
  //                  ScrollTimeline         scroll progress                  (no support: end state)
  //   unknown        idle                   never                            authored pose
  //
  //   prefers-reduced-motion: enter/view/scroll jump to the END state; the
  //   event triggers become no-ops; exit tears down synchronously.
  //
  // TIMELINE triggers — `motion:view` / `motion:scroll`. Progress-driven, not
  // time-driven: the animation's position is a function of SCROLL, not of the clock.
  // This is reveal-on-scroll and parallax, and where the platform supports scroll
  // timelines it costs ZERO JavaScript at runtime — no scroll listener, no rAF
  // loop. The platform drives it off the main thread, which is exactly why a
  // scroll effect built this way stays smooth under load and one built on a
  // scroll handler does not. (Browsers without timelines get the one-shot
  // IntersectionObserver fallback below — set up once, then out of the way.)
  //
  // Note what this is NOT: a second emit path. The compiler bakes the same
  // `el.animate(...)` it bakes for every other trigger; we simply hand the resulting
  // Animation a different timeline. One directive, one catalog, one code path.
  if (trigger === 'view' || trigger === 'scroll') {
    if (prefersReducedMotion()) {
      for (const a of enter) a.finish()
      return () => {
        for (const a of enter) a.cancel()
      }
    }
    const timeline = makeTimeline(trigger, el)
    if (!timeline) {
      // No scroll-timeline support (today: Firefox). `view` degrades from
      // scrub-by-scroll to play-once-on-reveal: the SAME baked animation,
      // time-driven from the element's first intersection. A static end
      // state is not a fallback for "animate as it enters the viewport" —
      // it reads as the feature being broken.
      if (trigger === 'view' && typeof IntersectionObserver !== 'undefined') {
        const io = new IntersectionObserver(
          (entries) => {
            if (entries.some((e) => e.isIntersecting)) {
              io.disconnect()
              for (const a of enter) a.play()
            }
          },
          { threshold: 0.15 },
        )
        io.observe(el)
        return () => {
          io.disconnect()
          for (const a of enter) a.cancel()
        }
      }
      // `scroll` (progress = container scroll) has no time-driven equivalent:
      // land on the END state, so it degrades to "visible" not "invisible".
      for (const a of enter) a.finish()
      return () => {
        for (const a of enter) a.cancel()
      }
    }
    for (const a of enter) {
      a.timeline = timeline
      a.play()
    }
    return () => {
      for (const a of enter) a.cancel()
    }
  }

  // LIFECYCLE triggers — `motion:enter` / `motion:exit`. These are not event
  // pairs: they fire when the element ENTERS or LEAVES the DOM, which is the one
  // thing CSS cannot do alone, because leaving requires the framework to HOLD the
  // element in the tree until its animation finishes. `if.ts` / `for.ts` already
  // do that (they await `runLeave` before removing), so the exit animation just
  // has to be findable from the node. This is what `transition:` / `in:` / `out:`
  // used to be — same mechanism, one catalog, one directive.
  if (trigger === 'enter' || trigger === 'exit') {
    installLifecycleRunner()
    if (trigger === 'enter') {
      // An entering element plays immediately: it is already in the DOM.
      if (prefersReducedMotion()) {
        for (const a of enter) a.finish()
        return () => {
          for (const a of enter) a.cancel()
        }
      }
      // ORCHESTRATION. The offset is the element's ARRIVAL ORDER among the
      // siblings entering in this same synchronous run — not its DOM index.
      // That distinction is the whole design: a list rendering for the first time
      // arrives together, so its rows cascade 0/1/2/3; a single row appended later
      // arrives ALONE, so it gets offset 0 and plays immediately instead of waiting
      // out a delay computed from a position it happens to occupy. Index-based
      // stagger gets that second case wrong, and the wrongness grows with the list.
      if (config.stagger) {
        const delay = staggerOffset() * config.stagger
        for (const a of enter) {
          a.effect?.updateTiming({ delay })
        }
      }
      playForward()
      return () => {
        for (const a of enter) a.cancel()
      }
    }
    // `exit`: park the baked animations on the node and stay out of the way until
    // the element is actually being removed. Paused, so nothing plays on mount.
    for (const a of enter) a.cancel()
    EXITS.set(el, enter)
    return () => {
      EXITS.delete(el)
      for (const a of enter) a.cancel()
    }
  }

  // Trigger dispatch.
  const events = TRIGGER_EVENTS[trigger]
  if (!events) {
    return () => {
      for (const a of enter) a.cancel()
    }
  }
  if (trigger === 'focus-visible') trackFocusSource()
  const [enterEvent, leaveEvent, cancelEvent] = events
  const onEnter = (ev: Event) => {
    if (prefersReducedMotion()) return
    if (trigger === 'focus-visible' && !isFocusVisible(ev as FocusEvent)) {
      return
    }
    playForward()
  }
  const onLeave = () => {
    if (prefersReducedMotion()) return
    playReverse()
  }
  el.addEventListener(enterEvent, onEnter as EventListener)
  el.addEventListener(leaveEvent, onLeave as EventListener)
  if (cancelEvent) {
    el.addEventListener(cancelEvent, onLeave as EventListener)
  }
  // Cleanup disposer.
  return () => {
    el.removeEventListener(enterEvent, onEnter as EventListener)
    el.removeEventListener(leaveEvent, onLeave as EventListener)
    if (cancelEvent) {
      el.removeEventListener(cancelEvent, onLeave as EventListener)
    }
    for (const a of enter) {
      try {
        a.cancel()
      } catch {
        // already finished
      }
    }
  }
}



// ── LIFECYCLE ────────────────────────────────────────────────────────────────
// The seam `if.ts` / `for.ts` already call (`runLeave(nodes)`): return a promise
// and they hold the nodes in the DOM until it settles; return null and they tear
// down synchronously, exactly as before. It used to be backed by the `transition:`
// catalog. It is backed by `motion:` now — same seam, one catalog.

const EXITS = new WeakMap<Element, BakedAnimation[]>()
let runnerInstalled = false

function installLifecycleRunner(): void {
  if (runnerInstalled) return
  runnerInstalled = true
  setTransitionRunner({
    // Entry needs nothing here: `motion:enter` plays on attach, because by the
    // time the action runs the element is in the tree.
    runEnter: () => null,

    runLeave: (nodes: Iterable<Node>): Promise<void> | null => {
      const playing: Promise<unknown>[] = []
      for (const n of nodes) {
        if (!(n instanceof Element)) continue
        // The leaving node may be a wrapper — the exit may be on a descendant.
        const targets = EXITS.has(n) ? [n] : [...n.querySelectorAll('*')].filter((e) => EXITS.has(e))
        for (const t of targets) {
          const anims = EXITS.get(t)
          if (!anims) continue
          for (const a of anims) {
            a.currentTime = 0
            a.play()
            playing.push(a.finished.catch(() => {}))
          }
        }
      }
      // Null (not an empty promise) when nothing is animating — that is the
      // signal `if.ts` reads to keep the synchronous teardown path intact.
      if (playing.length === 0) return null
      if (prefersReducedMotion()) return null
      return Promise.all(playing).then(() => {})
    },

    cancelTransitions: (node: Node): void => {
      if (!(node instanceof Element)) return
      const anims = EXITS.get(node)
      if (anims) for (const a of anims) a.cancel()
    },
  })
}


// ── ORCHESTRATION ────────────────────────────────────────────────────────────
// Everything that enters within one synchronous run is one wave. The counter
// resets on the microtask that follows, so the NEXT wave starts from zero.
//
// Stagger is compile-time arithmetic wearing a runtime hat: there is no timeline,
// no scheduler, no engine — each element simply gets `delay = arrivalIndex * step`
// pushed into its already-baked WAAPI animation. The cascade runs on the
// compositor like every other motion here.

let waveCount = 0
let waveScheduled = false

function staggerOffset(): number {
  const i = waveCount++
  if (!waveScheduled) {
    waveScheduled = true
    queueMicrotask(() => {
      waveCount = 0
      waveScheduled = false
    })
  }
  return i
}


// ── TIMELINES ────────────────────────────────────────────────────────────────
// `view()` — progress is the element's own passage through the viewport. This is
// the one you want almost always: reveals, parallax, anything that should respond
// to "how far into view am I".
//
// `scroll()` — progress is the nearest scroll container's total progress. For the
// things that track the DOCUMENT rather than themselves: reading-progress bars, a
// header that shrinks, a scrubbing timeline.

interface TimelineCtors {
  ViewTimeline?: new (opts: { subject: Element }) => AnimationTimeline
  ScrollTimeline?: new (opts: { source: Element | null }) => AnimationTimeline
}

function makeTimeline(kind: 'view' | 'scroll', el: Element): AnimationTimeline | null {
  const g = globalThis as unknown as TimelineCtors
  try {
    if (kind === 'view') {
      return g.ViewTimeline ? new g.ViewTimeline({ subject: el }) : null
    }
    return g.ScrollTimeline ? new g.ScrollTimeline({ source: document.scrollingElement }) : null
  } catch {
    return null
  }
}
