import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'alert',
  tag: 'alert',
  key: 'alert',
  category: 'feedback',
  kind: 'host',
  blurb: "Inline status banner. tone defaults to info and paints the left stripe + icon. warning/danger flip liveRegion to assertive role=alert; dismissible shows the close control (onDismiss then remove).",
  heading: "prose",
  host: 'doc-alert',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
