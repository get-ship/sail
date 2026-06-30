// <sui-code-editor> — SUI primitive, framework-pure. CodeMirror 6 wrapped as a
// lazy-loaded web-component. Drop-in for any code-editing surface in sui-site
// (or any SUI app) without depending on the ship platform.
//
// bind:this + mount() pattern: ref → EditorView → event bridge → cleanup.
// Same controller shape the ship-code-editor pack uses, minus the LSP worker
// (the deck bundler doesn't support `?worker` Vite suffix). Diagnostics and
// completions can be hot-patched by the host via setDiagnostics/setCompletions
// (the canonical path used by the tutorial compile-error squiggle flow).

import { controller, effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import { EditorState, Compartment } from '@codemirror/state'
import type { Extension } from '@codemirror/state'
import {
  EditorView, keymap,
  placeholder as cmPlaceholder, lineNumbers,
  highlightActiveLine, highlightActiveLineGutter,
  drawSelection, rectangularSelection,
} from '@codemirror/view'
import {
  defaultHighlightStyle, syntaxHighlighting,
  indentOnInput, bracketMatching, foldGutter, foldKeymap,
} from '@codemirror/language'
import {
  defaultKeymap, history, historyKeymap, indentWithTab,
} from '@codemirror/commands'
import {
  closeBrackets, closeBracketsKeymap,
  autocompletion,
  type CompletionContext, type CompletionResult, type Completion,
} from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import {
  lintGutter, setDiagnostics as cmSetDiagnostics,
  type Diagnostic,
} from '@codemirror/lint'
import { suiEditorTheme, suiHighlighting } from './sui-code-editor.theme'
import { loadLanguage } from './sui-code-editor.lang'
import type { Scope } from './sui-code-editor.sui'

/** A single editor diagnostic. Lines/columns are 1-based. */
export interface SuiDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

/** A completion item source — the trigger string + the labels/inserts. */
export interface SuiCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

/** Host-facing imperative API exposed on the element instance. */
export interface SuiCodeEditorApi {
  setDiagnostics: (diags: SuiDiagnostic[]) => void
  setCompletions: (src: SuiCompletionSource[]) => void
  focus: () => void
}

const DEBOUNCE_MS = 150

export default controller<Scope>({
  mount(state, host) {
    const hostEl = (state.$signals.editorHost as unknown as Signal<HTMLElement | null>)()
    if (!hostEl) {
      console.warn('[sui-code-editor] bind:this ref is null — CodeMirror not initialized')
      return
    }

    const langCompartment = new Compartment()
    const readonlyCompartment = new Compartment()
    const lineNumCompartment = new Compartment()
    const placeholderCompartment = new Compartment()
    const completionCompartment = new Compartment()

    let view: EditorView | null = null
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    let suppressUpdate = false
    let pendingDiagnostics: SuiDiagnostic[] | null = null
    let completions: SuiCompletionSource[] = []

    const valSig = state.$signals.value as unknown as Signal<string>
    const langSig = state.$signals.language as unknown as Signal<string>
    const roSig = state.$signals.readonly as unknown as Signal<boolean>
    const lnSig = state.$signals.lineNumbers as unknown as Signal<boolean>
    const phSig = state.$signals.placeholder as unknown as Signal<string>

    function buildCompletionExt(): Extension {
      if (completions.length === 0) return []
      return autocompletion({ override: completions.map(makeCompletionSource), defaultKeymap: true })
    }

    function onDocChange(newValue: string): void {
      suppressUpdate = true
      valSig.set(newValue)
      suppressUpdate = false
      host.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
      clearTimeout(debounceTimer)
      debounceTimer = setTimeout(() => {
        host.dispatchEvent(new CustomEvent('sui-change', {
          detail: { value: newValue }, bubbles: true, composed: true,
        }))
      }, DEBOUNCE_MS)
    }

    const showLn = lnSig() !== false
    const extensions: Extension[] = [
      showLn
        ? lineNumCompartment.of([lineNumbers(), highlightActiveLineGutter(), foldGutter()])
        : lineNumCompartment.of([]),
      history(),
      drawSelection(),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      closeBrackets(),
      rectangularSelection(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      keymap.of([
        {
          key: 'Mod-s',
          preventDefault: true,
          run: () => {
            host.dispatchEvent(new CustomEvent('sui-save', {
              detail: { value: view?.state.doc.toString() ?? '' },
              bubbles: true, composed: true,
            }))
            return true
          },
        },
        {
          key: 'Escape',
          run: () => {
            host.dispatchEvent(new CustomEvent('sui-cancel', { bubbles: true, composed: true }))
            return false
          },
        },
        ...closeBracketsKeymap, ...defaultKeymap,
        ...searchKeymap, ...historyKeymap, ...foldKeymap,
        indentWithTab,
      ]),
      suiEditorTheme,
      suiHighlighting,
      lintGutter(),
      langCompartment.of([]),
      completionCompartment.of(buildCompletionExt()),
      readonlyCompartment.of(EditorState.readOnly.of(roSig())),
      placeholderCompartment.of(phSig() ? cmPlaceholder(phSig()) : []),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && !suppressUpdate) {
          onDocChange(update.state.doc.toString())
        }
      }),
    ]

    view = new EditorView({
      state: EditorState.create({ doc: valSig() || '', extensions }),
      parent: hostEl,
    })

    if (pendingDiagnostics) {
      applyDiagnostics(pendingDiagnostics)
      pendingDiagnostics = null
    }

    function applyDiagnostics(diagnostics: SuiDiagnostic[]): void {
      if (!view) return
      const st = view.state
      const doc = st.doc
      const cmDiags: Diagnostic[] = diagnostics.map((d) => {
        const line = Math.max(1, Math.min(doc.lines, d.line | 0))
        const startLine = doc.line(line)
        const col = Math.max(1, (d.column ?? 1) | 0)
        const from = Math.min(startLine.to, startLine.from + col - 1)
        const endL = d.endLine ?? line
        const endLineObj = doc.line(Math.max(1, Math.min(doc.lines, endL | 0)))
        const endCol = d.endColumn
        const to = endCol != null
          ? Math.min(endLineObj.to, endLineObj.from + (endCol - 1))
          : endLineObj.to
        return {
          from,
          to: Math.max(from + 1, to),
          severity: d.severity,
          message: d.message,
          source: d.source,
        } as Diagnostic
      })
      view.dispatch(cmSetDiagnostics(st, cmDiags))
    }

    const api = host as unknown as Partial<SuiCodeEditorApi>
    api.setDiagnostics = (diags: SuiDiagnostic[]) => {
      if (!view) { pendingDiagnostics = diags; return }
      applyDiagnostics(diags)
    }
    api.setCompletions = (src: SuiCompletionSource[]) => {
      completions = src ?? []
      view?.dispatch({ effects: completionCompartment.reconfigure(buildCompletionExt()) })
    }
    api.focus = () => { view?.focus() }

    Object.defineProperty(host, 'value', {
      get: () => view?.state.doc.toString() ?? valSig(),
      set: (v: string) => { valSig.set(v ?? '') },
      configurable: true,
      enumerable: false,
    })

    const disposers: Array<() => void> = []

    disposers.push(effect(() => {
      const newVal = valSig()
      if (!view || suppressUpdate) return
      const current = view.state.doc.toString()
      if (newVal !== current) {
        suppressUpdate = true
        view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: newVal || '' } })
        suppressUpdate = false
      }
    }))

    disposers.push(effect(() => {
      const lang = langSig()
      if (!view) return
      if (lang) {
        loadLanguage(lang)
          .then((ext) => view?.dispatch({ effects: langCompartment.reconfigure(ext) }))
          .catch(() => {})
      } else {
        view.dispatch({ effects: langCompartment.reconfigure([]) })
      }
    }))

    disposers.push(effect(() => {
      view?.dispatch({ effects: readonlyCompartment.reconfigure(EditorState.readOnly.of(roSig())) })
    }))

    disposers.push(effect(() => {
      const show = lnSig() !== false
      view?.dispatch({
        effects: lineNumCompartment.reconfigure(
          show ? [lineNumbers(), highlightActiveLineGutter(), foldGutter()] : [],
        ),
      })
    }))

    disposers.push(effect(() => {
      const ph = phSig()
      view?.dispatch({ effects: placeholderCompartment.reconfigure(ph ? cmPlaceholder(ph) : []) })
    }))

    return () => {
      clearTimeout(debounceTimer)
      for (const d of disposers) d()
      api.setDiagnostics = undefined
      api.setCompletions = undefined
      // Leave api.focus — it shadows native HTMLElement.focus; nulling breaks it.
      try {
        delete (host as unknown as Record<string, unknown>).value
      } catch {
        (host as unknown as Record<string, unknown>).value = undefined
      }
      try { view?.destroy() } catch { /* teardown race */ }
      view = null
    }
  },
})

function toCmCompletion(it: SuiCompletionSource['items'][number]): Completion {
  const c: Completion = { label: it.label, apply: it.insert ?? it.label }
  if (it.detail !== undefined) c.detail = it.detail
  if (it.info !== undefined) c.info = it.info
  return c
}

function makeCompletionSource(src: SuiCompletionSource) {
  return (ctx: CompletionContext): CompletionResult | null => {
    const { pos, state } = ctx
    const trig = src.trigger
    const start = pos - trig.length
    if (start < 0) return null
    const preceding = state.sliceDoc(start, pos)
    if (preceding !== trig) {
      const word = ctx.matchBefore(/[A-Za-z0-9_.]+/)
      if (!word) return null
      const before = state.sliceDoc(Math.max(0, word.from - trig.length), word.from)
      if (before !== trig) return null
      return {
        from: word.from,
        options: src.items.map(toCmCompletion),
      }
    }
    return {
      from: pos,
      options: src.items.map(toCmCompletion),
    }
  }
}
