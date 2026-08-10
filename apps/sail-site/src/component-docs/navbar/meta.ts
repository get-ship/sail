import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'navbar',
  tag: 'navbar',
  key: 'navbar',
  category: 'navigation',
  kind: 'host',
  blurb: "One element, three placements — top bar, side rail, bottom bar. Four regions (brand, links, center, actions); bordered and sticky follow the placement.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
