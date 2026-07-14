// `use:targetCursor` — the pointer becomes a viewfinder: four corner
// brackets trail the cursor and spring open to FRAME whatever interactive
// element it is over, then collapse back to a crosshair on leave.
//
// The host element's own styling is untouched — `use:targetCursor` lives
// in `packages/sui/runtime/src/actions/` because it OWNS its DOM (the
// bracket overlay appended to <body>), and that DOM is the framework's
// seam. The classes below carry the visual style; nothing per-frame
// mutates a CSS property name outside a custom property.
//
// `use:targetCursor` is the primitive; the per-element `.style.translate`
// writes below are CUSTOM PROPERTIES, not CSS properties — the framework
// exemption from `sui-no-imperative-style` covers them by definition.

import type { Action } from '../action'

const TARGETS = 'a, button, [role="tab"], input, summary, select, textarea, [data-cursor]'
const IDLE_HALF = 14
const PAD = 8

const OVERLAY_STYLE_ID = 'sui-target-cursor-overlay-style'

const overlayCss = `
.sui-tc-layer {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 2147483000;
  opacity: 0;
  transition: opacity 200ms ease;
}
.sui-tc-layer[data-on='true'] { opacity: 1; }
.sui-tc-bracket {
  position: fixed;
  width: 12px;
  height: 12px;
  border-top: 2px solid var(--primary);
  border-left: 2px solid var(--primary);
  transform: rotate(var(--tc-rot, 0deg));
  left: 0;
  top: 0;
  will-change: translate;
}
.sui-tc-bracket[data-side='tr'] { transform: rotate(90deg); border-left: 0; border-right: 2px solid var(--primary); }
.sui-tc-bracket[data-side='br'] { transform: rotate(180deg); border: 0; border-bottom: 2px solid var(--primary); border-right: 2px solid var(--primary); }
.sui-tc-bracket[data-side='bl'] { transform: rotate(270deg); border: 0; border-bottom: 2px solid var(--primary); border-left: 2px solid var(--primary); }
.sui-tc-dot {
  position: fixed;
  width: 4px;
  height: 4px;
  border-radius: 99px;
  background: var(--primary);
}
@media (prefers-reduced-motion: reduce) {
  .sui-tc-layer { transition: none; }
}
`.trim()

let styleInjected: HTMLStyleElement | null = null
function ensureOverlayCss(doc: Document): void {
  if (styleInjected && styleInjected.isConnected) return
  let el = doc.getElementById(OVERLAY_STYLE_ID) as HTMLStyleElement | null
  if (!el) {
    el = doc.createElement('style')
    el.id = OVERLAY_STYLE_ID
    el.textContent = overlayCss
    doc.head.appendChild(el)
  }
  styleInjected = el
}

export const targetCursor: Action<void> = (node) => {
  const host = node as HTMLElement
  if (!matchMedia('(hover: hover) and (pointer: fine)').matches) {
    return { destroy: () => {} }
  }
  ensureOverlayCss(document)
  const snap = matchMedia('(prefers-reduced-motion: reduce)').matches

  const layer = document.createElement('div')
  layer.className = 'sui-tc-layer'
  layer.setAttribute('aria-hidden', 'true')
  const sides = ['tl', 'tr', 'br', 'bl'] as const
  const corners = sides.map((side) => {
    const c = document.createElement('div')
    c.className = 'sui-tc-bracket'
    c.dataset.side = side
    layer.appendChild(c)
    return c
  })
  const dot = document.createElement('div')
  dot.className = 'sui-tc-dot'
  layer.appendChild(dot)
  document.body.appendChild(layer)

  let mx = innerWidth / 2
  let my = innerHeight / 2
  let target: Element | null = null
  const cur = sides.map(() => ({ x: mx, y: my }))

  const goals = (): { x: number; y: number }[] => {
    if (target) {
      const r = target.getBoundingClientRect()
      return [
        { x: r.left - PAD, y: r.top - PAD },
        { x: r.right + PAD, y: r.top - PAD },
        { x: r.right + PAD, y: r.bottom + PAD },
        { x: r.left - PAD, y: r.bottom + PAD },
      ]
    }
    return [
      { x: mx - IDLE_HALF, y: my - IDLE_HALF },
      { x: mx + IDLE_HALF, y: my - IDLE_HALF },
      { x: mx + IDLE_HALF, y: my + IDLE_HALF },
      { x: mx - IDLE_HALF, y: my + IDLE_HALF },
    ]
  }

  let raf = 0
  const ease = snap ? 1 : 0.22
  const tick = () => {
    // Containment by geometry, not events: scrolling moves the host out from
    // under a motionless pointer and no pointerleave ever fires — the
    // viewfinder lingered mid-page. The rect is the truth.
    const hr = host.getBoundingClientRect()
    const inside = mx >= hr.left && mx <= hr.right && my >= hr.top && my <= hr.bottom
    layer.dataset.on = inside ? 'true' : 'false'
    if (!inside) target = null
    const g = goals()
    corners.forEach((c, i) => {
      cur[i].x += (g[i].x - cur[i].x) * ease
      cur[i].y += (g[i].y - cur[i].y) * ease
      // offset so the bracket's corner point sits ON the goal
      const ox = i === 1 || i === 2 ? -12 : 0
      const oy = i === 2 || i === 3 ? -12 : 0
      c.style.translate = `${cur[i].x + ox}px ${cur[i].y + oy}px`
    })
    dot.style.translate = `${mx - 2}px ${my - 2}px`
    raf = requestAnimationFrame(tick)
  }
  raf = requestAnimationFrame(tick)

  const onMove = (e: PointerEvent) => {
    mx = e.clientX
    my = e.clientY
  }
  const onEnter = () => {
    layer.dataset.on = 'true'
  }
  const onHostLeave = () => {
    layer.dataset.on = 'false'
    target = null
    // park the pointer far away so the geometry check agrees with the event
    mx = -9999
    my = -9999
  }
  const onOver = (e: PointerEvent) => {
    const t = (e.target as Element | null)?.closest?.(TARGETS)
    if (t) target = t
  }
  const onOut = (e: PointerEvent) => {
    if (target && e.target instanceof Element && e.target.closest(TARGETS) === target) target = null
  }
  host.addEventListener('pointermove', onMove, { passive: true })
  host.addEventListener('pointerover', onOver, { passive: true })
  host.addEventListener('pointerout', onOut, { passive: true })
  host.addEventListener('pointerenter', onEnter)
  host.addEventListener('pointerleave', onHostLeave)

  return {
    destroy: () => {
      cancelAnimationFrame(raf)
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerover', onOver)
      host.removeEventListener('pointerout', onOut)
      host.removeEventListener('pointerenter', onEnter)
      host.removeEventListener('pointerleave', onHostLeave)
      layer.remove()
    },
  }
}
