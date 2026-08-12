import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'icon',
  tag: 'icon',
  key: 'icon',
  category: 'display',
  kind: 'host',
  blurb: "Glyph box on the icon-size ladder. Prefer name for pack ids (local /_deck/icons resolve — never CDN); src only same-origin SVG. Empty label is decorative (aria-hidden); set label for role=img. Ink is currentColor + glyphFill (no tone prop). spin is owned by use:icon motion.",
  heading: "prose",
  host: 'doc-icon',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
