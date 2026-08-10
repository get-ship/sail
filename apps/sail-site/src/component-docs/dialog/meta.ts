import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'dialog',
  tag: 'dialog',
  key: 'dialog',
  category: 'overlay',
  kind: 'host',
  blurb: "The platform dialog — top layer, focus trap, Escape, backdrop — opened and closed by invoker commands. Zero JavaScript; side= docks it to an edge as a sheet.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
