import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'separator',
  tag: 'separator',
  key: 'separator',
  category: 'display',
  kind: 'host',
  blurb: "Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. tone defaults to muted. No separate `hr` catalog entry.",
  heading: "prose",
  host: 'doc-separator',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
