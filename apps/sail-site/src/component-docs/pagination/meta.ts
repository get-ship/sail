import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'pagination',
  tag: 'pagination',
  key: 'pagination',
  category: 'navigation',
  kind: 'host',
  blurb:
    'A nav of <page-item> controls. value marks the current one with aria-current; clicking a page-item calls onPageChange — your list, your fetch. Size is authored on each <page-item>.',
  host: 'doc-pagination',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
