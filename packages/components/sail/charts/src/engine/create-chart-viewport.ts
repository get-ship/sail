// createChartViewport — responsive geometry adapter (Backbone #2, Slice 4).
// Pure product surface: observeContentBox stands in for ResizeObserver.

import { observeContentBox, signal, type Signal } from '@ship/sail-runtime'

export interface ChartViewport {
  width: Signal<number>
  height: Signal<number>
  observe(el: unknown): void
  dispose(): void
}

export interface ChartViewportOptions {
  initialWidth?: number
  initialHeight?: number
  aspect?: number
}

export function createChartViewport(opts: ChartViewportOptions = {}): ChartViewport {
  const width = signal(opts.initialWidth ?? 300)
  const height = signal(opts.initialHeight ?? 200)
  const aspect = opts.aspect
  let stop: (() => void) | null = null

  function observe(el: unknown): void {
    stop?.()
    stop = observeContentBox(el, (w, h) => {
      if (w > 0) width.set(w)
      if (aspect && w > 0) height.set(w / aspect)
      else if (h > 0) height.set(h)
    })
  }

  function dispose(): void {
    stop?.()
    stop = null
  }

  return { width, height, observe, dispose }
}
