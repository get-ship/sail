import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'field',
  tag: 'field',
  key: 'field',
  category: 'inputs',
  kind: 'host',
  blurb: "The label-plus-control unit. It wires label/for, spaces the pair, and carries the invalid state so the message and the border agree.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
