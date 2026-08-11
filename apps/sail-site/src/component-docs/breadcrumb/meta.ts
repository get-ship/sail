import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'breadcrumb',
  tag: 'breadcrumb',
  key: 'breadcrumb',
  category: 'navigation',
  kind: 'host',
  blurb:
    'Trail host only — no PE props. Compose nested <a> path crumbs and inert current (small). Faces component.breadcrumb.fg / current.',
  heading: 'breadcrumb',
  host: 'doc-breadcrumb',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
