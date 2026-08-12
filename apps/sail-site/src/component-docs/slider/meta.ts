import type { DocMeta } from '../types'

export const meta = {
  slug: 'slider',
  tag: 'slider',
  key: 'slider',
  category: 'inputs',
  kind: 'host',
  blurb: 'A single-thumb range control. Set min, max, and step as needed.',
  heading: '<slider>',
  host: 'doc-slider',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
