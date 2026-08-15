import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'empty-state',
  tag: 'empty-state',
  key: 'emptyState',
  category: 'feedback',
  kind: 'host',
  blurb:
    'Nothing-here surface: icon + title + body + action slots. Chrome via variant (bordered default, dashed inline, bare placeholder/connecting). tone defaults to muted and paints the icon slot. connecting upgrades to role=status / aria-live=polite via use:variantLive.',
  heading: 'prose',
  host: 'doc-empty-state',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
