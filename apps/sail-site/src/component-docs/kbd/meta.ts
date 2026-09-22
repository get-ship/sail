import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'kbd',
  tag: 'kbd',
  key: 'kbd',
  category: 'display',
  kind: 'host',
  blurb:
    'Native <kbd> keycap — mono type, muted surface, doubled bottom edge. Paints muted / foreground / border; no size/variant/tone props (theme the roles, compose chords in markup).',
  heading: 'prose',
  host: 'doc-kbd',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
