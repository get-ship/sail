import type { DocMeta } from '../types'

/**
 * Catalog alias only — no first-class CE, no contract host invent.
 * Real control is <input type="checkbox">; open /components/input for type knobs.
 */
export const meta = {
  slug: 'checkbox',
  tag: 'checkbox',
  key: 'checkbox',
  category: 'inputs',
  kind: 'alias',
  blurb:
    'Catalog alias only — there is no <checkbox> L1 tag. Use <input type="checkbox"> (checkSized skin on input). size, invalid, disabled, and bind:checked are input props. Open the input detail for type knobs → /components/input.',
  heading: 'prose',
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
