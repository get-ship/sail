import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'button',
  tag: 'button',
  key: 'button',
  category: 'inputs',
  kind: 'host',
  blurb: "The platform button. tone paints the colour; variant is how much of it you get (solid fill, soft, outline, ghost). default is an alias of solid.",
  heading: "prose",
  host: 'doc-button',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
