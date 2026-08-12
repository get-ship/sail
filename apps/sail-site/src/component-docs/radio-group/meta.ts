import type { DocMeta } from '../types'

export const meta = {
  slug: 'radio-group',
  tag: 'radio-group',
  key: 'radio-group',
  category: 'inputs',
  kind: 'host',
  blurb: 'A set of radios with one shared value and accessible name.',
  heading: '<radio-group>',
  host: 'doc-radio-group',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
