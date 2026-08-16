import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'context-menu',
  tag: 'context-menu',
  key: 'contextMenu',
  category: 'overlay',
  kind: 'host',
  blurb:
    "Dropdown's right-click sibling: menu semantics, opened at the pointer on contextmenu of the target that for= names. onSelect event. Paints the popover role.",
  heading: 'context-menu',
  host: 'doc-context-menu',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
