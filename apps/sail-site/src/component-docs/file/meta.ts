import type { DocMeta } from '../types'

export const meta = {
  slug: 'file',
  tag: 'file',
  key: 'file',
  category: 'inputs',
  kind: 'host',
  blurb: 'Opens the system file picker. Use dropzone for drag-and-drop.',
  heading: '<file>',
  host: 'doc-file',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
