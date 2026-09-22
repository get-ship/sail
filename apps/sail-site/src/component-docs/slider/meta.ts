import type { DocMeta } from '../types'

export const meta = {
  slug: 'slider',
  tag: 'slider',
  key: 'slider',
  category: 'inputs',
  kind: 'host',
  blurb: 'Range control. Single thumb by default; range adds a second thumb (value-end).',
  heading: '<slider>',
  host: 'doc-slider',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
