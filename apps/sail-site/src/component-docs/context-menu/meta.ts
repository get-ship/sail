import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'context-menu',
  tag: 'context-menu',
  key: 'contextMenu',
  category: 'overlay',
  kind: 'host',
  blurb: "The dropdown’s right-click sibling: same menu semantics, opened at the pointer on contextmenu of the target that for= names.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
