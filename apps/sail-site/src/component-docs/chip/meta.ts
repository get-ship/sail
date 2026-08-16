import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'chip',
  tag: 'chip',
  key: 'chip',
  category: 'display',
  kind: 'host',
  blurb: "An interactive filter pill — the host is the control (role, keyboard, aria-pressed). tone defaults to primary. removable adds the ×; Backspace/Delete fires on-remove.",
  heading: "prose",
  host: 'doc-chip',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
