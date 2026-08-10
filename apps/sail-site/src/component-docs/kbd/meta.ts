import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'kbd',
  tag: 'kbd',
  key: 'kbd',
  category: 'display',
  kind: 'host',
  blurb: "Native <kbd> keycap — mono type, muted surface, doubled bottom edge. Paint from component.kbd (bg · foreground · border); no size/variant/tone props (theme the leaves, compose chords in markup).",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
