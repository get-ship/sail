import type { DocMeta } from '../types'

export const meta = {
  slug: 'radio',
  tag: 'radio',
  key: 'radio',
  category: 'inputs',
  kind: 'host',
  blurb: 'One option in a group. Prefer radio-group for exclusive choice.',
  heading: '<radio>',
  host: 'doc-radio',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
