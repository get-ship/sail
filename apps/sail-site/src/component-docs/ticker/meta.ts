import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'ticker',
  tag: 'ticker',
  key: 'ticker',
  category: 'effects',
  kind: 'host',
  blurb: "Infinite linear marquee. Host is geometry only (overflow clip); use:marquee dual-copies the slot and loops translate-x. duration · reverse on host. No component.ticker.* — author paint. Reduced motion settles via motion runtime.",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
