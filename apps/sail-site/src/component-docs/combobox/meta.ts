import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'combobox',
  tag: 'combobox',
  key: 'combobox',
  category: 'inputs',
  kind: 'host',
  blurb: "A text input that filters a list. Type to filter, arrows rove, Enter commits, Escape closes — the ARIA combobox pattern wired for you, themed like everything else.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
