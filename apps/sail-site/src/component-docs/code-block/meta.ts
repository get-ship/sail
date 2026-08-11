import type { DocMeta } from '../types'

/** Docs package meta — hub tile exists iff this file exists. */
export const meta = {
  slug: 'code-block',
  tag: 'code-block',
  key: 'codeBlock',
  category: 'display',
  kind: 'host',
  blurb:
    'Syntax-highlighted source block. Host is a div with use:syntaxHighlight; paint on the inner pre from component.code-block (bg · foreground) plus global border/radius. Pass code as a string prop (not children). lang defaults to sail and is an authoring attr — the highlighter classifies by content shape (markup vs code), not by inventing line numbers or a copy chrome.',
  heading: 'prose',
  host: 'doc-code-block',
  stageOwnsKnobs: true,
} as const satisfies DocMeta
