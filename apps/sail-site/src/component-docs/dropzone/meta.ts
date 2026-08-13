import type { DocMeta } from '../types'

export const meta = {
  slug: 'dropzone',
  tag: 'dropzone',
  key: 'dropzone',
  category: 'inputs',
  kind: 'host',
  blurb: 'Dashed drop surface. Drag files onto it, or click to browse.',
  heading: '<dropzone>',
  host: 'doc-dropzone',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
