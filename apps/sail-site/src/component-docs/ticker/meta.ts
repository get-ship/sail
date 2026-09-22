import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'ticker',
  tag: 'ticker',
  key: 'ticker',
  category: 'effects',
  kind: 'host',
  blurb:
    'A looping band of text or icons. The loop is compile-time anatomy; reduced motion settles automatically.',
  host: 'doc-ticker',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
