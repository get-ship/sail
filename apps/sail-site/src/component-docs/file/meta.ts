import type { DocMeta } from '../types'

export const meta = {
  slug: 'file',
  tag: 'file',
  key: 'file',
  category: 'inputs',
  kind: 'host',
  blurb: 'System file picker. variant="drop" is the drag-and-drop surface.',
  heading: '<file>',
  host: 'doc-file',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
