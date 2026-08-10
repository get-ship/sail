import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'tabs',
  tag: 'tabs',
  key: 'tabs',
  category: 'navigation',
  kind: 'host',
  blurb: "Tabs with the full keyboard suite — arrows rove, Home/End jump, panels wire their ARIA automatically. Three looks via variant on <tab-list>; size is authored on each <tab>.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
