// `use:dotGrid` — an interactive dot-matrix background: a grid of faint
// token-colored dots that brighten and swell toward the pointer with a
// smooth falloff. Canvas-drawn (one layer, no per-dot DOM), colors resolved
// from the theme and re-resolved on `ship:theme:change`, so the field
// re-inks itself when the site repaints.
//
// Reduced motion / coarse pointers get the static field — presence without
// tracking. The canvas sits behind the host's content; pointer-events stay
// with the page.

import type { Action } from '@ship/sui-runtime'

const SPACING = 26
const BASE_R = 1.4
const HOT_R = 3.2
const RADIUS = 170

export const dotGrid: Action<void> = (node) => {
  const host = node as HTMLElement
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative'
  if (!host.style.zIndex) host.style.zIndex = '0'

  const canvas = document.createElement('canvas')
  canvas.setAttribute('aria-hidden', 'true')
  Object.assign(canvas.style, {
    position: 'absolute',
    inset: '0',
    width: '100%',
    height: '100%',
    zIndex: '-1',
    pointerEvents: 'none',
  })
  host.prepend(canvas)
  const ctx = canvas.getContext('2d')
  if (!ctx) return { destroy: () => canvas.remove() }

  let base = 'oklch(0.5 0 0 / 0.25)'
  let hot = 'oklch(0.6 0.2 30)'
  const readInk = () => {
    const cs = getComputedStyle(host)
    const border = cs.getPropertyValue('--border-strong').trim()
    const primary = cs.getPropertyValue('--primary').trim()
    if (border) base = border
    if (primary) hot = primary
  }
  readInk()
  document.addEventListener('ship:theme:change', readInk)

  let w = 0
  let h = 0
  const dpr = Math.min(devicePixelRatio || 1, 2)
  const size = () => {
    const r = host.getBoundingClientRect()
    w = r.width
    h = r.height
    canvas.width = w * dpr
    canvas.height = h * dpr
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  }
  const ro = new ResizeObserver(() => {
    size()
    draw()
  })
  ro.observe(host)

  // pointer in host coords; far offscreen = idle field
  let px = -9999
  let py = -9999
  const still =
    matchMedia('(prefers-reduced-motion: reduce)').matches ||
    !matchMedia('(hover: hover) and (pointer: fine)').matches

  const draw = () => {
    ctx.clearRect(0, 0, w, h)
    for (let y = SPACING / 2; y < h; y += SPACING) {
      for (let x = SPACING / 2; x < w; x += SPACING) {
        const d = Math.hypot(x - px, y - py)
        const t = still ? 0 : Math.max(0, 1 - d / RADIUS)
        const eased = t * t
        ctx.beginPath()
        ctx.arc(x, y, BASE_R + (HOT_R - BASE_R) * eased, 0, Math.PI * 2)
        ctx.fillStyle = eased > 0.02 ? hot : base
        ctx.globalAlpha = 0.35 + 0.65 * eased
        ctx.fill()
      }
    }
    ctx.globalAlpha = 1
  }
  size()
  draw()

  let raf = 0
  const onMove = (e: PointerEvent) => {
    if (still || raf) return
    raf = requestAnimationFrame(() => {
      raf = 0
      const r = host.getBoundingClientRect()
      px = e.clientX - r.left
      py = e.clientY - r.top
      draw()
    })
  }
  const onLeave = () => {
    px = -9999
    py = -9999
    draw()
  }
  // The theme AND the product accent can both change --primary after mount
  // (the shell stamps data-product per navigation); entry is the cheap,
  // always-correct moment to re-resolve ink.
  const onEnter = () => readInk()
  host.addEventListener('pointerenter', onEnter)
  host.addEventListener('pointermove', onMove, { passive: true })
  host.addEventListener('pointerleave', onLeave)

  return {
    destroy: () => {
      host.removeEventListener('pointerenter', onEnter)
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerleave', onLeave)
      document.removeEventListener('ship:theme:change', readInk)
      if (raf) cancelAnimationFrame(raf)
      ro.disconnect()
      canvas.remove()
    },
  }
}
