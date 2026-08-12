import type { DocMeta } from '../types'

export const meta = {
  slug: 'combobox',
  tag: 'combobox',
  key: 'combobox',
  category: 'inputs',
  kind: 'host',
  blurb: 'Type to filter a list. Arrows move, Enter commits, Escape closes.',
  heading: '<combobox>',
  host: 'doc-combobox',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
