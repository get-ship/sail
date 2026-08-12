import type { DocMeta } from '../types'

export const meta = {
  slug: 'date',
  tag: 'date',
  key: 'date',
  category: 'inputs',
  kind: 'host',
  blurb: 'Date and time fields. type picks precision.',
  heading: '<date>',
  host: 'doc-date',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
