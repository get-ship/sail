import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'combobox',
  tag: 'combobox',
  key: 'combobox',
  category: 'inputs',
  kind: 'host',
  blurb:
    'A text input that filters a list. Type to filter, arrows rove, Enter commits, Escape closes — ARIA combobox wired for you. Host props: placeholder · value · onSelect. Listbox paint component.combobox.bg / fg.',
  heading: 'combobox',
  host: 'doc-combobox',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
