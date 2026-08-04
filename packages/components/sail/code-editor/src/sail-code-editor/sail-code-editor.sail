<!--
  <sail-code-editor> — SAIL primitive, framework-pure. CodeMirror 6 wrapped as a
  lazy-loaded web-component. Drop-in for any code-editing surface in sail-site
  (or any SAIL app) without depending on the ship platform.

  Prop / event parity
  ───────────────────────────────────────────────────────────────────────────
  value (string, '', form-value)   controlled text contents
  language (string, '')            dynamic-imported extension on assignment
  placeholder (string)             empty-state copy
  readonly (bool, false, reflect)  drop all editor interactions
  line-numbers (bool, true, reflect)
  size (sm|md|lg, md, reflect)     controls font-size + line-height
  tone (default|info|success|warning|destructive, default, reflect)
  theme (light|dark|auto, auto, reflect)
  ───────────────────────────────────────────────────────────────────────────
  sail-change              debounced 150ms `value` updates
  sail-save                Mod-s saves (host-bound events)
  sail-cancel              Escape cancels pending save
  input                   native bubbling change event
  ───────────────────────────────────────────────────────────────────────────
  host.setDiagnostics()   imperative API for compiler-error squiggles
  host.focus()            imperative focus
  ───────────────────────────────────────────────────────────────────────────
-->

<!--
  Styling: structure + mono type ride sx on the template root; size and chrome
  are variant() maps on the host (chrome: card = standalone box, flush =
  embedded in a host-drawn pane). CodeMirror's own internals (.cm-*) are
  themed in sail-code-editor.theme.ts — the FFI library's styling system.
  [hidden] comes from the framework contracts sheet.
-->
<scope>
  props: {
    value = ''
    language = ''
    placeholder = ''
    readonly = false
    'line-numbers' = true
    tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' = 'default'
    theme: 'auto' | 'light' | 'dark' = 'auto'
    'wasm-url' = '/sail_compiler_bg.wasm'
    size: {
      md: 'text-sm leading-normal-plus',
      sm: 'text-xs leading-normal',
      lg: 'text-base leading-loose',
    }
    chrome: {
      card: 'border-border bg-surface-1 rounded-lg shadow-xs',
      flush: '',
    }
  }
</scope>

<controller>
// use: action + mount() pattern: element → EditorView → event bridge → cleanup.

import type { Action, Signal } from '@ship/sail-runtime'
import { EditorState, Compartment } from '@codemirror/state'
import type { Extension } from '@codemirror/state'
import {
  EditorView, keymap,
  placeholder as cmPlaceholder, lineNumbers,
  highlightActiveLine, highlightActiveLineGutter,
  drawSelection, rectangularSelection,
  hoverTooltip, type Tooltip,
} from '@codemirror/view'
import {
  defaultHighlightStyle, syntaxHighlighting,
  indentOnInput, bracketMatching, foldGutter, foldKeymap,
} from '@codemirror/language'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import {
  closeBrackets, closeBracketsKeymap,
  autocompletion,
  type CompletionContext, type CompletionResult, type Completion,
} from '@codemirror/autocomplete'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { lintGutter, setDiagnostics as cmSetDiagnostics, type Diagnostic } from '@codemirror/lint'
import { sailEditorTheme, sailHighlighting } from './sail-code-editor.theme'
import { loadLanguage } from './sail-code-editor.lang'
import type { Scope } from './sail-code-editor.sail'

export interface SailDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

export interface SailCompletionSource {
  trigger: string
  items: Array<{
    label: string
    detail?: string
    info?: string
    insert?: string
  }>
}

const DEBOUNCE_MS = 150

export default controller((ctx: Scope) => {
    const langCompartment = new Compartment()
    const readonlyCompartment = new Compartment()
    const lineNumCompartment = new Compartment()
    const placeholderCompartment = new Compartment()
    const completionCompartment = new Compartment()

    let view: EditorView | null = null
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    let suppressUpdate = false
    /** Host-pushed diagnostics (studio checks). Merged with browser LSP diags. */
    let hostDiagnostics: SailDiagnostic[] = []
    /** Browser LSP diagnose() results. */
    let lspDiagnostics: SailDiagnostic[] = []
    let pendingDiagnostics: SailDiagnostic[] | null = null
    let completions: SailCompletionSource[] = []
    let hoverAtFn: ((source: string, offset: number) => { contents: string } | null) | null = null

    const valSig = ctx.value as unknown as Signal<string>
    const langSig = ctx.language as unknown as Signal<string>
    const roSig = ctx.readonly as unknown as Signal<boolean>
    const lnSig = ctx.lineNumbers as unknown as Signal<boolean>
    const phSig = ctx.placeholder as unknown as Signal<string>
    const wasmUrlSig = ctx.wasmUrl as unknown as Signal<string>

    function buildCompletionExt(): Extension {
      if (completions.length === 0) return []
      return autocompletion({ override: completions.map(makeCompletionSource), defaultKeymap: true })
    }

    /** Map a diagnostic to a tight CM range — never paint the whole line. */
    function diagToCm(doc: EditorState['doc'], d: SailDiagnostic): Diagnostic {
      const line = Math.max(1, Math.min(doc.lines, d.line | 0))
      const startLine = doc.line(line)
      const col = Math.max(1, (d.column ?? 1) | 0)
      const from = Math.min(startLine.to, startLine.from + col - 1)
      let to: number
      if (d.endLine != null && d.endColumn != null) {
        const endL = Math.max(1, Math.min(doc.lines, d.endLine | 0))
        const endLineObj = doc.line(endL)
        to = Math.min(endLineObj.to, endLineObj.from + Math.max(0, (d.endColumn | 0) - 1))
      } else if (d.endColumn != null) {
        to = Math.min(startLine.to, startLine.from + Math.max(0, (d.endColumn | 0) - 1))
      } else {
        // No end: span one identifier/token, not the remainder of the line
        // (full-line underlines stack into the "overlapping warnings" mess).
        let end = from
        const text = doc.sliceString(from, startLine.to)
        const m = text.match(/^[A-Za-z0-9_$:.-]+/)
        end = m ? from + m[0].length : Math.min(startLine.to, from + 1)
        to = end
      }
      return {
        from,
        to: Math.max(from + 1, to),
        severity: d.severity,
        message: d.message,
        source: d.source,
      } as Diagnostic
    }

    function mergeAndApplyDiagnostics(): void {
      if (!view) return
      const st = view.state
      const doc = st.doc
      // Dedupe by line+col+message so host checks and browser LSP don't
      // paint two squiggles on the same span.
      const seen = new Set<string>()
      const merged: Diagnostic[] = []
      for (const d of [...lspDiagnostics, ...hostDiagnostics]) {
        const key = `${d.line}|${d.column ?? 0}|${d.message}`
        if (seen.has(key)) continue
        seen.add(key)
        merged.push(diagToCm(doc, d))
      }
      // Sort by from so CM lint layer is stable.
      merged.sort((a, b) => a.from - b.from || a.to - b.to)
      view.dispatch(cmSetDiagnostics(st, merged))
    }

    function applyDiagnostics(diagnostics: SailDiagnostic[]): void {
      // Public host API: treat as host/check stream (replaces prior host set).
      hostDiagnostics = diagnostics ?? []
      if (!view) {
        pendingDiagnostics = diagnostics
        return
      }
      mergeAndApplyDiagnostics()
    }

    // The editor element owns the EditorView: the action builds it on mount
    // and destroys it on unmount. `editorHost` used to be a `bind:this`
    // target that was never declared in scope at all — it existed only
    // because the compiler pre-seeds a signal slot for every bind:this
    // target, and the controller reached it through a double cast.
    const editorMount: Action<void> = (node) => {

      function onDocChange(newValue: string): void {
        suppressUpdate = true
        valSig.set(newValue)
        suppressUpdate = false
        node.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
        if (debounceTimer) clearTimeout(debounceTimer)
        debounceTimer = setTimeout(() => {
          node.dispatchEvent(new CustomEvent('sail-change', {
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
              node.dispatchEvent(new CustomEvent('sail-save', {
                detail: { value: view?.state.doc.toString() ?? '' },
                bubbles: true, composed: true,
              }))
              return true
            },
          },
          {
            key: 'Escape',
            run: () => {
              node.dispatchEvent(new CustomEvent('sail-cancel', { bubbles: true, composed: true }))
              return false
            },
          },
          ...closeBracketsKeymap, ...defaultKeymap,
          ...searchKeymap, ...historyKeymap, ...foldKeymap,
          indentWithTab,
        ]),
        sailEditorTheme,
        sailHighlighting,
        lintGutter(),
        // Hover: SAIL structure docs from sail-lsp-browser/hover when available.
        hoverTooltip((v, pos): Tooltip | null => {
          if (!hoverAtFn) return null
          const text = v.state.doc.toString()
          const info = hoverAtFn(text, pos)
          if (!info) return null
          return {
            pos,
            above: true,
            create() {
              const dom = document.createElement('div')
              dom.className = 'cm-tooltip-sail-hover'
              dom.style.cssText =
                'padding:6px 10px;max-width:36rem;white-space:pre-wrap;font-size:12px;line-height:1.4'
              dom.textContent = info.contents
              return { dom }
            },
          }
        }, { hoverTime: 350 }),
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
        parent: node,
      })

      if (pendingDiagnostics) {
        hostDiagnostics = pendingDiagnostics
        pendingDiagnostics = null
        mergeAndApplyDiagnostics()
      }

      return {
        destroy: () => {
          if (debounceTimer) clearTimeout(debounceTimer)
          try { view?.destroy() } catch { /* teardown race */ }
          view = null
        },
      }
    }

  mount((host) => {
    const el = host as HTMLElement & {
      setDiagnostics?: ((d: SailDiagnostic[]) => void) | undefined
      setCompletions?: ((src: SailCompletionSource[]) => void) | undefined
      /** 1-based line/col caret + scrollIntoView (studio selection → source). */
      setCursor?: ((line: number, col?: number) => void) | undefined
      focus?: () => void
    }
    el.setDiagnostics = (diags: SailDiagnostic[]) => {
      if (!view) { pendingDiagnostics = diags; return }
      applyDiagnostics(diags)
    }
    el.setCompletions = (src: SailCompletionSource[]) => {
      completions = src ?? []
      view?.dispatch({ effects: completionCompartment.reconfigure(buildCompletionExt()) })
    }
    el.setCursor = (line: number, col = 1) => {
      if (!view || line < 1) return
      const doc = view.state.doc
      const ln = Math.min(line, doc.lines)
      const lineObj = doc.line(ln)
      const colOff = Math.max(0, Math.min(lineObj.length, Math.max(0, col - 1)))
      const pos = lineObj.from + colOff
      view.dispatch({
        selection: { anchor: pos, head: pos },
        effects: EditorView.scrollIntoView(pos, { y: 'center' }),
      })
      view.focus()
    }
    el.focus = () => { view?.focus() }

    Object.defineProperty(host, 'value', {
      get: () => view?.state.doc.toString() ?? valSig(),
      // Apply directly: sync the signal AND push the document to the
      // EditorView in the same tick. The reactive effect below is the
      // source of truth for subsequent user edits, but the initial
      // host-driven assignment (e.g. the tutorial boot wiring the
      // first lesson) must populate the view synchronously — otherwise
      // CodeMirror shows an empty line "1" while the effect queues a
      // microtask that never observes a non-empty valSig.
      set: (v: string) => {
        const next = v ?? ''
        if (view) {
          const current = view.state.doc.toString()
          if (current !== next) {
            suppressUpdate = true
            view.dispatch({
              changes: { from: 0, to: view.state.doc.length, insert: next },
            })
            suppressUpdate = false
          }
        }
        valSig.set(next)
      },
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

    // ── Browser LSP integration ───────────────────────────────────
    // Diagnostics from sail-lsp-browser merge with host checks (not replace).
    // Hover uses sail-lsp-browser/hover when the framework singleton is
    // available; otherwise a small static tag/directive fallback.
    type LspDiagnoseFn = (text: string, wasmUrl?: string) => Promise<SailDiagnostic[]>
    type LspAllSourcesFn = () => SailCompletionSource[]
    type LspHoverFn = (source: string, offset: number) => { contents: string } | null
    let lspDiagnose: LspDiagnoseFn | null = null
    let lspDebounce: ReturnType<typeof setTimeout> | null = null
    let lspInit: Promise<void> | null = null
    async function ensureLsp(): Promise<void> {
      if (lspDiagnose || lspInit) { await lspInit; return }
      lspInit = (async () => {
        try {
          const [diag, comp] = await Promise.all([
            import('@ship/sail-lsp-browser/diagnostics') as unknown as Promise<{ diagnose: LspDiagnoseFn }>,
            import('@ship/sail-lsp-browser/completions') as unknown as Promise<{ allSources: LspAllSourcesFn }>,
          ])
          lspDiagnose = diag.diagnose
          try { el.setCompletions?.(comp.allSources()) } catch { /* best-effort */ }
        } catch (err) {
          console.warn('[sail-code-editor] LSP init failed; diagnostics unavailable', err)
        }
        // Hover is optional — importmap may not list it yet.
        try {
          const hoverMod = await import(
            /* @vite-ignore */ '@ship/sail-lsp-browser/hover'
          ) as unknown as { hoverAt: LspHoverFn }
          if (typeof hoverMod.hoverAt === 'function') hoverAtFn = hoverMod.hoverAt
        } catch {
          hoverAtFn = staticSailHover
        }
        if (!hoverAtFn) hoverAtFn = staticSailHover
      })()
      await lspInit
    }
    disposers.push(effect(() => {
      const lang = langSig()
      if (lang !== 'sail') return
      void ensureLsp()
    }))
    disposers.push(effect(() => {
      const text = valSig()
      if (langSig() !== 'sail') return
      if (lspDebounce) clearTimeout(lspDebounce)
      lspDebounce = setTimeout(async () => {
        if (!lspDiagnose) await ensureLsp()
        if (!lspDiagnose) return
        try {
          const out = await lspDiagnose(text, wasmUrlSig())
          lspDiagnostics = out ?? []
          mergeAndApplyDiagnostics()
        } catch { /* best-effort */ }
      }, 350)
    }))
    disposers.push(() => {
      if (lspDebounce) clearTimeout(lspDebounce)
      lspDiagnose = null
      lspInit = null
      hoverAtFn = null
    })

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
      for (const d of disposers) d()
      el.setDiagnostics = undefined
      el.setCompletions = undefined
      el.setCursor = undefined
      // Leave el.focus — shadows native HTMLElement.focus; nulling breaks it.
      try {
        delete (host as unknown as Record<string, unknown>).value
      } catch {
        (host as unknown as Record<string, unknown>).value = undefined
      }
    }
  })
  return { editorMount }
})

/** Static hover fallback when `@ship/sail-lsp-browser/hover` is not on the importmap. */
function staticSailHover(source: string, offset: number): { contents: string } | null {
  if (offset < 0 || offset > source.length) return null
  let start = offset
  while (start > 0 && /[\w:-]/.test(source[start - 1]!)) start--
  let end = offset
  while (end < source.length && /[\w:-]/.test(source[end]!)) end++
  const token = source.slice(start, end)
  if (!token) return null
  const docs: Record<string, string> = {
    controller: '<controller>\n\nAuthored TypeScript for this surface. Export default a setup that returns scope.',
    scope: '<scope>\n\nDeclares props and style helpers for the template.',
    column: '<column>\n\nVertical flex stack (L1 layout primitive).',
    row: '<row>\n\nHorizontal flex stack (L1 layout primitive).',
    grow: 'grow\n\nFlex grow — fills remaining space on the cross/main axis.',
    sx: 'sx\n\nStyle expression attribute — chain style.* helpers.',
    'on:': 'on:<event>\n\nEvent handler binding. Supports |prevent |stop |debounce:N modifiers.',
    'bind:': 'bind:<prop>\n\nTwo-way binding into a signal/scope member.',
  }
  if (docs[token]) return { contents: docs[token]! }
  if (token.startsWith('on:')) return { contents: docs['on:']! }
  if (token.startsWith('bind:')) return { contents: docs['bind:']! }
  // Inside controller: offer a light TS hint for common imports.
  if (token === 'import' || token === 'export' || token === 'const' || token === 'function') {
    return { contents: `\`${token}\`\n\nTypeScript keyword (controller/scope body).` }
  }
  return null
}

// CodeMirror's Completion treats absent detail/info as "none"; under
// exactOptionalPropertyTypes we can't pass `undefined` for them, so omit.
function toCmCompletion(it: SailCompletionSource['items'][number]): Completion {
  const c: Completion = { label: it.label, apply: it.insert ?? it.label }
  if (it.detail !== undefined) c.detail = it.detail
  if (it.info !== undefined) c.info = it.info
  return c
}

function makeCompletionSource(src: SailCompletionSource) {
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

</controller>
<div use:editorMount sx={style.position(relative).overflow(hidden).fontFamily(mono).w(full).h(full).minH(0).when({ focusWithin: style.outline(ring) })}></div>
