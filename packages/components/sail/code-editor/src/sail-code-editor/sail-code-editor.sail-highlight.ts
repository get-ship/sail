/**
 * SAIL syntax overlay for CodeMirror 6.
 *
 * Paints `.sail` from `@ship/sail-highlight` (tree-sitter-sail.wasm +
 * `queries/highlights.scm` — the same query VS Code / Helix use).
 * `lang-html` can stay underneath for structure; SAIL-specific color
 * comes from tree-sitter spans, not prefix/TS-keyword scanners.
 *
 * Theme classes map onto `--syntax-*` so dark / light / theme overrides
 * carry through automatically.
 */

import { EditorView, ViewPlugin, Decoration } from '@codemirror/view'
import type { DecorationSet, ViewUpdate, PluginValue } from '@codemirror/view'
import {
  initHighlight,
  highlight,
  SCOPE_TO_CM_CLASS,
} from '@ship/sail-highlight'

// Static `new URL(..., import.meta.url)` so bundlers emit the assets.
// Path: packages/components/sail/code-editor/src/sail-code-editor → packages/sail/…
const FALLBACK_WASM = new URL(
  '../../../../../sail/highlight/tree-sitter-sail.wasm',
  import.meta.url,
)
const FALLBACK_SCM = new URL(
  '../../../../../sail/grammar/queries/highlights.scm',
  import.meta.url,
)

let boot: Promise<void> | null = null

function resolveViaPackage(specifier: string): string | null {
  try {
    if (typeof import.meta.resolve === 'function') {
      const resolved = import.meta.resolve(specifier)
      if (typeof resolved === 'string') return resolved
    }
  } catch {
    /* no import map / package export */
  }
  return null
}

function highlightPackageRoot(): URL {
  const entry = resolveViaPackage('@ship/sail-highlight')
  // Package entry is `src/index.ts` — wasm + grammar sit next to / above src/.
  if (entry) return new URL('../', entry)
  return new URL('./', FALLBACK_WASM)
}

function resolveWasmUrl(): string {
  return resolveViaPackage('@ship/sail-highlight/tree-sitter-sail.wasm')
    ?? new URL('tree-sitter-sail.wasm', highlightPackageRoot()).href
}

function resolveScmUrl(): string {
  const entry = resolveViaPackage('@ship/sail-highlight')
  if (entry) return new URL('../grammar/queries/highlights.scm', highlightPackageRoot()).href
  return FALLBACK_SCM.href
}

async function readAssetText(href: string): Promise<string> {
  const res = await fetch(href)
  if (!res.ok) {
    throw new Error(`sail-highlight: failed to load ${href} (${res.status})`)
  }
  return res.text()
}

/**
 * Load wasm + highlights.scm once. Safe to call from the language loader
 * and from the ViewPlugin; subsequent calls share the same promise.
 */
export async function ensureSailHighlight(): Promise<void> {
  if (!boot) {
    boot = (async () => {
      const wasm = resolveWasmUrl()
      const highlightsScm = await readAssetText(resolveScmUrl())
      await initHighlight({ wasm, highlightsScm })
    })().catch((err: unknown) => {
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
