import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'prose',
  tag: 'prose',
  key: 'prose',
  category: 'display',
  kind: 'host',
  blurb:
    'Typography scope for rendered content — wrap markdown blobs and long copy in <prose>. Host ink is component.prose.fg; nested h1…p, lists, blockquote, and code pick up the framework type map. No size/variant/tone props (theme the leaf + type roles, not a Title API).',
  heading: 'prose',
  host: 'doc-prose',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
