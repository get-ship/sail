// `use:spotlight` — a token-lit glow that follows the pointer across the
// decorated element. The effect is INPUT FEEDBACK, not autonomous motion:
// it moves only when the pointer does, so prefers-reduced-motion keeps it
// (the soft transition is dropped there). Touch/keyboard sessions get a
// static ambient glow instead of nothing.
//
// The overlay's visual style is carried by a companion <style> block that
// keys on `data-sail="spotlight-glow"` and reads custom properties
// (`--spot-strength`, `--spot-x`, `--spot-y`) set from JS. The host element
// is NOT styled from JS — `position: relative` is declared by the consumer
// (`sx="relative"`).

import type { Action } from '../action'

const OVERLAY_STYLE_ID = 'sail-spotlight-overlay-style'

const overlayCss = `
[data-sail='spotlight-glow'] {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0;
  transition: opacity 400ms ease;
  background: radial-gradient(
    38rem circle at var(--spot-x, 50%) var(--spot-y, 40%),
    color-mix(in oklch, var(--primary) var(--spot-strength, 10%), transparent),
    transparent 65%
  );
}
@media (prefers-reduced-motion: reduce) {
  [data-sail='spotlight-glow'] { transition: none; }
}
[data-sail='spotlight-glow'][data-on='true'] { opacity: 1; }
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

export const spotlight: Action<void> = (node) => {
  const host = node as HTMLElement
  ensureOverlayCss(document)

  const glow = document.createElement('div')
  glow.dataset.sail = 'spotlight-glow'
  glow.setAttribute('aria-hidden', 'true')

  const finePointer = matchMedia('(hover: hover) and (pointer: fine)').matches
  if (!finePointer) {
    // Touch / keyboard: a centered ambient glow. `data-on` toggles opacity
    // through CSS; nothing per-frame is required on the visual side.
    glow.dataset.on = 'true'
    host.prepend(glow)
    return { destroy: () => glow.remove() }
  }

  host.prepend(glow)

  let raf = 0
  const onMove = (e: PointerEvent) => {
    if (raf) return
    raf = requestAnimationFrame(() => {
      raf = 0
      const r = host.getBoundingClientRect()
      glow.style.setProperty('--spot-x', `${e.clientX - r.left}px`)
      glow.style.setProperty('--spot-y', `${e.clientY - r.top}px`)
      glow.style.setProperty('--spot-strength', '14')
      glow.dataset.on = 'true'
    })
  }
  const onLeave = () => {
    glow.dataset.on = 'false'
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
