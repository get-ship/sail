import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'toggle-group',
  tag: 'toggle-group',
  key: 'toggleGroup',
  category: 'inputs',
  kind: 'host',
  blurb:
    'A row of toggles with one value (or many, with type=multiple). Roving focus, aria-pressed and the segmented look are all in the element. Size is authored on each <toggle>. Segmented control host — no standalone toggle hub tile.',
  heading: 'prose',
  host: 'doc-toggle-group',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
