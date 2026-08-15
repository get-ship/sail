import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'avatar',
  tag: 'avatar',
  key: 'avatar',
  category: 'display',
  kind: 'host',
  blurb:
    'Identity frame: image, icon-slug, or initials — broken-image falls back to initials. size xs–xl, shape round|square, tone defaults to muted. Paints the muted role (or the chosen tone).',
  heading: 'avatar',
  host: 'doc-avatar',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
