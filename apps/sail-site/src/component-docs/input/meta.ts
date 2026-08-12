import type { DocMeta } from '../types'

export const meta = {
  slug: 'input',
  tag: 'input',
  key: 'input',
  category: 'inputs',
  kind: 'host',
  blurb: 'Single-line text fields. Passwords show a reveal control by default.',
  heading: '<input>',
  host: 'doc-input',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
