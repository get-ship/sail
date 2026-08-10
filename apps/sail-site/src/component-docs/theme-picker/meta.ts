import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'theme-picker',
  tag: 'theme-picker',
  key: 'themePicker',
  category: 'inputs',
  kind: 'host',
  blurb: "The theme switcher: a trigger that opens a native [popover] panel of theme cards, each previewing the surface it selects with the real stylesheet.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
