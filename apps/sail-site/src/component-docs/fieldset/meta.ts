import type { DocMeta } from '../types'

export const meta = {
  slug: 'fieldset',
  tag: 'fieldset',
  key: 'fieldset',
  category: 'inputs',
  kind: 'host',
  blurb: 'Groups related controls under a legend. Disabled freezes everything inside.',
  heading: '<fieldset>',
  host: 'doc-fieldset',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
