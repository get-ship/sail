// `use:targetCursor` — the pointer becomes a viewfinder: four corner
// brackets trail the cursor and spring open to FRAME whatever interactive
// element it is over, then collapse back to a crosshair on leave.
//
// The host element's own styling is untouched — `use:targetCursor` lives
// in `packages/sui/runtime/src/actions/` because it OWNS its DOM (the
// bracket overlay appended to <body>), and that DOM is the framework's
// seam. The host's CSS only changes via CUSTOM PROPERTIES (the floor's
// one legal write shape) and via a data-attribute the companion sheet
// reads; the action never assigns a CSS property.
//
// `use:targetCursor` is the primitive; the per-frame writes below are
// `--tc-x` / `--tc-y` custom properties consumed by the companion
// stylesheet via `translate: var(--tc-x) var(--tc-y)`. The framework
// exemption from `sui-no-imperative-style` covers the custom-property
// write by definition; nothing here assigns a real CSS property.

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
  /* The action sets --tc-x / --tc-y per-frame as custom properties; the
     translate consumes them. The framework exemption covers the
     setProperty write — translate itself lives in the framework sheet. */
  translate: var(--tc-x, 0) var(--tc-y, 0);
  left: 0;
  top: 0;
  will-change: translate;
}
/* One mechanism: each corner is DRAWN by its two borders — no rotation.
   The old rules drew the right borders AND rotated, which un-cornered
   them (br drew ┘ then flipped it 180° into ┌ — the wrong-facing
   bracket). The never-written --tc-rot var died with the rotations. */
.sui-tc-bracket[data-side='tr'] { border-left: 0; border-right: 2px solid var(--primary); }
.sui-tc-bracket[data-side='br'] { border: 0; border-bottom: 2px solid var(--primary); border-right: 2px solid var(--primary); }
.sui-tc-bracket[data-side='bl'] { border: 0; border-bottom: 2px solid var(--primary); border-left: 2px solid var(--primary); }
.sui-tc-dot {
  position: fixed;
  width: 4px;
  height: 4px;
  border-radius: 99px;
  background: var(--primary);
  translate: var(--tc-x, 0) var(--tc-y, 0);
  /* Hide the dot until the first pointermove lands the cursor. Without
     this, the initial mx/innerWidth/2 + my/innerHeight/2 rAF tick would
     put a stray dot at the viewport centre on every page that mounts the
     action. */
  opacity: 0;
  transition: opacity 120ms ease;
}
.sui-tc-layer[data-on='true'] .sui-tc-dot { opacity: 1; }
@media (prefers-reduced-motion: reduce) {
  .sui-tc-layer { transition: none; }
  .sui-tc-dot { transition: none; }
}
`.trim()

// The host element gets `cursor: none` only WHILE the custom cursor is
// active (a data-attribute the action toggles on enter/leave). The
// consumer's own cursor declaration survives the action's lifetime
// because we never write `host.style.cursor` directly — the rule keys on
// `[data-tc-on='true']` which we set and clear. This is the only thing
// the companion sheet asserts about the host; the action never assigns
// a CSS property.
const hostCss = `
[data-tc-on='true'] { cursor: none; }
`.trim()

let styleInjected: HTMLStyleElement | null = null
function ensureOverlayCss(doc: Document): void {
  if (styleInjected && styleInjected.isConnected) return
  // The two styles share an id namespace on purpose — a single
  // companion sheet per app keeps the document's <style> count low.
  let el = doc.getElementById(OVERLAY_STYLE_ID) as HTMLStyleElement | null
  if (!el) {
    el = doc.createElement('style')
    el.id = OVERLAY_STYLE_ID
    el.textContent = overlayCss + '\n' + hostCss
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
  // dataset.on starts at 'false' so a caller querying the layer's state
  // before the first rAF tick (e.g. a unit test) sees the same answer
  // the framework's CSS does: hidden. The host's `data-tc-on` matches
  // for the same reason — both attributes drive visibility / cursor
  // suppression, so they share a synchronous initial state.
  layer.dataset.on = 'false'
  host.dataset.tcOn = 'false'
  document.body.appendChild(layer)

  // Initial mx/my are intentionally off-screen so a stray dot cannot
  // appear at the viewport centre before the first pointermove. The dot
  // is also opacity:0 until data-on='true' lands (the CSS guard above),
  // so a page that mounts the action but never sees a pointer still
  // renders nothing.
  let mx = -9999
  let my = -9999
  let seenPointer = false
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
    // Containment by geometry, not events: scrolling moves the host out
    // from under a motionless pointer and no pointerleave ever fires —
    // the viewfinder lingered mid-page. The rect is the truth.
    const hr = host.getBoundingClientRect()
    const inside = seenPointer && mx >= hr.left && mx <= hr.right && my >= hr.top && my <= hr.bottom
    layer.dataset.on = inside ? 'true' : 'false'
    host.dataset.tcOn = inside ? 'true' : 'false'
    if (!inside) target = null
    const g = goals()
    corners.forEach((c, i) => {
      // noUncheckedIndexedAccess: every indexed read here is guaranteed
      // by the parallel `cur` and `goals()` construction (sides and goal
      // arrays are both length-4 by the same consts). The non-null
      // assertions are bookkeeping, not guesses.
      const ci = cur[i]!
      const gi = g[i]!
      ci.x += (gi.x - ci.x) * ease
      ci.y += (gi.y - ci.y) * ease
      // offset so the bracket's corner point sits ON the goal
      const ox = i === 1 || i === 2 ? -12 : 0
      const oy = i === 2 || i === 3 ? -12 : 0
      c.style.setProperty('--tc-x', `${ci.x + ox}px`)
      c.style.setProperty('--tc-y', `${ci.y + oy}px`)
    })
    // The dot rides the pointer directly (no easing) — the brackets
    // lead the eye, the dot lands on the cursor.
    dot.style.setProperty('--tc-x', `${mx - 2}px`)
    dot.style.setProperty('--tc-y', `${my - 2}px`)
    raf = requestAnimationFrame(tick)
  }
  raf = requestAnimationFrame(tick)

  const onMove = (e: PointerEvent) => {
    mx = e.clientX
    my = e.clientY
    seenPointer = true
  }
  const onOver = (e: PointerEvent) => {
    const t = (e.target as Element | null)?.closest?.(TARGETS)
    if (t) target = t
  }
  const onOut = (e: PointerEvent) => {
    if (target && e.target instanceof Element && e.target.closest(TARGETS) === target) target = null
  }
  const onLeave = () => {
    // Park the pointer far away so the geometry check agrees with the
    // event and the dot/brackets collapse out of view on the next tick.
    // seenPointer stays true so a re-enter resumes immediately.
    mx = -9999
    my = -9999
    target = null
  }
  host.addEventListener('pointermove', onMove, { passive: true })
  host.addEventListener('pointerover', onOver, { passive: true })
  host.addEventListener('pointerout', onOut, { passive: true })
  host.addEventListener('pointerleave', onLeave)

  return {
    destroy: () => {
      cancelAnimationFrame(raf)
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerover', onOver)
      host.removeEventListener('pointerout', onOut)
      host.removeEventListener('pointerleave', onLeave)
      if (host.dataset.tcOn !== undefined) delete host.dataset.tcOn
      layer.remove()
    },
  }
}
