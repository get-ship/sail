import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'fieldset',
  tag: 'fieldset',
  key: 'fieldset',
  category: 'inputs',
  kind: 'host',
  blurb: "Native form group with a themeable border (component.fieldset.border). Put a <legend> first (caption ink from component.legend.fg — not a separate hub tile), then controls. disabled freezes every control inside (platform) and dims the host; no tone/size/variant axes.",
  heading: "prose",
  host: 'doc-fieldset',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
