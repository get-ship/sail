import type { DocMeta } from '../types'

export const meta = {
  slug: 'field',
  tag: 'field',
  key: 'field',
  category: 'inputs',
  kind: 'host',
  blurb: 'Label plus control. Wires spacing and invalid state so the message and border agree.',
  heading: '<field>',
  host: 'doc-field',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
