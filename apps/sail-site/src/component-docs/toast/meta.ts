import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'toast',
  tag: 'toast',
  key: 'toast',
  category: 'feedback',
  kind: 'host',
  blurb:
    'Transient announcement. Auto-dismiss after duration ms (0 sticky), pauses while hovered/focused. Optional tone; onDismiss. Lives in a toast-region stack. Faces component.toast.*.',
  heading: 'toast',
  host: 'doc-toast',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
