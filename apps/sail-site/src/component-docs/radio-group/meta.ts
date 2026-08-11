import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'radio-group',
  tag: 'radio-group',
  key: 'radioGroup',
  category: 'inputs',
  kind: 'host',
  blurb: "Groups native radios under one accessible name and one value. Host is layout + typeface only (no component.radio-group.* leaves — option paint is child radio). The platform owns exclusivity and keyboard; the group syncs ARIA and emits one change.",
  heading: "prose",
  host: 'doc-radio-group',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
