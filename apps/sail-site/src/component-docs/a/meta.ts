import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'a',
  tag: 'a',
  key: 'a',
  category: 'navigation',
  kind: 'host',
  blurb:
    'Anchor — native <a>. Bare or variant=text has no chrome: wrap a card, span, or label. Button variants are default|solid|soft|outline|ghost.',
  heading: 'anchor',
  host: 'doc-a',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
