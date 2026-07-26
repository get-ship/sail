// `use:marquee` — seamless infinite track scroll for L1 <ticker>.
//
// Stamps a fx:loop translate-x animation on the attached node (the
// track). Reads `duration` / `reverse` from the nearest ticker host
// (`data-sail-host="ticker"`) so prop axes stay attribute-driven. Duration
// map matches the retired PE (fast 16s / normal 24s / slow 32s). Reduced
// motion settles the track at rest via the motion runtime (no scroll).

import type { Action } from '../action'
import { __sui_motion_attach, type MotionAttachConfig } from '../motion'

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

export const marquee: Action<void> = (node) => {
  const host = hostOf(node)
  let dispose: (() => void) | null = null

  const attach = (): void => {
    dispose?.()
    dispose = __sui_motion_attach(node, configFor(host))
  }

  const mo =
    host &&
    new MutationObserver(attach)
  mo && host && mo.observe(host, { attributes: true, attributeFilter: ['duration', 'reverse'] })
  attach()

  return {
    destroy: () => {
      mo && mo.disconnect()
      dispose?.()
      dispose = null
    },
  }
}
