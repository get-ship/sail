import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'navbar',
  tag: 'navbar',
  key: 'navbar',
  category: 'navigation',
  kind: 'host',
  blurb:
    'One element, three placements — top bar, side rail, bottom bar. sticky · bordered · autohide · offset. Regions: brand, center/links, actions. Paint from component.navbar.*.',
  heading: 'navbar',
  host: 'doc-navbar',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
