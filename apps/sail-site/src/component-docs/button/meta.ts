import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'button',
  tag: 'button',
  key: 'button',
  category: 'inputs',
  kind: 'host',
  blurb: 'Buttons for what people do next — save, cancel, confirm, open.',
  heading: "prose",
  host: 'doc-button',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
