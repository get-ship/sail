// `use:marquee` — seamless infinite track scroll for L1 <ticker>.
//
// Stamps a style.loop translate-x animation on the attached node (the
// track). Reads `duration` / `reverse` from the nearest ticker host
// (`data-sail-host="ticker"`) so prop axes stay attribute-driven. Duration
// map matches the retired host Style (fast 16s / normal 24s / slow 32s). Reduced
// motion settles the track at rest via the motion runtime (no scroll).
//
// Seamless seam: the track must be TWO identical segments so -50% translate
// lands exactly on the next copy. Compile-away `<slot>` only fills once when
// authors pass a single child, so we clone the first segment when a second
// is missing (or empty). Without that, track width ≈ host width and the
// loop appears frozen.

import type { Action } from '../action'
import { writeCssVar } from '../bind'
import { __sail_motion_attach, type MotionAttachConfig } from '@ship/sail-runtime/motion'

const DURATION_S: Record<string, number> = {
  fast: 16,
  normal: 24,
  slow: 32,
}

function hostOf(node: Element): HTMLElement | null {
  return (
    (node.closest('[data-sail-host="ticker"]') as HTMLElement | null) ??
    (node.parentElement as HTMLElement | null)
  )
}

function configFor(host: HTMLElement | null): MotionAttachConfig {
  const key = host?.getAttribute('duration') || 'normal'
  const duration = DURATION_S[key] ?? 24
  const reverse = host?.hasAttribute('reverse') ?? false
  return {
    trigger: 'loop',
    motions: [
      {
        property: 'translate-x',
        preset: 'inline',
        kind: 'tween',
        from: 0,
        to: reverse ? '50%' : '-50%',
        duration,
        easing: 'linear',
        iterations: Infinity,
        direction: 'normal',
      },
    ],
  }
}

/** Ensure the track has two segments of equal content for a seamless loop. */
function ensureDualCopy(track: HTMLElement): void {
  const kids = Array.from(track.children) as HTMLElement[]
  if (kids.length === 0) return

  const harden = (el: HTMLElement): void => {
    // Defeat `.sail-row > * { min-width: 0 }` which collapses copies into the
    // host width so percent translate never moves past one viewport.
    writeCssVar(el, 'min-width', 'max-content')
    writeCssVar(el, 'flex-shrink', '0')
    writeCssVar(el, 'white-space', 'nowrap')
  }

  if (kids.length === 1) {
    const clone = kids[0]!.cloneNode(true) as HTMLElement
    clone.setAttribute('aria-hidden', 'true')
    clone.querySelectorAll<HTMLElement>('a, button, input, select, textarea, [tabindex]').forEach((el) => {
      el.setAttribute('tabindex', '-1')
      if (el instanceof HTMLAnchorElement) el.removeAttribute('href')
    })
    track.appendChild(clone)
    harden(kids[0]!)
    harden(clone)
    return
  }
  // Second segment empty (second <slot> never filled) — mirror the first.
  if (kids.length >= 2 && (kids[1]!.childNodes.length === 0 || !kids[1]!.textContent?.trim())) {
    kids[1]!.replaceChildren(...Array.from(kids[0]!.childNodes).map((n) => n.cloneNode(true)))
    kids[1]!.setAttribute('aria-hidden', 'true')
  }
  for (const k of Array.from(track.children) as HTMLElement[]) harden(k)
}

export const marquee: Action<void> = (node) => {
  const track = node as HTMLElement
  const host = hostOf(track)
  let dispose: (() => void) | null = null

  const attach = (): void => {
    ensureDualCopy(track)
    dispose?.()
    dispose = __sail_motion_attach(track, configFor(host))
  }

  const mo =
    host &&
    new MutationObserver(attach)
  mo && host && mo.observe(host, { attributes: true, attributeFilter: ['duration', 'reverse'] })
  // Layout may not be ready on first paint (width 0 → dead percent keyframes).
  // Re-attach after a frame so dual-copy width is real.
  attach()
  const raf =
    typeof requestAnimationFrame === 'function'
      ? requestAnimationFrame(() => attach())
      : 0

  return {
    destroy: () => {
      if (raf) cancelAnimationFrame(raf)
      mo && mo.disconnect()
      dispose?.()
      dispose = null
    },
  }
}
