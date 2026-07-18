// `timeline()` — keyframes as DATA, choreography as signals.
//
// The style layer already reflects controller signals into custom properties
// (`return { tileX }` → `--tile-x` → `sx="translate-x-(--tile-x)"`). What was
// missing is the layer above it: a way to say "this channel is 0px at 0,
// 120px at .34, and HOLDS there until 1" without hand-rolling
// `Math.min/Math.max` offset math in an `effect`.
//
// This module is that layer and nothing more. It takes a progress reader
// (any `() => number` — a slider signal, `clock()`, `scrollProgress()`) and a
// plain-data track map, and returns one `derived` per track. No DOM, no
// element references, no `el.style`. The returned map is what a controller
// returns; the existing custom-property reflection does the rest.
//
// Why `derived` and not `effect`: deriveds are lazy and memoized. A track
// nobody reads costs nothing per frame, and a track whose interpolated value
// didn't change (a HOLD segment — the common case in real choreography)
// notifies no observers, so the style write is skipped. Hand-rolled effect
// math re-wrote every channel every frame.
//
// Two lanes, one story (see also `motion.ts`):
//   - ambient, fire-and-forget loops  → `motion:` (WAAPI, compositor-driven)
//   - scrubbed / choreographed timing → `timeline()` (signals, main thread)
// `motion:` cannot express a 7-stop compound curve driven by a scroll
// position; `timeline()` cannot hand work to the compositor. Pick by whether
// something outside the animation owns its clock.

import { interpolate } from 'motion-dom'
import { cubicBezier } from 'motion'
import { derived, type Derived } from './signal'

// ── Easing ───────────────────────────────────────────────────────────────────

/** The CSS-named easings, spelled exactly as CSS spells them. */
const NAMED_EASINGS = {
  linear: (t: number) => t,
  ease: cubicBezier(0.25, 0.1, 0.25, 1),
  'ease-in': cubicBezier(0.42, 0, 1, 1),
  'ease-out': cubicBezier(0, 0, 0.58, 1),
  'ease-in-out': cubicBezier(0.42, 0, 0.58, 1),
} as const

/** A named CSS easing accepted anywhere an easing is accepted. */
export type TimelineEasingName = keyof typeof NAMED_EASINGS

/**
 * An easing, in any of the three shapes real keyframe data comes in:
 * a cubic-bezier control-point tuple (what `cubic-bezier(a, b, c, d)` and
 * every design tool exports), a CSS easing name, or a raw `0..1 → 0..1`
 * function for the cases neither covers.
 */
export type TimelineEasing =
  | readonly [number, number, number, number]
  | TimelineEasingName
  | ((t: number) => number)

function resolveEasing(easing: TimelineEasing, where: string): (t: number) => number {
  if (typeof easing === 'function') return easing
  if (typeof easing === 'string') {
    const fn = NAMED_EASINGS[easing]
    if (!fn) {
      throw new SailTimelineError(
        `${where}: unknown easing '${easing}'. Use a cubic-bezier tuple ` +
          `[x1, y1, x2, y2], or one of: ${Object.keys(NAMED_EASINGS).join(', ')}.`,
      )
    }
    return fn
  }
  if (!Array.isArray(easing) || easing.length !== 4 || easing.some((n) => typeof n !== 'number')) {
    throw new SailTimelineError(
      `${where}: easing must be a 4-number cubic-bezier tuple, an easing name, or a function.`,
    )
  }
  const [x1, y1, x2, y2] = easing as readonly [number, number, number, number]
  return cubicBezier(x1, y1, x2, y2)
}

// ── Track data ───────────────────────────────────────────────────────────────

/**
 * A value a track interpolates. Numbers interpolate numerically; unit
 * strings (`'120px'`, `'-12%'`, `'15deg'`) interpolate their numeric part
 * and keep their unit. See {@link timeline} for the unit rules.
 */
export type TimelineValue = number | string

/** Per-stop options. `easing` governs the segment STARTING at this stop. */
export interface TimelineStopOptions {
  easing?: TimelineEasing
}

/**
 * One keyframe: an offset in `0..1`, the value at that offset, and
 * optionally the easing for the segment that STARTS here.
 */
export type TimelineStop<V extends TimelineValue = TimelineValue> =
  | readonly [offset: number, value: V]
  | readonly [offset: number, value: V, options: TimelineStopOptions]

/** A named channel: an ascending list of stops. */
export type TimelineTrack<V extends TimelineValue = TimelineValue> = readonly TimelineStop<V>[]

/** The data a `timeline()` call is built from: channel name → stops. */
export type TimelineTracks = Record<string, TimelineTrack>

/** Recover a track's value type so the returned signals stay typed. */
type TrackValue<T> = T extends TimelineTrack<infer V> ? V : TimelineValue

/** What `timeline()` returns: one read-only signal per track, same keys. */
export type TimelineSignals<T extends TimelineTracks> = {
  readonly [K in keyof T]: Derived<TrackValue<T[K]>>
}

export interface TimelineOptions {
  /**
   * Easing applied to every segment that does not name its own. Defaults to
   * `'linear'` — with `linear`, a `timeline()` is exactly the geometry its
   * data describes, and any curve you see is one you asked for.
   */
  easing?: TimelineEasing
}

/** Thrown for malformed track data. Timeline data is validated eagerly. */
export class SailTimelineError extends Error {
  override name = 'SailTimelineError'
}

// ── Unit handling ────────────────────────────────────────────────────────────

// Deliberately strict. A track is EITHER all numbers or all strings sharing
// one unit suffix; anything else throws at construction. The alternative —
// motion-dom's `mixComplex`, which will happily blend `'0px'` into `'50%'` —
// produces values that render but are silently wrong, and the failure shows
// up as "the animation is subtly off" three layers downstream. Colors and
// multi-part values (`'0 0 4px red'`) are not v1; a track needing one should
// be split into its numeric channels, which is what the custom-property
// reflection wants anyway.
const UNIT_VALUE = /^\s*(-?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)\s*([a-z%]*)\s*$/i

interface ParsedValue {
  number: number
  unit: string
}

function parseValue(value: TimelineValue, where: string): ParsedValue {
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) {
      throw new SailTimelineError(`${where}: value must be a finite number, got ${value}.`)
    }
    return { number: value, unit: '' }
  }
  if (typeof value !== 'string') {
    throw new SailTimelineError(
      `${where}: value must be a number or a unit string like '120px'; got ${typeof value}.`,
    )
  }
  const m = UNIT_VALUE.exec(value)
  if (!m) {
    throw new SailTimelineError(
      `${where}: '${value}' is not an interpolable value. A track holds numbers, or ` +
        `strings of one number and one unit ('120px', '-12%', '15deg'). Colors and ` +
        `multi-part values are not supported — split them into numeric channels.`,
    )
  }
  return { number: Number(m[1]), unit: m[2] ?? '' }
}

// ── Build ────────────────────────────────────────────────────────────────────

interface CompiledTrack {
  offsets: number[]
  numbers: number[]
  easings: ((t: number) => number)[]
  /** '' for a numeric track; the shared suffix for a unit-string track. */
  unit: string
  /** True when the authored values were strings — preserved on output. */
  stringly: boolean
}

function compileTrack(
  name: string,
  track: TimelineTrack,
  defaultEasing: (t: number) => number,
): CompiledTrack {
  const where = `timeline track '${name}'`
  if (!Array.isArray(track) || track.length === 0) {
    throw new SailTimelineError(`${where}: needs at least one stop.`)
  }

  const offsets: number[] = []
  const numbers: number[] = []
  const easings: ((t: number) => number)[] = []
  let unit: string | null = null
  let stringly = false

  for (let i = 0; i < track.length; i++) {
    const stop = track[i] as TimelineStop | undefined
    if (!Array.isArray(stop) || stop.length < 2) {
      throw new SailTimelineError(
        `${where} stop ${i}: expected [offset, value] or [offset, value, { easing }].`,
      )
    }
    const [offset, value, options] = stop as readonly [number, TimelineValue, TimelineStopOptions?]

    if (typeof offset !== 'number' || !Number.isFinite(offset) || offset < 0 || offset > 1) {
      throw new SailTimelineError(
        `${where} stop ${i}: offset must be a number in 0..1, got ${String(offset)}.`,
      )
    }
    const previous = i > 0 ? (offsets[i - 1] as number) : -1
    if (i > 0 && offset <= previous) {
      throw new SailTimelineError(
        `${where} stop ${i}: offsets must ascend — ${offset} follows ${previous}. ` +
          `To HOLD a value, repeat the VALUE at a later offset, not the offset.`,
      )
    }

    const parsed = parseValue(value, `${where} stop ${i}`)
    if (unit === null) {
      unit = parsed.unit
      stringly = typeof value === 'string'
    } else if (parsed.unit !== unit) {
      throw new SailTimelineError(
        `${where} stop ${i}: unit '${parsed.unit || '<none>'}' does not match the track's ` +
          `'${unit || '<none>'}'. Every stop in a track must share one unit.`,
      )
    } else if (typeof value === 'string') {
      stringly = true
    }

    offsets.push(offset)
    numbers.push(parsed.number)
    if (i < track.length - 1) {
      // WAAPI semantics: a keyframe's easing governs the segment that
      // STARTS at it, so the final stop's easing is meaningless and the
      // easing list is always one shorter than the stop list.
      easings.push(
        options?.easing !== undefined
          ? resolveEasing(options.easing, `${where} stop ${i}`)
          : defaultEasing,
      )
    }
  }

  return { offsets, numbers, easings, unit: unit ?? '', stringly }
}

/**
 * Turn a progress reader and a map of keyframe tracks into a map of signals.
 *
 * `timeline()` is the choreography lane: one clock, many channels, keyframes
 * expressed as data rather than as arithmetic. Return its result from a
 * controller and every track becomes a custom property the style layer can
 * read.
 *
 * ### Progress
 * Any `() => number` works — a `signal(0)` a slider writes, a {@link clock},
 * a {@link scrollProgress}, a {@link springValue}. Progress is in `0..1` and
 * is CLAMPED: below the first offset every track holds its first value,
 * above the last offset its last.
 *
 * ### Offsets and holds
 * Offsets are `0..1` and must ascend. A HOLD is the same VALUE repeated at a
 * later offset — `[[0.34, '120px'], [0.44, '120px']]` settles at `.34` and
 * stays put until `.44`. (Repeating an *offset* is an error; it is almost
 * always a typo for a hold.)
 *
 * ### Easing
 * A stop's `{ easing }` governs the segment that STARTS at that stop — the
 * same rule WAAPI keyframes use, so captured `cubic-bezier(...)` values
 * transfer verbatim. Segments with no easing of their own use
 * `options.easing`, which defaults to `'linear'`.
 *
 * ### Units
 * A track is either all numbers or all unit strings sharing ONE unit.
 * Numbers stay numbers on the way out (`1.75`); unit strings come back as
 * strings with their unit (`'84.2px'`). Mixed units throw at construction
 * rather than silently interpolating nonsense. Split multi-part values
 * (colors, shadows) into numeric channels.
 *
 * @param progress Reader returning progress in `0..1`.
 * @param tracks Channel name → ascending keyframe stops.
 * @param options Timeline-wide defaults.
 * @returns One memoized signal per track, keyed identically.
 *
 * @example
 * ```ts
 * const p = signal(0)
 * const tile = timeline(p, {
 *   tileX: [[0, '0px'], [0.12, '0px'], [0.34, '120px'], [1, '0px']],
 *   tileScale: [[0.44, 1], [0.54, 1.75], [0.64, 1]],
 * }, { easing: [0.6, 0, 0.2, 1] })
 * return { p, ...tile }   // → --tile-x, --tile-scale
 * ```
 *
 * @see clock, scrollProgress, springValue for stock progress drivers.
 */
export function timeline<const T extends TimelineTracks>(
  progress: () => number,
  tracks: T,
  options: TimelineOptions = {},
): TimelineSignals<T> {
  if (typeof progress !== 'function') {
    throw new SailTimelineError(
      'timeline(progress, tracks): progress must be a signal or a () => number reader.',
    )
  }
  if (tracks === null || typeof tracks !== 'object') {
    throw new SailTimelineError('timeline(progress, tracks): tracks must be an object of tracks.')
  }

  const defaultEasing = resolveEasing(options.easing ?? 'linear', 'timeline options')
  const out: Record<string, unknown> = {}

  for (const name of Object.keys(tracks)) {
    const { offsets, numbers, easings, unit, stringly } = compileTrack(
      name,
      tracks[name] as TimelineTrack,
      defaultEasing,
    )

    // Single-stop track: a constant. `interpolate` needs two points.
    const only = numbers[0] as number
    const sample: (p: number) => number =
      offsets.length === 1
        ? () => only
        : interpolate(offsets, numbers, { ease: easings, clamp: true })

    // One `derived` per track: lazy (an unread channel costs nothing) and
    // memoized (a HOLD segment recomputes to an `Object.is`-equal value, so
    // downstream style writes are skipped for the whole hold).
    out[name] = stringly
      ? derived(() => `${sample(progress())}${unit}`)
      : derived(() => sample(progress()))
  }

  return out as unknown as TimelineSignals<T>
}
