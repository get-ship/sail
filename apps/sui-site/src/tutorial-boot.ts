// Tutorial editor↔preview wiring — sui-site's port of ship-marketing's
// TutorialApp. Uses `<sui-code-editor>` (the local SUI CodeMirror wrapper,
// framework-pure) for the two editor panes + a sandboxed iframe preview.
//
// Flow on every edit (debounced):
//   sui-change (debounced 150ms inside <sui-code-editor>)
//     → wasm compile(source, {componentName: tag, setupUrl: ...})
//     → tsToEsm(controller) regex strip
//     → postMessage({kind:'tutorial:run', template, controller, tag}) to iframe
//
// The iframe (public/tut-host.html) blob-injects the controller, rewrites the
// template's controller import to that blob, dynamic-imports the template,
// and mounts a fresh <tag>.
//
// Plain imperative TS, kept OUT of the route controller so the SUI compiler's
// scope-scanner never sees the regex / postMessage / signal code strings.

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
  setLesson: (lesson: LessonCode) => void
  dispose: () => void
}

interface SuiDiagnostic {
  line: number
  column?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

/** Imperative handle to a `<sui-code-editor>` once it has upgraded. */
interface CodeEditor {
  host: HTMLElement
  getValue: () => string
  setValue: (next: string) => void
  setDiagnostics: (diags: SuiDiagnostic[]) => void
}

/** Narrow an unknown element to a `<sui-code-editor>` host-shape via a small
 *  set of structural checks — avoid the inline `(... as { ... }).x` pattern. */
function asCodeEditor(el: Element | null): CodeEditor | null {
  if (!el) return null
  const host = el as HTMLElement & Record<string, unknown>
  if (typeof host.value !== 'string') return null
  if (typeof host.setDiagnostics !== 'function') return null
  return {
    host,
    getValue: () => host.value as string,
    setValue: (next: string) => { host.value = next },
    setDiagnostics: (diags) => {
      const fn = host.setDiagnostics as (...a: unknown[]) => void
      fn.call(host, diags)
    },
  }
}

interface DiffHandle { from: string; to: string }
function asDiff(el: Element | null): DiffHandle | null {
  if (!el) return null
  const host = el as HTMLElement & Record<string, unknown>
  if (typeof host.from !== 'string' || typeof host.to !== 'string') return null
  return {
    get from() { return host.from as string },
    set from(s: string) { host.from = s },
    get to() { return host.to as string },
    set to(s: string) { host.to = s },
  }
}

const DEBOUNCE_MS = 200

// Slugs that ship a mock-daemon scenario (public/tut-<slug>.js). Mirrors the
// committed tut-*.js files — explicit set so we don't HEAD-probe non-existent
// scenarios and surface 404s as console errors.
const SCENARIO_SLUGS = new Set(['22-data-live', '23-while-loading', '24-morphs'])

/** Strip TypeScript-only syntax from a (simple) lesson controller so it runs
 *  as ESM in the preview iframe. Verbatim port of ship-marketing's tsToEsm —
 *  the lesson controllers are simple and this transform is framework-agnostic. */
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

function createDebouncer(): (fn: () => void) => () => void {
  let timer: ReturnType<typeof setTimeout> | null = null
  return (fn: () => void) => {
    clearTimeout(timer)
    timer = setTimeout(fn, DEBOUNCE_MS)
  }
}

export function bootTutorialApp(root: HTMLElement, compile: CompileFn): TutorialHandle {
  const tplEl = asCodeEditor(root.querySelector('sui-code-editor[data-pane="template"]'))
  const ctrlEl = asCodeEditor(root.querySelector('sui-code-editor[data-pane="controller"]'))
  const iframe = root.querySelector<HTMLIFrameElement>('.tut-preview')
  const statusDot = root.querySelector<HTMLElement>('.tut-status-dot')
  const tplDiff = asDiff(root.querySelector('sui-diff[data-pane="template"]'))
  const ctrlDiff = asDiff(root.querySelector('sui-diff[data-pane="controller"]'))
  const consoleEl = root.querySelector<HTMLElement>('.tut-console-lines')
  if (!tplEl || !ctrlEl || !iframe) {
    console.error('[sui-site] tutorial editor panes not found')
    return { setLesson: () => {}, dispose: () => {} }
  }

  let lesson: LessonCode = {
    slug: '',
    starterTemplate: '',
    starterController: '',
    solvedTemplate: '',
    solvedController: '',
  }
  let tagSeq = 0
  let iframeReady = false
  const schedule = createDebouncer()

  // ── Output console ──────────────────────────────────────────────────────
  // An honest log of the real boot flow: compile start/success, compile errors
  // (full message, destructive tone), run / reset / solution actions, plus
  // console.{log,warn,error} forwarded from inside the preview iframe. Capped at
  // MAX_LOG entries, auto-scrolled to the newest line. Elements are built via
  // createElement + textContent (never innerHTML) so compiler / user output is
  // rendered as inert text.
  const MAX_LOG = 50
  type LogKind = 'info' | 'success' | 'warn' | 'error'
  const pad2 = (n: number): string => (n < 10 ? '0' + n : String(n))
  function stamp(): string {
    const d = new Date()
    return pad2(d.getMinutes()) + ':' + pad2(d.getSeconds())
  }
  function log(message: string, kind: LogKind = 'info'): void {
    if (!consoleEl) return
    const line = document.createElement('div')
    line.className = kind === 'info' ? 'tut-log-line' : `tut-log-line tut-log-line--${kind}`
    const ts = document.createElement('span')
    ts.className = 'tut-log-ts'
    ts.textContent = stamp()
    const msg = document.createElement('span')
    msg.className = 'tut-log-msg'
    msg.textContent = message
    line.append(ts, msg)
    consoleEl.append(line)
    while (consoleEl.childElementCount > MAX_LOG && consoleEl.firstElementChild) {
      consoleEl.firstElementChild.remove()
    }
    consoleEl.scrollTop = consoleEl.scrollHeight
  }
  function clearLog(): void {
    if (consoleEl) consoleEl.replaceChildren()
  }

  function setStatus(state: string, text: string): void {
    if (statusDot) {
      const toneMap: Record<string, string> = {
        idle: 'muted', typing: 'info', running: 'warning',
        ok: 'success', error: 'destructive',
      }
      statusDot.setAttribute('tone', toneMap[state] ?? 'muted')
      // Pulse while working AND when live+green — the resting "· live" state.
      if (state === 'typing' || state === 'running' || state === 'ok') {
        statusDot.setAttribute('pulse', '')
      } else {
        statusDot.removeAttribute('pulse')
      }
      statusDot.setAttribute('label', text)
    }
    root.dataset.tutState = state
    // Compile errors surface in the OUTPUT CONSOLE (full message, destructive
    // tone) rather than a cramped inline box — see runNow()'s log() calls.
  }

  function resolvedMode(): 'light' | 'dark' {
    return document.documentElement.dataset.mode === 'dark' ? 'dark' : 'light'
  }
  function reloadIframe(): void {
    const base = iframe!.dataset.baseSrc ?? '/tut-host.html'
    iframeReady = false
    iframe!.src = `${base}?mode=${resolvedMode()}&_=${Date.now()}`
  }

  function runNow(): void {
    if (!iframeReady) return
    setStatus('running', 'compiling…')
    const safeSlug = (lesson.slug || 'lesson').replace(/[^a-z0-9-]/gi, '')
    const tag = `tut-${safeSlug}-${++tagSeq}`
    try {
      const result = compile(tplEl.getValue(), {
        componentName: tag,
        shadow: false,
        setupUrl: `./${tag}.controller.ts`,
      })
      if (result.kind === 'err') {
        setStatus('error', result.message)
        // The big UX win: the FULL compiler message lands in the console,
        // destructive-toned, instead of a cramped one-line box.
        log(result.message, 'error')
        tplEl.setDiagnostics([{
          line: result.line ?? 1,
          column: result.col ?? 1,
          severity: 'error',
          message: result.message,
          source: 'sui-compiler',
        }])
        return
      }
      tplEl.setDiagnostics([])
      const controllerJs = tsToEsm(ctrlEl.getValue())
      iframe!.contentWindow?.postMessage(
        { kind: 'tutorial:run', template: result.js, controller: controllerJs, tag },
        '*',
      )
      setStatus('ok', 'running')
      log('▶ ran template.sui', 'success')
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      setStatus('error', msg)
      log(msg, 'error')
    }
  }

  function scheduleRun(): void {
    setStatus('typing', 'editing…')
    schedule(runNow)
  }

  function onTplChange(ev: Event): void {
    const v = (ev as CustomEvent<{ value: string }>).detail?.value
    if (typeof v !== 'string') return
    if (tplDiff) tplDiff.from = v
    scheduleRun()
  }
  function onCtrlChange(ev: Event): void {
    const v = (ev as CustomEvent<{ value: string }>).detail?.value
    if (typeof v !== 'string') return
    if (ctrlDiff) ctrlDiff.from = v
    scheduleRun()
  }
  tplEl.host.addEventListener('sui-change', onTplChange)
  ctrlEl.host.addEventListener('sui-change', onCtrlChange)

  function applyCode(template: string, controller: string): void {
    tplEl.setValue(template)
    ctrlEl.setValue(controller)
    if (tplDiff) tplDiff.from = template
    if (ctrlDiff) ctrlDiff.from = controller
    runNow()
  }
  // Solution is a toggle: Show solution → Hide solution (back to your code).
  let solutionShown = false
  const solutionBtn = root.querySelector<HTMLElement>('[data-action="solution"]')
  function syncSolutionBtn(): void {
    if (solutionBtn) solutionBtn.textContent = solutionShown ? 'Hide solution' : 'Show solution'
  }
  root.querySelector('[data-action="reset"]')?.addEventListener('click', () => {
    solutionShown = false
    syncSolutionBtn()
    applyCode(lesson.starterTemplate, lesson.starterController)
    log('↺ reset to starter', 'info')
  })
  solutionBtn?.addEventListener('click', () => {
    solutionShown = !solutionShown
    syncSolutionBtn()
    if (solutionShown) {
      applyCode(lesson.solvedTemplate, lesson.solvedController)
      log('shown solution', 'info')
    } else {
      applyCode(lesson.starterTemplate, lesson.starterController)
      log('hid solution — back to your code', 'info')
    }
  })
  root.querySelector('[data-action="run"]')?.addEventListener('click', () => {
    log('▶ run', 'info')
    runNow()
  })

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
  root.querySelector('sui-tabs')?.addEventListener('sui-tab-change', onTabChange)

  const onMessage = async (e: MessageEvent): Promise<void> => {
    if (e.source !== iframe!.contentWindow) return
    // console.{log,warn,error} forwarded from inside the preview iframe.
    if (e.data?.kind === 'tutorial:console') {
      const level = String(e.data.level ?? 'log')
      const text = String(e.data.text ?? '')
      const kind: LogKind = level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'info'
      log(`[preview] ${text}`, kind)
      return
    }
    if (e.data?.kind !== 'tutorial:ready') return
    if (SCENARIO_SLUGS.has(lesson.slug)) {
      try {
        const scenarioUrl = `/tut-${lesson.slug}.js`
        const head = await fetch(scenarioUrl, { method: 'HEAD' })
        if (head.ok) {
          // Runtime-selected scenario — see SCENARIO_SLUGS.
          const mod = (await import(/* @vite-ignore */ scenarioUrl)) as { default: Record<string, unknown> }
          iframe!.contentWindow?.postMessage({ kind: 'tutorial:scenario', scenario: mod.default }, '*')
          await new Promise((r) => setTimeout(r, 80))
        }
      } catch (err) {
        console.warn('[sui-site/tutorial] scenario load skipped:', err)
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
      solutionShown = false
      syncSolutionBtn()
      tplEl.setValue(next.starterTemplate)
      ctrlEl.setValue(next.starterController)
      if (tplDiff) { tplDiff.to = next.solvedTemplate; tplDiff.from = next.starterTemplate }
      if (ctrlDiff) { ctrlDiff.to = next.solvedController; ctrlDiff.from = next.starterController }
      clearLog()
      log(`loaded ${next.slug || 'lesson'}`, 'info')
      reloadIframe()
    },
    dispose(): void {
      tplEl.host.removeEventListener('sui-change', onTplChange)
      ctrlEl.host.removeEventListener('sui-change', onCtrlChange)
      window.removeEventListener('message', onMessage)
      root.querySelector('sui-tabs')?.removeEventListener('sui-tab-change', onTabChange)
      themeObserver.disconnect()
    },
  }
}
