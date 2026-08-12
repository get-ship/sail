import type { DocMeta } from '../types'

export const meta = {
  slug: 'switch',
  tag: 'switch',
  key: 'switch',
  category: 'inputs',
  kind: 'host',
  blurb: 'On / off toggle for settings. Use checkbox for multi-select lists.',
  heading: '<switch>',
  host: 'doc-switch',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
