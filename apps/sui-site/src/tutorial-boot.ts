// Tutorial editor↔preview wiring — a port of ship-marketing's proven
// TutorialApp bootApp, adapted for the deck sui-site:
//   - the editor is a sui-native <textarea> per pane (NO ship-code-editor —
//     sui-site ships public and must not depend on the ship platform);
//   - the compile seam is @ship/sui-viewer's wasm `CompileFn` (object opts);
//   - the preview is a sandboxed <iframe src="/tutorial/host.html"> that runs
//     the compiled lesson in full isolation (its own /_deck/runtime.js).
//
// Flow on every edit (debounced): wasmCompile(template, {componentName: tag,
// shadow:false, setupUrl:'./<tag>.controller.ts'}) → tsToEsm(controller) regex
// TS→JS → postMessage({kind:'tutorial:run', template, controller, tag}) to the
// iframe, which blob-injects the controller, rewrites the template's controller
// import to that blob, dynamic-imports the template, and mounts a fresh <tag>.
//
// This module is plain imperative TS kept OUT of the route controller so the
// SUI compiler's scope-scanner never sees the regex/postMessage code strings.

/** The wasm compile result the engine returns (mirrors @ship/sui-viewer). */
type CompileResult =
  | { kind: 'ok'; js: string; css?: string | null }
  | { kind: 'err'; message: string; line?: number | null; col?: number | null }
/** The bound compile function (source, opts) → result. */
export type CompileFn = (
  source: string,
  opts?: { componentName?: string; shadow?: boolean; setupUrl?: string },
) => CompileResult

export interface LessonCode {
  slug: string
  starterTemplate: string
  starterController: string
  solvedTemplate: string
  solvedController: string
}

export interface TutorialHandle {
  /** Load a lesson — resets the editors to its starter and re-runs. */
  setLesson: (lesson: LessonCode) => void
  /** Tear down listeners + observers. */
  dispose: () => void
}

const DEBOUNCE_MS = 300

// Slugs that ship a mock-daemon scenario (public/tut-<slug>.js). Kept as an
// explicit set so the host never HEAD-probes a non-existent scenario (a 404 the
// browser surfaces as a console error). Mirrors the committed tut-*.js files.
const SCENARIO_SLUGS = new Set(['22-data-live', '23-while-loading', '24-morphs'])

/** Strip TypeScript-only syntax from a (simple) lesson controller so it runs as
 *  ESM in the preview iframe. Verbatim port of ship-marketing's tsToEsm — the
 *  lesson controllers are simple and this transform is framework-agnostic. */
function tsToEsm(src: string): string {
  let s = src
  s = s.replace(/^\s*import\s+type\s+\{[^}]*\}\s+from\s+["'][^"']+["']\s*;?\s*$/gm, '')
  s = s.replace(/import\s+\{([^}]+)\}\s+from/g, (_m, inner: string) => {
    const cleaned = inner
      .split(',')
      .map((p) => p.trim())
      .filter((p) => p && !p.startsWith('type '))
      .join(', ')
    return `import { ${cleaned} } from`
  })
  s = s.replace(/^\s*interface\s+\w+(\s+extends\s+[^{]+)?\s*\{[^}]*\}\s*/gm, '')
  // Type aliases — terminated by `;` OR end-of-line (lesson controllers don't
  // always semicolon them, e.g. `type State = 'idle' | 'loading'`).
  s = s.replace(/^\s*type\s+\w+\s*=\s*[^;\n]+;?[ \t]*$/gm, '')
  s = s.replace(/\(([^()]*)\)\s*(:\s*[^=>{;,)]+)?\s*=>/g, (_m, params: string) => {
    const cleaned = params
      .split(',')
      .map((p) => p.split(':')[0].trim())
      .filter(Boolean)
      .join(', ')
    return `(${cleaned}) =>`
  })
  s = s.replace(/(\w+)<[^<>()]*>\(/g, '$1(')
  s = s.replace(/\s+as\s+[\w<>[\]|\s.,]+(?=[;,)\]}])/g, '')
  s = s.replace(/(\bconst\s+\w+|\blet\s+\w+|\bvar\s+\w+)\s*:\s*[^=;,]+/g, '$1')
  return s
}

/** Insert two spaces on Tab (and outdent on Shift+Tab) in a textarea editor. */
function wireTabKey(ta: HTMLTextAreaElement): void {
  ta.addEventListener('keydown', (e) => {
    if (e.key !== 'Tab') return
    e.preventDefault()
    const start = ta.selectionStart
    const end = ta.selectionEnd
    const v = ta.value
    if (e.shiftKey) {
      // Outdent: drop up to two leading spaces on the current line.
      const lineStart = v.lastIndexOf('\n', start - 1) + 1
      const removed = v.slice(lineStart).match(/^ {1,2}/)?.[0].length ?? 0
      if (removed) {
        ta.value = v.slice(0, lineStart) + v.slice(lineStart + removed)
        ta.selectionStart = ta.selectionEnd = Math.max(lineStart, start - removed)
      }
    } else {
      ta.value = v.slice(0, start) + '  ' + v.slice(end)
      ta.selectionStart = ta.selectionEnd = start + 2
    }
    ta.dispatchEvent(new Event('input', { bubbles: true }))
  })
}

const TONE_MAP: Record<string, string> = {
  idle: 'muted',
  typing: 'info',
  running: 'warning',
  ok: 'success',
  error: 'destructive',
}

/**
 * Boot the editor↔preview wiring on an already-mounted root element. `compile`
 * is the bound wasm compile fn (must be ready). Returns a handle whose
 * `setLesson` swaps in a lesson and re-runs.
 */
export function bootTutorialApp(root: HTMLElement, compile: CompileFn): TutorialHandle {
  const tplEl = root.querySelector<HTMLTextAreaElement>('textarea[data-pane="template"]')!
  const ctrlEl = root.querySelector<HTMLTextAreaElement>('textarea[data-pane="controller"]')!
  const iframe = root.querySelector<HTMLIFrameElement>('.tut-preview')!
  const statusDot = root.querySelector<HTMLElement>('.tut-status-dot')
  const errorBox = root.querySelector<HTMLElement>('.tut-error')
  const errorMsg = root.querySelector<HTMLElement>('.tut-error-msg')
  const tplDiff = root.querySelector<HTMLElement & { from?: string; to?: string }>('sui-diff[data-pane="template"]')
  const ctrlDiff = root.querySelector<HTMLElement & { from?: string; to?: string }>('sui-diff[data-pane="controller"]')

  let lesson: LessonCode = {
    slug: '',
    starterTemplate: '',
    starterController: '',
    solvedTemplate: '',
    solvedController: '',
  }
  let currentTemplate = ''
  let currentController = ''
  let tagSeq = 0
  let iframeReady = false
  let debounceTimer = 0

  function setStatus(state: string, text: string): void {
    if (statusDot) {
      statusDot.setAttribute('tone', TONE_MAP[state] ?? 'muted')
      if (state === 'typing' || state === 'running') statusDot.setAttribute('pulse', '')
      else statusDot.removeAttribute('pulse')
      statusDot.setAttribute('label', text)
    }
    root.dataset.tutState = state
    if (errorBox && errorMsg) {
      if (state === 'error') {
        errorMsg.textContent = text
        errorBox.hidden = false
      } else {
        errorBox.hidden = true
      }
    }
  }

  function resolvedMode(): 'light' | 'dark' {
    return document.documentElement.dataset.mode === 'dark' ? 'dark' : 'light'
  }
  function reloadIframe(): void {
    const base = iframe.dataset.baseSrc ?? '/tut-host.html'
    iframeReady = false
    iframe.src = `${base}?mode=${resolvedMode()}&_=${Date.now()}`
  }

  function runNow(): void {
    if (!iframeReady) return
    setStatus('running', 'compiling…')
    const safeSlug = (lesson.slug || 'lesson').replace(/[^a-z0-9-]/gi, '')
    const tag = `tut-${safeSlug}-${++tagSeq}`
    try {
      const result = compile(currentTemplate, {
        componentName: tag,
        shadow: false,
        setupUrl: `./${tag}.controller.ts`,
      })
      if (result.kind === 'err') {
        setStatus('error', result.message)
        return
      }
      const controllerJs = tsToEsm(currentController)
      iframe.contentWindow!.postMessage(
        { kind: 'tutorial:run', template: result.js, controller: controllerJs, tag },
        '*',
      )
      setStatus('ok', 'running')
    } catch (e: unknown) {
      setStatus('error', e instanceof Error ? e.message : String(e))
    }
  }

  function scheduleRun(): void {
    setStatus('typing', 'editing…')
    clearTimeout(debounceTimer)
    debounceTimer = window.setTimeout(runNow, DEBOUNCE_MS)
  }

  function updateDiffFrom(): void {
    if (tplDiff) tplDiff.from = currentTemplate
    if (ctrlDiff) ctrlDiff.from = currentController
  }

  // --- editors ---
  tplEl.addEventListener('input', () => {
    currentTemplate = tplEl.value
    updateDiffFrom()
    scheduleRun()
  })
  ctrlEl.addEventListener('input', () => {
    currentController = ctrlEl.value
    updateDiffFrom()
    scheduleRun()
  })
  wireTabKey(tplEl)
  wireTabKey(ctrlEl)

  // --- actions ---
  function applyCode(template: string, controller: string): void {
    currentTemplate = template
    currentController = controller
    tplEl.value = template
    ctrlEl.value = controller
    updateDiffFrom()
    runNow()
  }
  root.querySelector('[data-action="reset"]')?.addEventListener('click', () => {
    applyCode(lesson.starterTemplate, lesson.starterController)
  })
  root.querySelector('[data-action="solution"]')?.addEventListener('click', () => {
    applyCode(lesson.solvedTemplate, lesson.solvedController)
  })
  root.querySelector('[data-action="run"]')?.addEventListener('click', () => runNow())

  // --- tab switching (sui-tabs emits sui-tab-change once upgraded) ---
  const panels = Array.from(root.querySelectorAll<HTMLElement>('sui-tab-panel'))
  function showPanel(name: string): void {
    for (const panel of panels) {
      if (panel.getAttribute('name') === name) {
        panel.setAttribute('active', '')
        panel.removeAttribute('hidden')
      } else {
        panel.removeAttribute('active')
        panel.setAttribute('hidden', '')
      }
    }
  }
  const onTabChange = (e: Event): void => {
    const name = (e as CustomEvent<{ name: string }>).detail?.name
    if (name) showPanel(name)
  }
  const tabs = root.querySelector('sui-tabs')
  tabs?.addEventListener('sui-tab-change', onTabChange)

  // --- iframe lifecycle: ready → load scenario → run; theme follows parent ---
  const onMessage = async (e: MessageEvent): Promise<void> => {
    if (e.source !== iframe.contentWindow) return
    if (e.data?.kind !== 'tutorial:ready') return
    if (SCENARIO_SLUGS.has(lesson.slug)) {
      try {
        const mod: { default: Record<string, unknown> } = await import(
          /* @vite-ignore */ `/tut-${lesson.slug}.js`
        )
        iframe.contentWindow!.postMessage({ kind: 'tutorial:scenario', scenario: mod.default }, '*')
        // Give the SW a beat to ingest the scenario before runNow wires the
        // EventSource subscriptions.
        await new Promise((r) => setTimeout(r, 80))
      } catch (err) {
        console.warn('[tutorial] scenario load skipped:', err)
      }
    }
    iframeReady = true
    runNow()
  }
  window.addEventListener('message', onMessage)

  const themeObserver = new MutationObserver(() => reloadIframe())
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-mode', 'data-theme'],
  })

  return {
    setLesson(next: LessonCode): void {
      lesson = next
      currentTemplate = next.starterTemplate
      currentController = next.starterController
      tplEl.value = next.starterTemplate
      ctrlEl.value = next.starterController
      if (tplDiff) tplDiff.to = next.solvedTemplate
      if (ctrlDiff) ctrlDiff.to = next.solvedController
      updateDiffFrom()
      // Reload the iframe so the new lesson's scenario (if any) is primed before
      // the first run, then runNow fires on `tutorial:ready`.
      reloadIframe()
    },
    dispose(): void {
      clearTimeout(debounceTimer)
      window.removeEventListener('message', onMessage)
      tabs?.removeEventListener('sui-tab-change', onTabChange)
      themeObserver.disconnect()
    },
  }
}
