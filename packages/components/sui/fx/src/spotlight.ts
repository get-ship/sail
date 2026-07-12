// `use:spotlight` — a token-lit glow that follows the pointer across the
// decorated element. The effect is INPUT FEEDBACK, not autonomous motion:
// it moves only when the pointer does, so prefers-reduced-motion keeps it
// (the soft transition is dropped there). Touch/keyboard sessions get a
// static ambient glow instead of nothing.
//
// Everything is set from JS on an overlay the action owns — the pack ships
// no stylesheet and the host element's own styling is untouched beyond
// position (which a spotlight needs to anchor its overlay).

import type { Action } from '@ship/sui-runtime'

export const spotlight: Action<void> = (node) => {
  const host = node as HTMLElement
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative'

  const glow = document.createElement('div')
  glow.setAttribute('aria-hidden', 'true')
  const paint = (x: string, y: string, strength: number) => {
    glow.style.background =
      `radial-gradient(38rem circle at ${x} ${y}, ` +
      `color-mix(in oklch, var(--primary) ${strength}%, transparent), transparent 65%)`
  }
  Object.assign(glow.style, {
    position: 'absolute',
    inset: '0',
    pointerEvents: 'none',
    zIndex: '0',
    opacity: '0',
    transition: matchMedia('(prefers-reduced-motion: reduce)').matches
      ? 'none'
      : 'opacity 400ms ease',
  })
  paint('50%', '40%', 10)
  host.prepend(glow)

  const finePointer = matchMedia('(hover: hover) and (pointer: fine)').matches
  if (!finePointer) {
    // Touch: a still, centered ambient glow — presence without tracking.
    glow.style.opacity = '1'
    return { destroy: () => glow.remove() }
  }

  let raf = 0
  const onMove = (e: PointerEvent) => {
    if (raf) return
    raf = requestAnimationFrame(() => {
      raf = 0
      const r = host.getBoundingClientRect()
      paint(`${e.clientX - r.left}px`, `${e.clientY - r.top}px`, 14)
      glow.style.opacity = '1'
    })
  }
  const onLeave = () => {
    glow.style.opacity = '0'
  }
  host.addEventListener('pointermove', onMove)
  host.addEventListener('pointerleave', onLeave)

  return {
    destroy: () => {
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerleave', onLeave)
      if (raf) cancelAnimationFrame(raf)
      glow.remove()
    },
  }
}
