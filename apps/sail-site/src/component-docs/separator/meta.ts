import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'separator',
  tag: 'separator',
  key: 'separator',
  category: 'display',
  kind: 'host',
  blurb: "Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. Omit tone for themeable component.separator.line; set muted · primary · destructive only when the line must speak. No separate `hr` catalog entry.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
