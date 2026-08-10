import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'tooltip',
  tag: 'tooltip',
  key: 'tooltip',
  category: 'overlay',
  kind: 'host',
  blurb: "A label on hover and focus, from the content attribute — the bubble is filled by the action, positioned by placement, and never steals focus.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
