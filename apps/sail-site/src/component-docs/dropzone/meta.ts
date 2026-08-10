import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'dropzone',
  tag: 'dropzone',
  key: 'dropzone',
  category: 'inputs',
  kind: 'action',
  blurb: "An attribute, not a component: put dropzone on a surface and use:dropzone wires dragover states, click-to-browse and the file input.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
