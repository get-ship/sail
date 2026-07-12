// `use:targetCursor` — the pointer becomes a viewfinder: four corner
// brackets trail the cursor and spring open to FRAME whatever interactive
// element it is over, then collapse back to a crosshair on leave.
//
// The brackets are DOM the action owns, appended to <body> so they ride
// above everything; the host element only scopes the effect's lifetime.
// Fine pointers only (touch keeps its native behavior); reduced motion
// snaps instead of trailing. All ink is var(--primary) — the cursor
// re-skins with the theme like everything else.

import type { Action } from '@ship/sui-runtime'

const TARGETS = 'a, button, [role="tab"], input, summary, select, textarea, [data-cursor]'
const IDLE_HALF = 14
const PAD = 8

export const targetCursor: Action<void> = (node) => {
  const host = node as HTMLElement
  if (!matchMedia('(hover: hover) and (pointer: fine)').matches) {
    return { destroy: () => {} }
  }
  const snap = matchMedia('(prefers-reduced-motion: reduce)').matches

  const layer = document.createElement('div')
  layer.setAttribute('aria-hidden', 'true')
  Object.assign(layer.style, {
    position: 'fixed',
    inset: '0',
    pointerEvents: 'none',
    zIndex: '2147483000',
  })
  const corner = (rot: number) => {
    const c = document.createElement('div')
    Object.assign(c.style, {
      position: 'fixed',
      width: '12px',
      height: '12px',
      borderTop: '2px solid var(--primary)',
      borderLeft: '2px solid var(--primary)',
      transform: `rotate(${rot}deg)`,
      left: '0',
      top: '0',
      willChange: 'translate',
    })
    layer.appendChild(c)
    return c
  }
  // top-left, top-right, bottom-right, bottom-left
  const corners = [corner(0), corner(90), corner(180), corner(270)]
  const dot = document.createElement('div')
  Object.assign(dot.style, {
    position: 'fixed',
    width: '4px',
    height: '4px',
    borderRadius: '99px',
    background: 'var(--primary)',
    left: '0',
    top: '0',
  })
  layer.appendChild(dot)
  document.body.appendChild(layer)
  // Scoped to the HOST: the viewfinder exists only while the pointer is
  // inside it; everywhere else keeps the native cursor. A full-site custom
  // cursor is a costume — scoped, it reads as an instrument.
  const prevCursor = host.style.cursor
  host.style.cursor = 'none'
  layer.style.opacity = '0'
  layer.style.transition = 'opacity 200ms ease'

  let mx = innerWidth / 2
  let my = innerHeight / 2
  let target: Element | null = null
  // current corner positions (x,y per corner), lerped toward goals
  const cur = [0, 1, 2, 3].map(() => ({ x: mx, y: my }))

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
    layer.style.opacity = inside ? '1' : '0'
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
    layer.style.opacity = '1'
  }
  const onHostLeave = () => {
    layer.style.opacity = '0'
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
      host.style.cursor = prevCursor
      layer.remove()
    },
  }
}
