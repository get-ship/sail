import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'accordion',
  tag: 'accordion',
  key: 'accordion',
  category: 'display',
  kind: 'host',
  blurb: "ONE disclosure element (replaces collapsible + details). Trigger slot, disabled, onToggle; exclusive multi-item via shared name.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
