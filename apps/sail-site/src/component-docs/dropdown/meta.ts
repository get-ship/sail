import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'dropdown',
  tag: 'dropdown',
  key: 'dropdown',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Popover with menu semantics: role=menu, roving arrows, type-ahead, <menu-item> children. placement string; onSelect event. Paint component.dropdown.*.',
  heading: 'dropdown',
  host: 'doc-dropdown',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
