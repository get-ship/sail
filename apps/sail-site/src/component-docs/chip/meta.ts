import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'chip',
  tag: 'chip',
  key: 'chip',
  category: 'display',
  kind: 'host',
  blurb: "An interactive filter pill — the host is the control (role, keyboard, aria-pressed). Omit tone for component.chip rest tokens; set tone and the semantic colour always paints. removable adds the ×; Backspace/Delete fires on-remove.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
