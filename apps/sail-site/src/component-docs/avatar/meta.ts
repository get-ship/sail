import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'avatar',
  tag: 'avatar',
  key: 'avatar',
  category: 'display',
  kind: 'host',
  blurb: "An identity frame: image, icon slug, or initials — and the broken-image path falls back to initials automatically, via one shared action.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
