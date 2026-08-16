import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'drawer',
  tag: 'drawer',
  key: 'drawer',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Edge-anchored non-modal sheet. side start|end|top|bottom (default end); size sm|md|lg. Native <dialog> via command="show" (not showModal). Header/section/footer as light-dom children. Swipe toward the edge closes. Paint card / card-foreground.',
  heading: 'drawer',
  host: 'doc-drawer',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
