import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'textarea',
  tag: 'textarea',
  key: 'textarea',
  category: 'inputs',
  kind: 'host',
  blurb: "Native multi-line field on the shared component.field surface (bg · fg · border with input/select). bind:value locks the host to a signal. size xs|sm|md|lg scales min-height + type; rows sets initial height (default 3). invalid paints border(destructive); disabled is native freeze + PE opacity. No tone prop — theme component.field.*; use:autosize grows height with content.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
