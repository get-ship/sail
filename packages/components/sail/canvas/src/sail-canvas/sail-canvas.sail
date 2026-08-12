<!--
  <sail-canvas> — pan/zoom infinite-surface substrate. Viewport + transformed
  world layer + default slot. Engine: createCanvas() on runtime signals.
  Live engine via `canvas` identity (board/draw share one engine); `value` JSON is SSR fallback.
-->

<scope>
  props: {
    value = ''
    readonly = false
    theme: 'light' | 'dark' = 'light'
    grid: 'none' | 'dots' | 'lines' = 'none'
    'min-scale' = 0
    'max-scale' = 0
    label = ''
  }
</scope>
<controller>
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './sail-canvas.sail'
import { createCanvas } from '../engine/create-canvas'
import type { Canvas, Transform } from '../engine/types'

function parseSnapshot(raw: string): { transform?: Partial<Transform> } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { transform?: Partial<Transform> }
  } catch {
    return null
  }
}

export default controller(({ value }: Scope) => {
  const viewportEl = signal<HostBag | null>(null)
  const seedTransform = parseSnapshot(value())?.transform
  const fallback = createCanvas({
    getViewportElement: () => viewportEl(),
    ...(seedTransform ? { initialTransform: seedTransform } : {}),
  })
  // Composing face injects via canvas identity; supersedes fallback.
  const injected = signal<Canvas | null>(null)
  const canvas = derived<Canvas>(() => injected() ?? fallback)
  const transformSig = derived<Transform>(() => canvas().transform())
  const debounced = debounce(transformSig, 200)
  const worldTransformCss = derived<string>(() => {
    const t = canvas().transform()
    return `translate(${t.x}px, ${t.y}px) scale(${t.scale})`
  })

  const pointerDown = (e: DomEv): void => {
    canvas().onPointerDown(e)
  }
  const pointerMove = (e: DomEv): void => {
    canvas().onPointerMove(e)
  }
  const pointerUp = (e: DomEv): void => {
    canvas().onPointerUp(e)
  }
  const wheel = (e: DomEv): void => {
    canvas().onWheel(e)
    e.preventDefault()
  }

  mount((host) => {
    defineHostProperty(asSailHost(host) as HostBag, 'canvas', {get: () => injected(),
      set: (c: Canvas | null) => {
        if (c) c.setOptions({ getViewportElement: () => viewportEl()})
        injected.set(c)
      },
    })

    const minScale = host.getAttribute('min-scale')
    const maxScale = host.getAttribute('max-scale')
    canvas().setOptions({
      ...(minScale != null ? { minScale: Number(minScale) } : {}),
      ...(maxScale != null ? { maxScale: Number(maxScale) } : {}),
    })

    const stopEmit = effect(() => {
      const t = debounced()
      if (!t) return
      host.emit!('sail-canvas-viewport', { transform: t })
    })

    return () => {
      stopEmit()
      delete (host as { canvas?: unknown }).canvas
      fallback.dispose()
    }
  })

  const captureViewport: Action<void> = (node) => {
    viewportEl.set(node)
    return { destroy: () => viewportEl.set(null) }
  }
  return { canvas, injected, fallback, captureViewport, debounced, worldTransformCss, pointerDown, pointerMove, pointerUp, wheel }
})

</controller>
<div
  role="application"
  aria-label={label()}
  use:captureViewport
  on:pointerdown={pointerDown}
  on:pointermove={pointerMove}
  on:pointerup={pointerUp}
  on:wheel={wheel}
  style({
    box: position(relative).w(full).h(full).minH(panel-min-w-lg).overflow(hidden),
    paint: bg(background).fg(foreground).borderWidth(1).border(border).radius(md).cursor(grab),
  })
>
  <div
    use:worldTransform={worldTransformCss()}
    style({ box: position(absolute).top(0).left(0), paint: transform(bind(worldTx)) })
  >
    <slot></slot>
  </div>
</div>
