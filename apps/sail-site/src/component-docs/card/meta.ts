import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'card',
  tag: 'card',
  key: 'card',
  category: 'display',
  kind: 'host',
  blurb:
    'The surface everything else sits on. variant default|outline, padding, tone, interactive, accent-bar, status-rail, fill — no soft/elevated/ghost invent. Rest from component.card.*.',
  heading: 'card',
  host: 'doc-card',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
