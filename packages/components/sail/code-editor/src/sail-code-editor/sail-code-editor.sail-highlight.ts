/**
 * SAIL syntax overlay for CodeMirror 6.
 *
 * Paints `.sail` from `@ship/sail-highlight` (same query VS Code / Helix use).
 * `lang-html` can stay underneath for structure; SAIL-specific color
 * comes from tree-sitter spans, not prefix/TS-keyword scanners.
 *
 * Theme classes map onto `--syntax-*` so dark / light / theme overrides
 * carry through automatically.
 */

import { EditorView, ViewPlugin, Decoration } from '@codemirror/view'
import type { DecorationSet, ViewUpdate, PluginValue } from '@codemirror/view'
import {
  ensureHighlight,
  highlight,
  SCOPE_TO_CM_CLASS,
} from '@ship/sail-highlight'

let boot: Promise<void> | null = null

/**
 * Load the shared highlighter once. Safe to call from the language loader
 * and from the ViewPlugin; subsequent calls share the same promise.
 */
export async function ensureSailHighlight(): Promise<void> {
  if (!boot) {
    boot = ensureHighlight().catch((err: unknown) => {
      boot = null
      throw err
    })
  }
  await boot
}

function decorationsFor(doc: string): DecorationSet {
  let spans
  try {
    spans = highlight(doc)
  } catch {
    return Decoration.none
  }
  const ranges = []
  for (const span of spans) {
    if (span.start >= span.end) continue
    const cls = SCOPE_TO_CM_CLASS[span.scope]
    if (!cls) continue
    ranges.push(Decoration.mark({ class: cls }).range(span.start, span.end))
  }
  return Decoration.set(ranges, true)
}

class SailHighlighter implements PluginValue {
  decorations: DecorationSet

  constructor(view: EditorView) {
    this.decorations = decorationsFor(view.state.doc.toString())
    void ensureSailHighlight()
      .then(() => {
        this.decorations = decorationsFor(view.state.doc.toString())
        view.dispatch({})
      })
      .catch(() => {
        /* jsdom / missing wasm — lang-html remains */
      })
  }

  update(update: ViewUpdate): void {
    if (update.docChanged) {
      this.decorations = decorationsFor(update.state.doc.toString())
    }
  }
}

const sailHighlightPlugin = ViewPlugin.fromClass(SailHighlighter, {
  decorations: (v) => v.decorations,
})

/**
 * Color rules for tree-sitter token classes (`SCOPE_TO_CM_CLASS`).
 * They sit on top of `lang-html`'s HighlightStyle so directive / style()
 * / interpolation paints win over generic attribute coloring.
 */
const sailHighlightTheme = EditorView.theme({
  '& .sail-tag': {
    color: 'var(--syntax-tag)',
  },
  '& .sail-string': {
    color: 'var(--syntax-string)',
  },
  '& .sail-kw': {
    color: 'var(--syntax-keyword)',
    fontWeight: '500',
  },
  '& .sail-interp': {
    color: 'var(--syntax-variable)',
  },
  '& .sail-interp-brace': {
    color: 'var(--syntax-punctuation, var(--syntax-operator, var(--muted-foreground)))',
    fontWeight: '600',
  },
  '& .sail-attr-prefix': {
    color: 'var(--syntax-keyword)',
    fontWeight: '600',
  },
  '& .sail-attr-name': {
    color: 'var(--syntax-attr)',
  },
  '& .sail-attr-modifier': {
    color: 'var(--syntax-number)',
  },
  '& .sail-ts-kw': {
    color: 'var(--syntax-keyword)',
    fontWeight: '500',
  },
  '& .sail-ts-str': {
    color: 'var(--syntax-string)',
  },
  '& .sail-ts-num': {
    color: 'var(--syntax-number)',
  },
  '& .sail-ts-cmt': {
    color: 'var(--syntax-comment)',
    fontStyle: 'italic',
  },
  '& .sail-ts-fn': {
    color: 'var(--syntax-function)',
  },
})

/**
 * The SAIL language extension: tree-sitter decorations on top of `lang-html`.
 */
export const sailHighlight = [sailHighlightPlugin, sailHighlightTheme]
