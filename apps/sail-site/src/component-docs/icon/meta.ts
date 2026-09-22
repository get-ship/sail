import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'icon',
  tag: 'icon',
  key: 'icon',
  category: 'display',
  kind: 'host',
  blurb: "Glyph box on the icon-size ladder. Pack ids are compiler-inlined (never CDN). Empty label is decorative (aria-hidden); set label for role=img. Ink is currentColor + glyphFill (no tone prop).",
  heading: "prose",
  host: 'doc-icon',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
