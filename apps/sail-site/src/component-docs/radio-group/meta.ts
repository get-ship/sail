import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'radio-group',
  tag: 'radio-group',
  key: 'radioGroup',
  category: 'inputs',
  kind: 'host',
  blurb: "Groups native radios under one accessible name and one value. The platform owns exclusivity and keyboard; the group syncs ARIA and emits one change.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
