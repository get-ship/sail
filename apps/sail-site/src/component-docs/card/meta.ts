import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'card',
  tag: 'card',
  key: 'card',
  category: 'display',
  kind: 'host',
  blurb: "The surface everything else sits on. Padding, tone, an accent bar and the hover lift are attributes — the card never needs a stylesheet.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
