import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'spinner',
  tag: 'spinner',
  key: 'spinner',
  category: 'feedback',
  kind: 'host',
  blurb: "Indeterminate wait ring. Size scales the host; omit tone for themeable spinner.foreground / spinner.track; centered parks it in the middle of its container.",
  heading: "prose",
  host: 'doc-spinner',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
