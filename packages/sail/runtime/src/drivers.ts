// Progress drivers — the stock things that move a `timeline()`.
//
// A driver's whole contract is: it returns a plain signal carrying progress
// in `0..1`. That is deliberately the SAME shape a hand-written
// `signal(0)` a range input writes has, so the designer's slider spike and a
// production `clock()` are the same program with one line swapped, and
// `timeline()` never learns what is driving it.
//
// Every driver stops itself when the enclosing scope disposes. There is no
// "remember to cancel" step, because the failure mode of forgetting is a
// frame loop that survives navigation and keeps writing custom properties to
// a detached element.
//
// Reduced motion is decided HERE, once, per driver — not by each caller.
// See `clock()` for the rule and why it is jump-to-end.

import { frame, cancelFrame, motionValue, springValue as motionSpringValue } from 'motion-dom'
import type { SpringOptions } from 'motion-dom'
import { signal, effect, type Signal } from './signal'
import { onCleanup } from './effect-scope'

// ── Shared plumbing ──────────────────────────────────────────────────────────

/** Live read — never cached, so an OS-level change is picked up mid-session. */
function prefersReducedMotion(): boolean {
  if (typeof matchMedia === 'undefined') return false
  return matchMedia('(prefers-reduced-motion: reduce)').matches
}

/**
 * Register `stop` with the enclosing scope when there is one. Outside a
 * scope (a test, a module-level driver) the caller owns disposal via the
 * returned handle.
 */
function bindDisposal(stop: () => void): void {
  try {
    onCleanup(stop)
  } catch {
    // `onCleanup` outside a scope is the caller-owns-it case, not an error.
  }
}

// ── clock ────────────────────────────────────────────────────────────────────

export interface ClockOptions {
  /**
   * Repeat when the run finishes. `true` repeats forever, a number repeats
   * that many total iterations. Default `false` — one pass, then hold at the
   * end.
   */
  loop?: boolean | number
  /**
   * `'normal'` runs 0→1. `'reverse'` runs 1→0. `'alternate'` runs 0→1 then
   * 1→0, which is what a "breathing" loop wants and what you would
   * otherwise fake with a second mirrored track. Default `'normal'`.
   */
  direction?: 'normal' | 'reverse' | 'alternate'
  /** Start on creation. Default `true`. */
  autoplay?: boolean
  /**
   * What to do under `prefers-reduced-motion: reduce`. Default `'end'`.
   * @see clock for the rationale.
   */
  reducedMotion?: 'end' | 'start' | 'play'
}

/**
 * A progress signal with transport controls. It reads exactly like any
 * other signal — `clock(3600)()` is a number in `0..1` — so it drops into
 * `timeline()` and into `(--name)` reflection with no special-casing.
 */
export interface ClockSignal extends Signal<number> {
  /** Resume (or start) advancing. No-op while already running. */
  play(): void
  /** Freeze at the current progress. `play()` resumes from here. */
  pause(): void
  /** Freeze AND rewind to the start of the first iteration. */
  stop(): void
  /** Is the frame loop currently running? */
  running(): boolean
}

/**
 * Drive progress from wall-clock time.
 *
 * The returned signal advances `0 → 1` across `durationMs`, on motion.dev's
 * frame loop (the same loop `fx:` animations are scheduled on, so a
 * page mixing both lanes still has exactly one rAF).
 *
 * ### Reduced motion
 * Under `prefers-reduced-motion: reduce` the default is `'end'`: the signal
 * is set to the value the animation would have FINISHED at (`1`, or `0` for
 * `direction: 'reverse'`) and the frame loop never starts. This mirrors what
 * `motion.ts` does for `fx:` directives — the reduced-motion user sees
 * the settled, intended layout rather than a first frame frozen mid-flight.
 * `'start'` holds at the beginning instead; `'play'` opts out and animates
 * anyway, which is only correct when the motion IS the content (a progress
 * bar, a loading indicator).
 *
 * The clock stops when the enclosing scope disposes.
 *
 * @param durationMs One iteration's length in milliseconds.
 * @param options Loop / direction / autoplay / reduced-motion behavior.
 * @returns Progress signal in `0..1`, with `play`/`pause`/`stop`.
 *
 * @example
 * ```ts
 * const p = clock(3600, { loop: true })
 * const { tileX } = timeline(p, { tileX: [[0, '0px'], [0.34, '120px'], [1, '0px']] })
 * return { p, tileX }
 * ```
 *
 * @see timeline, scrollProgress, springValue.
 */
export function clock(durationMs: number, options: ClockOptions = {}): ClockSignal {
  if (typeof durationMs !== 'number' || !Number.isFinite(durationMs) || durationMs <= 0) {
    throw new Error(`clock(durationMs): duration must be a positive number, got ${durationMs}.`)
  }

  const {
    loop = false,
    direction = 'normal',
    autoplay = true,
    reducedMotion = 'end',
  } = options

  const totalIterations = loop === true ? Infinity : loop === false ? 1 : Math.max(1, loop)
  const progress = signal(direction === 'reverse' ? 1 : 0) as ClockSignal

  let elapsed = 0
  let iteration = 0
  let active = false

  const shape = (t: number): number => {
    // `t` is 0..1 within the current iteration.
    if (direction === 'reverse') return 1 - t
    if (direction === 'alternate') return iteration % 2 === 0 ? t : 1 - t
    return t
  }

  const tick = ({ delta }: { delta: number }): void => {
    elapsed += delta
    while (elapsed >= durationMs) {
      if (iteration + 1 >= totalIterations) {
        elapsed = durationMs
        progress.set(shape(1))
        pause()
        return
      }
      elapsed -= durationMs
      iteration += 1
    }
    progress.set(shape(elapsed / durationMs))
  }

  const play = (): void => {
    if (active) return
    active = true
    // `keepAlive: true` — the loop must re-schedule itself every frame even
    // when nothing else on the page is animating.
    frame.update(tick, true)
  }

  const pause = (): void => {
    if (!active) return
    active = false
    cancelFrame(tick)
  }

  const stop = (): void => {
    pause()
    elapsed = 0
    iteration = 0
    progress.set(shape(0))
  }

  progress.play = play
  progress.pause = pause
  progress.stop = stop
  progress.running = () => active

  bindDisposal(pause)

  if (reducedMotion !== 'play' && prefersReducedMotion()) {
    // Land on the settled value and never schedule a frame.
    elapsed = reducedMotion === 'end' ? durationMs : 0
    iteration = 0
    progress.set(shape(reducedMotion === 'end' ? 1 : 0))
    // Transport controls stay live: a caller that knows better (a user
    // pressing "play this animation") can still start it explicitly.
  } else if (autoplay) {
    play()
  }

  return progress
}

// ── scrollProgress ───────────────────────────────────────────────────────────

export interface ScrollProgressOptions {
  /** Scroll axis. Default `'y'`. */
  axis?: 'x' | 'y'
}

/**
 * Drive progress from scroll position.
 *
 * With no `target`, progress is the document's own scroll: `0` at the top,
 * `1` when scrolled to the bottom. With a `target` element, progress is that
 * element's travel through the viewport: `0` the moment its leading edge
 * enters, `1` the moment its trailing edge leaves — CSS's `cover 0% → 100%`
 * range, spelled the same way in both implementations below.
 *
 * Two implementations, one definition. Where `ScrollTimeline` /
 * `ViewTimeline` exist, the browser measures and we only read; elsewhere a
 * frame-throttled measurement computes the identical quantity. Both are
 * normalized to the same `0..1`, so a fallback browser is not a different
 * animation — it is the same animation measured in JS.
 *
 * There is no reduced-motion branch here on purpose: scroll-driven progress
 * is not motion the page initiates, it is a readout of something the user is
 * already doing. Suppressing it would freeze the page mid-scroll.
 *
 * The listener/timeline is released when the enclosing scope disposes.
 *
 * @param target Element (or a getter for one) whose viewport travel drives
 *   progress. Omit for document scroll.
 * @param options Axis selection.
 * @returns Progress signal in `0..1`.
 *
 * @see timeline, clock.
 */
export function scrollProgress(
  target?: Element | (() => Element | null | undefined) | null,
  options: ScrollProgressOptions = {},
): Signal<number> {
  const axis = options.axis ?? 'y'
  const progress = signal(0)

  // SSR / jsdom: no scroller to read. Stay at 0 rather than throwing, so a
  // controller using scrollProgress still renders on the server.
  if (typeof window === 'undefined' || typeof document === 'undefined') return progress

  const resolveTarget = (): Element | null => {
    if (typeof target === 'function') return target() ?? null
    return target ?? null
  }

  const clamp01 = (n: number): number => (n < 0 ? 0 : n > 1 ? 1 : n)

  const measure = (): number => {
    const el = resolveTarget()
    if (el) {
      const rect = el.getBoundingClientRect()
      const viewport = axis === 'y' ? window.innerHeight : window.innerWidth
      const start = axis === 'y' ? rect.top : rect.left
      const size = axis === 'y' ? rect.height : rect.width
      // `cover`: 0 when the leading edge is at the far viewport edge,
      // 1 when the trailing edge has passed the near one.
      const travelled = viewport - start
      const total = viewport + size
      return total <= 0 ? 0 : clamp01(travelled / total)
    }
    const doc = document.documentElement
    const max =
      axis === 'y' ? doc.scrollHeight - doc.clientHeight : doc.scrollWidth - doc.clientWidth
    if (max <= 0) return 0
    return clamp01((axis === 'y' ? window.scrollY : window.scrollX) / max)
  }

  // Measurement is cheap but layout-reading; coalesce to one read per frame
  // so a burst of scroll events can't force multiple reflows.
  let queued = false
  const read = (): void => {
    queued = false
    progress.set(measure())
  }
  const schedule = (): void => {
    if (queued) return
    queued = true
    frame.read(read)
  }

  window.addEventListener('scroll', schedule, { passive: true })
  window.addEventListener('resize', schedule, { passive: true })
  read()

  const stop = (): void => {
    window.removeEventListener('scroll', schedule)
    window.removeEventListener('resize', schedule)
    if (queued) cancelFrame(read)
  }
  bindDisposal(stop)

  return progress
}

// ── springValue ──────────────────────────────────────────────────────────────

export type { SpringOptions }

/**
 * Follow another signal with real spring physics.
 *
 * `springValue(source)` returns a signal that chases `source()` — it does not
 * jump to it. Writing `source` sets a new target; the returned signal
 * integrates toward it on motion.dev's frame loop and settles. This is the
 * one driver whose output is not necessarily bounded to `0..1`: it carries
 * whatever units its source carries, so it is equally a progress driver for
 * `timeline()` and a value to reflect directly (`translate-y-(--ball-y)`).
 *
 * Under `prefers-reduced-motion: reduce` the output mirrors the source
 * instantly — the value still arrives, it just doesn't travel.
 *
 * The spring is released when the enclosing scope disposes.
 *
 * @param source Reader for the target value.
 * @param options Spring physics (`stiffness`, `damping`, `mass`, `bounce`,
 *   `visualDuration`, …) — motion.dev's own `SpringOptions`.
 * @returns Signal carrying the spring's current value.
 *
 * @example
 * ```ts
 * const dropped = signal(0)
 * const ballY = springValue(dropped, { stiffness: 220, damping: 14 })
 * return { dropped, ballY }   // → --ball-y, a real bouncing drop
 * ```
 *
 * @see timeline, clock, scrollProgress.
 */
export function springValue(source: () => number, options: SpringOptions = {}): Signal<number> {
  const initial = source()
  const out = signal(initial)

  if (prefersReducedMotion()) {
    effect(() => out.set(source()))
    return out
  }

  const target = motionValue(initial)
  const spring = motionSpringValue(target, options)

  // `on('change')` fires on the frame loop as the spring integrates. Writing
  // through the signal means a settled spring (same value twice) notifies
  // nobody — `signal.set` bails on `Object.is`-equal writes.
  const unsubscribe = spring.on('change', (v: number) => out.set(v))

  // Re-runs whenever `source` changes: a new target, not a new spring, so
  // velocity carries through a retarget mid-flight (which is the whole
  // reason to use a spring for interruptible motion).
  effect(() => target.set(source()))

  const stop = (): void => {
    unsubscribe()
    spring.destroy()
    target.destroy()
  }
  bindDisposal(stop)

  return out
}
