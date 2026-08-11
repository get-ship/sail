import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'label',
  tag: 'label',
  key: 'label',
  category: 'inputs',
  kind: 'host',
  blurb: "Native <label> — for= focuses the control. size sm|md|lg matches the control type ladder. required shows the * marker from component.label.required; host ink from component.label.fg and weight from typography.label.weight. No tone prop.",
  heading: "prose",
  host: 'doc-label',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
