import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'hover-card',
  tag: 'hover-card',
  key: 'hoverCard',
  category: 'overlay',
  kind: 'host',
  blurb: "A rich panel on hover or focus of its trigger — the button whose popovertarget names it. Native top layer; open/close delays so it never flickers.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
