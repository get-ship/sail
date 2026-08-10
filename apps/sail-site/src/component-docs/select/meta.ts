import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'select',
  tag: 'select',
  key: 'select',
  category: 'inputs',
  kind: 'host',
  blurb: "Native closed picker — keyboard, form submission, and mobile sheets come from the platform. Rest surface is component.field (bg · fg · border) shared with input/textarea; size xs|sm|md|lg; invalid → destructive border; disabled dims. SELECT_MAP draws the chevron + option ink. Not a combobox — open listbox chrome is a different L1.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
