<!--
  /playground — free workbench: recipe select + editor + same-doc preview
  + console. Fill/overflow lives on <sail-fill-shell> (layout). Route is
  tags only — pad → card, no Style.
-->
<scope>
  type LogLineProps = {
    ts: string
    kind: string
    msg: string
  }
</scope>

<controller>
import type { Action, ControllerScope, SailHost } from '@ship/sail-runtime'
import { ensureWasm, makeCompile } from '../../src/compositions/live-sail-boot'
import type { Scope } from './page.sail'
import {
  createDebouncer,
  runInHost,
  type CodeEditorHost,
  type CompileFn,
  type PreviewMount,
} from '../../src/compositions/live-sail-boot'
import {
  DEFAULT_RECIPE_ID,
  PLAYGROUND_RECIPES,
  recipeById,
} from '../../src/compositions/playground-recipes'

interface LogLine {
  id: number
  ts: string
  kind: 'info' | 'success' | 'warn' | 'error'
  msg: string
}

export default controller((_ctx: ControllerScope<Scope>) => {
  const editorEl = signal<CodeEditorHost | null>(null)
  const hostEl = signal<SailHost | null>(null)
  let previewMount: PreviewMount | null = null

  const captureEditor: Action<void> = (node) => {
    editorEl.set(node as CodeEditorHost)
    return { destroy: () => editorEl.set(null) }
  }
  const captureHost: Action<void> = (node) => {
    hostEl.set(asSailHost(node))
    return {
      destroy: () => {
        try {
          previewMount?.dispose()
        } catch {
          /* best-effort */
        }
        previewMount = null
        hostEl.set(null)
      },
    }
  }

  const recipeId = signal(DEFAULT_RECIPE_ID)
  const statusTone = signal<string>('muted')
  const statusText = signal<string>('loading…')
  const logs = signal<LogLine[]>([])
  const srcValue = signal('')
  const recipes = PLAYGROUND_RECIPES

  const compile = signal<CompileFn | null>(null)
  ensureWasm('/sail_compiler_bg.wasm')
    .then(() => compile.set(() => makeCompile()))
    .catch((err) => console.error('[sail-site] wasm compile seam failed:', err))

  const bootedAt = Date.now()
  const pad2 = (n: number): string => (n < 10 ? '0' + n : String(n))
  let logSeq = 0
  const log = (msg: string, kind: LogLine['kind'] = 'info'): void => {
    const s = Math.floor((Date.now() - bootedAt) / 1000)
    const ts = pad2(Math.floor(s / 60)) + ':' + pad2(s % 60)
    logs.set((cur) => [...cur, { id: ++logSeq, ts, kind, msg }].slice(-50))
  }

  const setStatus = (state: 'idle' | 'typing' | 'running' | 'ok' | 'error', text: string): void => {
    const tones: Record<string, string> = {
      idle: 'muted', typing: 'info', running: 'warning', ok: 'success', error: 'danger',
    }
    statusTone.set(tones[state])
    statusText.set(text)
  }

  let tagSeq = 0
  let lastRecipeId = ''
  let lastPushedKey = ''
  const schedule = createDebouncer(200)

  const runNow = (): void => {
    const c = compile()
    const host = hostEl()
    if (!c || !host) return
    setStatus('running', 'compiling…')
    const safeId = (recipeId() || 'pg').replace(/[^a-z0-9-]/gi, '')
    const { result, mount, ready } = runInHost(
      c,
      host,
      {
        tag: `pg-${safeId}-${++tagSeq}`,
        source: srcValue(),
      },
      previewMount,
    )
    previewMount = mount
    if (result.kind === 'err') {
      setStatus('error', 'compile error')
      log(result.message, 'error')
      editorEl()?.setDiagnostics?.([{
        line: result.line ?? 1,
        column: result.col ?? 1,
        severity: 'error',
        message: result.message,
        source: 'sail-compiler',
      }])
      return
    }
    editorEl()?.setDiagnostics?.([])
    if (ready) {
      void ready
        .then((m) => {
          previewMount = m
          setStatus('ok', 'running')
          log('▶ ran playground', 'success')
        })
        .catch((err: unknown) => {
          const message = err instanceof Error ? err.message : String(err)
          setStatus('error', 'compile error')
          log('mount: ' + message, 'error')
        })
    } else {
      setStatus('ok', 'running')
      log('▶ ran playground', 'success')
    }
  }

  const applyCode = (source: string): void => {
    srcValue.set(source)
    lastPushedKey = ''
    const e = editorEl()
    if (e && typeof e.setDiagnostics === 'function') {
      e.value = source
      lastPushedKey = recipeId() + '\0' + source.length + '\0' + source.slice(0, 32)
    }
    runNow()
  }

  const onSourceChange = (detail: { value: string }): void => {
    const v = detail?.value
    if (typeof v !== 'string') return
    srcValue.set(v)
    setStatus('typing', 'editing…')
    schedule(runNow)
  }

  const reset = (): void => {
    const r = recipeById(recipeId()) ?? recipeById(DEFAULT_RECIPE_ID)
    if (!r) return
    applyCode(r.source)
    log('↺ reset to recipe', 'info')
  }

  const clearConsole = (): void => {
    logs.set([])
  }

  const run = (): void => {
    log('▶ run', 'info')
    runNow()
  }

  effect(() => {
    const id = recipeId()
    const r = recipeById(id)
    if (!r) return
    if (id !== lastRecipeId) {
      lastRecipeId = id
      srcValue.set(r.source)
      lastPushedKey = ''
      log(`loaded recipe: ${r.title}`, 'info')
    }
    const ed = editorEl()
    const source = srcValue()
    if (!ed || !source) return
    const key = id + '\0' + source.length + '\0' + source.slice(0, 32)
    if (key === lastPushedKey) return
    const push = (): void => {
      if (editorEl() !== ed) return
      if (typeof ed.setDiagnostics !== 'function') {
        scheduleFrame(push)
        return
      }
      ed.value = source
      lastPushedKey = key
    }
    push()
  })

  let lastPreviewRecipe = ''
  effect(() => {
    const id = recipeId()
    const host = hostEl()
    const c = compile()
    if (!host || !c) return
    if (id === lastPreviewRecipe && previewMount) return
    lastPreviewRecipe = id
    runNow()
  })

  effect(() => () => {
    try {
      previewMount?.dispose()
    } catch {
      /* best-effort */
    }
    previewMount = null
  })

  const recipeTitle = derived(() => recipeById(recipeId())?.title ?? 'Blank')
  const recipeDesc = derived(() => recipeById(recipeId())?.description ?? '')

  return {
    recipeId,
    recipes,
    recipeTitle,
    recipeDesc,
    statusTone,
    statusText,
    logs,
    srcValue,
    captureEditor,
    captureHost,
    onSourceChange,
    reset,
    clearConsole,
    run,
  }
})
</controller>

<partial log-line props: LogLineProps>
  <match {kind}>
    {'success' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {'warn' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {'error' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {else => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
  </match>
</partial>

<when {tablet}>
<column gap="2">
  <row nowrap> gap="2" testid="pg-toolbar">
    <a href="/docs" label="Back to Docs" testid="pg-back">← Docs</a>
    <p>PLAYGROUND</p>
    <select
      size="sm"
      bind:value={recipeId}
      label="Recipe"
      testid="pg-recipe"
    >
      <for {recipes as r} key={r.id}>
        <option value={r.id}>{r.title}</option>
      </for>
    </select>
    <small>{recipeDesc()}</small>
    <theme-picker></theme-picker>
    <button variant="ghost" size="sm" on:click={reset}>↺ Reset</button>
    <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
  </row>

  <row nowrap gap="2" align="stretch">
    <card padding="none">
      <sail-code-editor
        pane="source"
        language="sail"
        size="sm"
        chrome="flush"
        wasm-url="/sail_compiler_bg.wasm"
        use:captureEditor
        on-change={onSourceChange}
      ></sail-code-editor>
    </card>

    <column gap="0">
      <row nowrap> gap="2">
        <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
        <small>PREVIEW · {recipeTitle()}</small>
        <button variant="ghost" size="sm" on:click={clearConsole}>Clear console</button>
      </row>
      <card padding="sm">
        <div
          testid="pg-preview"
          label="Playground preview"
          use:captureHost
        >Preview</div>
      </card>
      <card padding="sm" live="polite" label="Playground console">
        <for {logs() as entry} key={entry.id}>
          <log-line ts={entry.ts} kind={entry.kind} msg={entry.msg}></log-line>
        </for>
      </card>
    </column>
  </row>
</column>
{else =>
<column gap="2">
  <row nowrap> gap="2" testid="pg-toolbar">
    <a href="/docs" label="Back to Docs" testid="pg-back">← Docs</a>
    <select size="sm" bind:value={recipeId} label="Recipe" testid="pg-recipe">
      <for {recipes as r} key={r.id}>
        <option value={r.id}>{r.title}</option>
      </for>
    </select>
    <theme-picker></theme-picker>
    <button variant="ghost" size="sm" on:click={reset}>↺</button>
    <button variant="outline" tone="primary" size="sm" on:click={run}>▶</button>
  </row>
  <card padding="none">
    <sail-code-editor
      pane="source"
      language="sail"
      size="sm"
      chrome="flush"
      wasm-url="/sail_compiler_bg.wasm"
      use:captureEditor
      on-change={onSourceChange}
    ></sail-code-editor>
  </card>
  <column gap="0">
    <row nowrap> gap="2">
      <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
      <button variant="ghost" size="sm" on:click={clearConsole}>Clear</button>
    </row>
    <card padding="sm">
      <div
        testid="pg-preview"
        label="Playground preview"
        use:captureHost
      >Preview</div>
    </card>
    <card padding="sm" live="polite" label="Playground console">
      <for {logs() as entry} key={entry.id}>
        <log-line ts={entry.ts} kind={entry.kind} msg={entry.msg}></log-line>
      </for>
    </card>
  </column>
</column>}</when>
