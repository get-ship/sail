import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'skeleton',
  tag: 'skeleton',
  key: 'skeleton',
  category: 'feedback',
  kind: 'host',
  blurb: "Loading placeholder sheen. Shape + size draw the geometry; paint is always the shimmer from component.skeleton.bg / highlight (themeable). No tone axis — compose width with layout Style.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
