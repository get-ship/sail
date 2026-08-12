import type { DocMeta } from '../types'

export const meta = {
  slug: 'color',
  tag: 'color',
  key: 'color',
  category: 'inputs',
  kind: 'host',
  blurb: 'A system colour swatch for a single hex value.',
  heading: '<color>',
  host: 'doc-color',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
