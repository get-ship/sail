import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'breadcrumb',
  tag: 'breadcrumb',
  key: 'breadcrumb',
  category: 'navigation',
  kind: 'host',
  blurb: "A trail of links with the right nav/ARIA and themed separators. Anchors for the path, something inert for where you are.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
