import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'popover',
  tag: 'popover',
  key: 'popover',
  category: 'overlay',
  kind: 'host',
  blurb: "The native [popover], positioned. Top layer, Escape and light-dismiss come from the platform; use:anchor adds placement — that is the whole element.",
  heading: "row · column · grid · center",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
