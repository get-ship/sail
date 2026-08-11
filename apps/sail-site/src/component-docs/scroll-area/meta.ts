import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'scroll-area',
  tag: 'scroll-area',
  key: 'scrollArea',
  category: 'layout',
  kind: 'host',
  blurb:
    'Overflow box with thin, themed scrollbars. Only orientation is a PE prop (vertical · horizontal · both; default vertical). Constrain height/width with Style box so content overflows. Rest thumb from component.scroll-area.thumb; hover uses free border-strong (SPEC-D1 — thumb-hover leaf is parked unread). Standard scrollbar-width/color only (no WebKit pseudo setters yet).',
  host: 'doc-scroll-area',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
