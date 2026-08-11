import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'hover-card',
  tag: 'hover-card',
  key: 'hoverCard',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Rich panel on hover/focus of the trigger whose popovertarget names it. placement string. Paint component.hover-card.*.',
  heading: 'hover-card',
  host: 'doc-hover-card',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
