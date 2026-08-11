import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'input',
  tag: 'input',
  key: 'input',
  category: 'inputs',
  kind: 'host',
  blurb:
    'Native <input> with type-gated Style skins — text-like fields share component.field.* with select/textarea; checkbox/radio get checkSized/radioSized (no separate <checkbox> tag); range/color/file/hidden have dedicated arms. size xs|sm|md|lg; invalid → destructive border; disabled native + PE dim. No tone, no filled/unstyled variant. bind:value / bind:checked are language directives.',
  heading: 'prose',
  host: 'doc-input',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
