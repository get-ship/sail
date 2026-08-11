import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'error-boundary',
  tag: 'error-boundary',
  key: 'errorBoundary',
  category: 'feedback',
  kind: 'host',
  blurb:
    'Render-fault isolation. A throw inside the subtree swaps in the fallback (built-in or slot="fallback"); Retry — or el.reset() — brings the content back.',
  heading: 'prose',
  host: 'doc-error-boundary',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
