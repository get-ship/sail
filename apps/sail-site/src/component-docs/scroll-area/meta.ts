import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'scroll-area',
  tag: 'scroll-area',
  key: 'scrollArea',
  category: 'layout',
  kind: 'host',
  blurb:
    'Overflow box with thin, themed scrollbars. Only orientation is a host prop (vertical · horizontal · both; default vertical). Constrain height/width with Style so content overflows. Thumb follows the theme; hover uses border-strong. Standard scrollbar-width/color only (no WebKit pseudo setters yet).',
  host: 'doc-scroll-area',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
