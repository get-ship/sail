import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'tabs',
  tag: 'tabs',
  key: 'tabs',
  category: 'navigation',
  kind: 'host',
  blurb:
    'Tabs with the full keyboard suite — arrows rove, Home/End jump, panels wire their ARIA automatically. Click a tab to show its panel. orientation=vertical puts the list beside the panels. Strip look on <tab-list variant>; size Style on each <tab>.',
  heading: 'tabs',
  host: 'doc-tabs',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
