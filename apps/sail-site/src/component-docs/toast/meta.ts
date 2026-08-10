import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'toast',
  tag: 'toast',
  key: 'toast',
  category: 'feedback',
  kind: 'host',
  blurb: "A transient announcement. It auto-dismisses after duration ms (0 = sticky), pauses while hovered or focused, and lives in a <toast-region> stack. Dismiss removes the instance — re-push from your controller to show again.",
  heading: "prose",
  host: 'legacy',
  stageOwnsKnobs: false,
} as const satisfies DocMeta
