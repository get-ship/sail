import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'dropdown',
  tag: 'dropdown',
  key: 'dropdown',
  category: 'overlay',
  kind: 'host',
  blurb: "A popover with menu semantics: role=menu, roving arrows, type-ahead, and <menu-item> children that close it on select.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
