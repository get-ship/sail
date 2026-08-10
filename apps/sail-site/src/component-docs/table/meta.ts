import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'table',
  tag: 'table',
  key: 'table',
  category: 'data',
  kind: 'host',
  blurb: "Not components — keywords. The preprocessor lowers them to a div plus exactly the rules used, and they are the one family that accepts dynamic values.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
