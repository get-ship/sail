import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'theme-picker',
  tag: 'theme-picker',
  key: 'themePicker',
  category: 'inputs',
  kind: 'host',
  blurb:
    'Theme switcher: trigger opens a native [popover] of theme cards previewing real stylesheets. Host props: value · inline · modeOnly · onThemeChange. Paint component.theme-picker.* (panel · trigger · option).',
  heading: 'theme-picker',
  host: 'doc-theme-picker',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
