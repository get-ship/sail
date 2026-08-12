import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'avatar',
  tag: 'avatar',
  key: 'avatar',
  category: 'display',
  kind: 'host',
  blurb:
    'Identity frame: image, icon-slug, or initials — broken-image falls back to initials. size xs–xl, shape round|square, optional tone. Faces component.avatar.bg / fg.',
  heading: 'avatar',
  host: 'doc-avatar',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
