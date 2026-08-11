import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'a',
  tag: 'a',
  key: 'a',
  category: 'navigation',
  kind: 'host',
  blurb:
    'Native <a>. Bare (no variant attr) stays a text link — freeform never stamps variant/tone. With variant set it becomes a button-link: default rest uses component.button.*; solid / soft / outline / ghost match the button family (default ≠ solid). tone always paints when set. No size axis — use <button> for sized CTAs. disabled paint needs aria-disabled on anchors.',
  heading: 'prose',
  host: 'doc-a',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
