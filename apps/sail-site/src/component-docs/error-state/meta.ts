import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'error-state',
  tag: 'error-state',
  key: 'errorState',
  category: 'feedback',
  kind: 'host',
  blurb:
    'Recovery surface for failed loads and partial failures. Rest chrome paints the card role; icon + inline border keep brand destructive. code fills a mono chip; details slot gates Show details; action slot holds Retry. use:recoverySurface stamps role=alert, syncs code, and focuses the first action.',
  heading: 'prose',
  host: 'doc-error-state',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
