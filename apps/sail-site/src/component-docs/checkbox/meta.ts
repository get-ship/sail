import type { DocMeta } from '../types'

export const meta = {
  slug: 'checkbox',
  tag: 'checkbox',
  key: 'checkbox',
  category: 'inputs',
  kind: 'host',
  blurb: 'A boolean check. Pair with a label. Use switch for on/off settings.',
  heading: '<checkbox>',
  host: 'doc-checkbox',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
