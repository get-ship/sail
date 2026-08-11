import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'tooltip',
  tag: 'tooltip',
  key: 'tooltip',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Label on hover and focus from the content attribute. placement top|bottom|left|right. Bubble paint component.tooltip.bg / fg.',
  heading: 'tooltip',
  host: 'doc-tooltip',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
