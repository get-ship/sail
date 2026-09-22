import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'accordion',
  tag: 'accordion',
  key: 'accordion',
  category: 'display',
  kind: 'host',
  blurb:
    'ONE disclosure element (replaces collapsible + details). open · disabled · name · multiple · onToggle. Shared name = exclusive multi-item (native details name). Paints the card role.',
  heading: 'accordion',
  host: 'doc-accordion',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
