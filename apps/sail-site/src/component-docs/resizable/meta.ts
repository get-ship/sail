import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'resizable',
  tag: 'resizable',
  key: 'resizable',
  category: 'layout',
  kind: 'host',
  blurb:
    'Two-pane split: start and end slots with a drag handle between. orientation horizontal (default) or vertical. Drag — or arrow-nudge — the separator to size the start pane. Handle is a focusable role=separator.',
  heading: 'resizable',
  host: 'doc-resizable',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
