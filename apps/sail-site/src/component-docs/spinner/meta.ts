import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'spinner',
  tag: 'spinner',
  key: 'spinner',
  category: 'feedback',
  kind: 'host',
  blurb: "Indeterminate wait ring. Size scales the glyph; tone defaults to info. Center it with the parent layout.",
  heading: "prose",
  host: 'doc-spinner',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
