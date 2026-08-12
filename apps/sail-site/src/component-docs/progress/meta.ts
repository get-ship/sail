import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'progress',
  tag: 'progress',
  key: 'progressBar',
  category: 'feedback',
  kind: 'host',
  blurb: "Native progress bar, themed. value/max are the platform model; omit value for the indeterminate sweep (no separate prop). size is sm|md|lg track height. Paint always from component.progress.track + fill — no tone axis.",
  heading: "prose",
  host: 'doc-progress',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
