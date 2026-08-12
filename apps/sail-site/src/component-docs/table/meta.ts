import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'table',
  tag: 'table',
  key: 'table',
  category: 'data',
  kind: 'host',
  blurb:
    "The platform's table model — thead, tbody, tr — skinned by the theme. dense · striped · hoverable · bordered presence attrs. Sticky thead and numeric columns are structural; sorting/selection are L2.",
  host: 'doc-table',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
