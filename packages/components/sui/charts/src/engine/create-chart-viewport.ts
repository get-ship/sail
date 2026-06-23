// createChartViewport — responsive geometry adapter (Backbone #2, Slice 4).
//
// One shared `ResizeObserver` wrapper: turns the host's measured pixel box into
// reactive `width()`/`height()` signals so charts stop hardcoding the viewBox
// and `preserveAspectRatio="none"` (which stretches strokes). One observer
// implementation, every chart a consumer.
//
// PURE `@ship/sui-runtime` (signal) + the DOM `ResizeObserver` platform API,
// created in `observe()` and torn down in `dispose()`. The single write site is
// the observer callback, invoked from the controller's `mount` scope.

import { signal, type Signal } from '@ship/sui-runtime'

export interface ChartViewport {
  /** Measured host content-box width, reactive. */
  width: Signal<number>
  /** Measured host content-box height, reactive. */
  height: Signal<number>
  /** Wire a `ResizeObserver` to `el`; call from `mount` with the host/svg ref. */
  observe(el: Element): void
  /** Disconnect the observer. Registered to the controller's mount cleanup. */
  dispose(): void
}

export interface ChartViewportOptions {
  /** Seed width from the `width` prop estimate. */
  initialWidth?: number
  /** Seed height from the `height` prop estimate. */
  initialHeight?: number
  /** Optional aspect lock: `height = width / aspect` on each measured resize. */
  aspect?: number
}

export function createChartViewport(opts: ChartViewportOptions = {}): ChartViewport {
  const width = signal(opts.initialWidth ?? 300)
  const height = signal(opts.initialHeight ?? 200)
  const aspect = opts.aspect

  let observer: ResizeObserver | null = null

  function observe(el: Element): void {
    // Re-observing replaces a prior connection (idempotent from mount).
    if (observer) observer.disconnect()
    observer = new ResizeObserver((entries) => {
      const box = entries[entries.length - 1]?.contentRect
      if (!box) return
      const w = box.width
      if (w > 0) width.set(w)
      if (aspect && w > 0) {
        height.set(w / aspect)
      } else if (box.height > 0) {
        height.set(box.height)
      }
    })
    observer.observe(el)
  }

  function dispose(): void {
    observer?.disconnect()
    observer = null
  }

  return { width, height, observe, dispose }
}
