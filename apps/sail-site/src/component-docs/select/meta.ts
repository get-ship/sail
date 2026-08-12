import type { DocMeta } from '../types'

export const meta = {
  slug: 'select',
  tag: 'select',
  key: 'select',
  category: 'inputs',
  kind: 'host',
  blurb: 'Closed list picker. Keyboard, form submit, and mobile sheets come from the platform.',
  heading: '<select>',
  host: 'doc-select',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
