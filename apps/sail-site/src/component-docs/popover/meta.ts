import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'popover',
  tag: 'popover',
  key: 'popover',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Native [popover] positioned by placement. Escape and light-dismiss from the platform; use:anchor places the panel. Paint from component.popover.*.',
  heading: 'popover',
  host: 'doc-popover',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
