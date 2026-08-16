import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'skeleton',
  tag: 'skeleton',
  key: 'skeleton',
  category: 'feedback',
  kind: 'host',
  blurb: "Loading placeholder sheen. Shape + size draw the geometry; the shimmer mixes muted with muted-foreground so it reads on light and dark. No tone axis — compose width with layout Style.",
  heading: "prose",
  host: 'doc-skeleton',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
