import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'dialog',
  tag: 'dialog',
  key: 'dialog',
  category: 'overlay',
  kind: 'host',
  blurb:
    'Platform dialog — top layer, focus trap, Escape, backdrop. size · backdrop · side · dismissible; onClose / onDismiss. Scaffold shows open anatomy; invoker open/close is native.',
  heading: 'dialog',
  host: 'doc-dialog',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
