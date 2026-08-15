import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'theme-picker',
  tag: 'theme-picker',
  key: 'themePicker',
  category: 'inputs',
  kind: 'host',
  blurb:
    'A control for picking a theme. Click to browse live previews, or put the list on the page with inline. mode-only is just Light / Dark / System.',
  heading: 'theme-picker',
  host: 'doc-theme-picker',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
