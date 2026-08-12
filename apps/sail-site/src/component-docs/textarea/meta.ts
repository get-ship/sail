import type { DocMeta } from '../types'

export const meta = {
  slug: 'textarea',
  tag: 'textarea',
  key: 'textarea',
  category: 'inputs',
  kind: 'host',
  blurb: 'Multi-line text. Same field look as input and select.',
  heading: '<textarea>',
  host: 'doc-textarea',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
