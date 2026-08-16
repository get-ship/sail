import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'ticker',
  tag: 'ticker',
  key: 'ticker',
  category: 'effects',
  kind: 'host',
  blurb:
    'A looping band of text or icons. duration and reverse set the motion. Reduced motion settles automatically.',
  host: 'doc-ticker',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
