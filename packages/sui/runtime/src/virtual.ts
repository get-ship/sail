// Backbone #2 — createVirtualizer(): the reactive adapter over the
// `@tanstack/virtual-core` windowing engine (devDependency; the bundler
// inlines it at build, so the published pack carries no runtime dep).
//
// This is the ONLY consumer of `@tanstack/virtual-core`. Consumers bind to the
// `Virtual<T>` surface re-exported from the runtime barrel — never to the
// upstream core directly. "One contract per boundary": if the upstream engine
// changes internals, only this file moves.
//
// Signal-graph mechanics (build-spec §"Signal-graph mechanics"):
//   1. One internal `tick = signal(0)` is the single reactive SOURCE. The
//      core's `onChange` bumps it inside `untrack` so a tracked render that
//      triggers onChange can never accidentally subscribe to the tick.
//   2. `virtualItems`/`totalSize`/`range` are `derived()` that read `tick()`
//      then PULL `core.getVirtualItems()`/`getTotalSize()`/`range`.
//   3. An `effect()` bridges the reactive `count()` (and other reactive
//      options) into `core.setOptions(...)`.
//   4. `getScrollElement` is a thunk; an `effect` wires the DOM observers via
//      `_willUpdate()` whenever the element identity changes (null until mount).
//   5. A viewport ResizeObserver re-windows on viewport resize.
//   6. Every effect/observer registers teardown via `onCleanup` (mirrors
//      `live()` / `data()`); `dispose()` is also returned for manual teardown.
//
// Identity short-circuit (build-spec §"Identity short-circuit — corrected"):
//   `virtualItems` shallow-compares key + start + size, NOT key alone. A pure
//   no-op / scroll-only frame preserves array identity so `{#for}`'s
//   `next === lastItems` short-circuit skips reconcile; a MEASURE frame (a
//   measured size differing from the estimate) DOES change start/size and so
//   MUST break identity to force a re-render.

// Reactive core via the BARE self-specifier, not relative paths (mirrors
// template-hooks/index.ts). This module is a `@ship/sui-runtime/virtual`
// subpath OFF the root barrel (#297): consumer bundles inline it (plus
// `@tanstack/virtual-core`), so its runtime imports must stay external —
// the deck bundler externalizes the bare `@ship/sui-runtime` specifier,
// keeping ONE signal graph. Relative imports here would inline a second
// copy of the reactive core into every consumer (split-brain signals).
import {
  signal,
  derived,
  effect,
  untrack,
  onCleanup,
  type Derived,
  type Disposer,
} from '@ship/sui-runtime'
import {
  Virtualizer,
  elementScroll,
  observeElementOffset,
  type VirtualItem as CoreVirtualItem,
} from '@tanstack/virtual-core'

export type VirtualItem = CoreVirtualItem

export interface VirtualOptions<TScroll extends Element = HTMLElement> {
  /** Reactive item count (read is TRACKED) — binds to `centerRows().length`. */
  count: () => number
  estimateSize: (index: number) => number
  /** Thunk — `null` until mount (e.g. `() => state.viewportEl`). */
  getScrollElement: () => TScroll | null
  horizontal?: boolean
  overscan?: number
  paddingStart?: number
  paddingEnd?: number
  gap?: number
  /** Sticky-header height — the windowing offset accounts for it. */
  scrollMargin?: number
  getItemKey?: (index: number) => string | number
  lanes?: number
  /** Surface `range()` changes for `onReachEnd` wiring. */
  onChange?: (v: Virtual) => void
}

export interface Virtual {
  /** Visible slice + overscan. Stable identity on no-op frames; identity
   *  BREAKS on measure frames (key + start + size compared). */
  virtualItems: Derived<readonly VirtualItem[]>
  /** Drives the spacer height/width. */
  totalSize: Derived<number>
  /** For "showing X–Y of N" / ARIA / onReachEnd. */
  range: Derived<{ startIndex: number; endIndex: number }>
  /** Imperative — call per rendered row from mount/effect. */
  measureElement: (el: Element | null) => void
  scrollToIndex: (
    i: number,
    opts?: { align?: 'auto' | 'start' | 'center' | 'end'; behavior?: 'auto' | 'smooth' },
  ) => void
  scrollToOffset: (
    px: number,
    opts?: { align?: 'auto' | 'start' | 'center' | 'end'; behavior?: 'auto' | 'smooth' },
  ) => void
  measure: () => void
  /** Auto via onCleanup in a mount scope; returned for manual teardown. */
  dispose: Disposer
}

/** Shallow identity comparison — key + start + size per the corrected
 *  short-circuit. Pure-scroll frames compare equal (identity preserved);
 *  measure frames differ in start/size (identity breaks). */
function sameWindow(
  a: readonly VirtualItem[],
  b: readonly VirtualItem[],
): boolean {
  if (a === b) return true
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i++) {
    const x = a[i]!
    const y = b[i]!
    if (x.key !== y.key || x.start !== y.start || x.size !== y.size) return false
  }
  return true
}

const EMPTY_RANGE = { startIndex: 0, endIndex: 0 } as const

export function createVirtualizer<TScroll extends Element = HTMLElement>(
  options: VirtualOptions<TScroll>,
): Virtual {
  // (1) the single reactive source. Every core onChange bumps it.
  const tick = signal(0)

  // Snapshot the reactive count once for the initial construction; the
  // bridge effect keeps it live thereafter.
  const initialCount = untrack(() => options.count())

  // (4) the core. `getScrollElement` is the live thunk; `observeElementRect`
  // is supplied locally so it reads offsetHeight/offsetWidth-derived size
  // (the thunked element) rather than depending on layout that jsdom/tests
  // do not populate. `elementScroll`/`observeElementOffset` are vendored.
  const core = new Virtualizer<TScroll, Element>({
    count: initialCount,
    estimateSize: options.estimateSize,
    getScrollElement: options.getScrollElement,
    scrollToFn: elementScroll,
    observeElementRect,
    observeElementOffset,
    overscan: options.overscan ?? 5,
    horizontal: options.horizontal ?? false,
    paddingStart: options.paddingStart ?? 0,
    paddingEnd: options.paddingEnd ?? 0,
    gap: options.gap ?? 0,
    scrollMargin: options.scrollMargin ?? 0,
    lanes: options.lanes ?? 1,
    ...(options.getItemKey ? { getItemKey: options.getItemKey } : {}),
    // (1) onChange is the single WRITE site — bump tick untracked.
    onChange: () => {
      untrack(() => tick.set((n) => n + 1))
    },
  })

  // (3) bridge reactive options into the core. Reading `options.count()` here
  // tracks it, so a count change re-windows. We re-pull every reactive option
  // and re-arm the observers via `_willUpdate()` (which is a no-op when the
  // scroll element identity is unchanged).
  const stopBridge = effect(() => {
    const count = options.count()
    core.setOptions({
      ...core.options,
      count,
      estimateSize: options.estimateSize,
      getScrollElement: options.getScrollElement,
      overscan: options.overscan ?? 5,
      horizontal: options.horizontal ?? false,
      paddingStart: options.paddingStart ?? 0,
      paddingEnd: options.paddingEnd ?? 0,
      gap: options.gap ?? 0,
      scrollMargin: options.scrollMargin ?? 0,
      lanes: options.lanes ?? 1,
    })
    // (4) wire/re-wire DOM observers when the element appears or swaps. Safe
    // when null (the core no-ops + notifies). Re-window after re-arm.
    core._willUpdate()
    untrack(() => tick.set((n) => n + 1))
  })

  // (4 cont.) `_didMount` registers the resize/scroll teardown; call it once
  // and enroll its cleanup. A separate viewport ResizeObserver re-windows on
  // viewport resize (the core only observes ROW elements via measureElement).
  const stopMount = core._didMount()
  let viewportRO: ResizeObserver | null = null
  let observedEl: TScroll | null = null
  const stopViewportResize = effect(() => {
    const el = options.getScrollElement()
    if (el === observedEl) return
    if (viewportRO) {
      viewportRO.disconnect()
      viewportRO = null
    }
    observedEl = el
    if (el && typeof ResizeObserver !== 'undefined') {
      viewportRO = new ResizeObserver(() => {
        core._willUpdate()
        untrack(() => tick.set((n) => n + 1))
      })
      viewportRO.observe(el)
    }
  })

  // (2) the derived pull surfaces. Each reads `tick()` first so a core change
  // re-derives, then pulls the windowed state.
  let lastItems: readonly VirtualItem[] = []
  const virtualItems = derived<readonly VirtualItem[]>(() => {
    tick()
    const next = core.getVirtualItems()
    // Identity short-circuit: preserve the prior array when key+start+size are
    // unchanged (pure-scroll / no-op), break it on measure frames.
    if (sameWindow(lastItems, next)) return lastItems
    lastItems = next
    return next
  })

  const totalSize = derived<number>(() => {
    tick()
    return core.getTotalSize()
  })

  const range = derived<{ startIndex: number; endIndex: number }>(() => {
    tick()
    return core.range ?? EMPTY_RANGE
  })

  // (5) imperative methods — called from on:* / mount only (Cardinals 2 & 3).
  const measureElement = (el: Element | null): void => {
    core.measureElement(el)
  }
  const scrollToIndex: Virtual['scrollToIndex'] = (i, opts) => {
    core.scrollToIndex(i, opts)
  }
  const scrollToOffset: Virtual['scrollToOffset'] = (px, opts) => {
    core.scrollToOffset(px, opts)
  }
  const measure = (): void => {
    core.measure()
  }

  let disposed = false
  const dispose: Disposer = () => {
    if (disposed) return
    disposed = true
    stopBridge()
    stopViewportResize()
    if (viewportRO) {
      viewportRO.disconnect()
      viewportRO = null
    }
    stopMount()
  }
  onCleanup(dispose)

  const api: Virtual = {
    virtualItems,
    totalSize,
    range,
    measureElement,
    scrollToIndex,
    scrollToOffset,
    measure,
    dispose,
  }

  // (6 seam) surface range changes via onChange for onReachEnd wiring. The
  // core fires its own onChange (→ tick); bridge that to the consumer's
  // onChange so range() consumers (infinite scroll) get a notification.
  if (options.onChange) {
    const userOnChange = options.onChange
    effect(() => {
      tick()
      untrack(() => userOnChange(api))
    })
  }

  return api
}

/** Initial viewport size used until layout measures the scroll element. A
 *  scroll container that has not yet been measured (offset size 0 — pre-layout,
 *  or a headless environment with no layout engine) still windows a meaningful
 *  initial slice rather than zero rows; the real measured size supersedes it the
 *  moment layout or a ResizeObserver reports it. */
const UNMEASURED_VIEWPORT = 400

/** Local `observeElementRect`: reads `offsetWidth`/`offsetHeight` (the size the
 *  view sets on the scroll element) synchronously on attach, then keeps it
 *  current via a ResizeObserver when available. Mirrors the vendored core's
 *  default but binds to the thunked element identity the adapter already owns,
 *  so the very first synchronous `virtualItems()` read has a real viewport size
 *  even before any layout/scroll event fires. A zero measurement (pre-layout)
 *  falls back to `UNMEASURED_VIEWPORT` so the initial window is non-empty. */
function observeElementRect<T extends Element>(
  instance: Virtualizer<T, Element>,
  cb: (rect: { width: number; height: number }) => void,
): (() => void) | undefined {
  const element = instance.scrollElement as (T & HTMLElement) | null
  if (!element) return
  const measure = (): void =>
    cb({
      width: element.offsetWidth || UNMEASURED_VIEWPORT,
      height: element.offsetHeight || UNMEASURED_VIEWPORT,
    })
  measure()
  const win = instance.targetWindow
  if (!win || !win.ResizeObserver) return () => {}
  const ro = new win.ResizeObserver(() => measure())
  ro.observe(element, { box: 'border-box' })
  return () => ro.unobserve(element)
}
