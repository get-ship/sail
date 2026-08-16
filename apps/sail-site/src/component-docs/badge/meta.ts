import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'badge',
  tag: 'badge',
  key: 'badge',
  category: 'display',
  kind: 'host',
  blurb: "A small statement of state. Soft by default; solid or outline when it must shout or whisper. tone defaults to muted.",
  heading: "prose",
  host: 'doc-badge',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
