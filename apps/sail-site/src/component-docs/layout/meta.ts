import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'layout',
  tag: 'layout',
  key: 'layout',
  category: 'layout',
  kind: 'keywords',
  blurb: "Not components — keywords. The preprocessor lowers them to a div plus exactly the rules used, and they are the one family that accepts dynamic values.",
  heading: "row · column · grid · center",
  host: 'doc-layout',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
