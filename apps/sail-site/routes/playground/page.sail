<!--
  /playground — free workbench: recipe select + editor + same-doc preview
  + console. Reuses live-sail-boot (wasm compile + mount). No lesson prose,
  stepper, solution toggle, or diff tab.
-->
<scope>
  style workbench = { box: size({ w: full, h: fill, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  style workbenchMobile = { box: size({ w: full, h: fill, min-h: 0 }).overflow({ all: hidden }) }
  // Full-bleed workbench: single chrome bar, no site navbar tax.
  style playground = { box: flow({ grow: 1 }).basis(0).size({ min-w: 0, min-h: 0 }).space({ pad: half }), paint: fill({ color: surface-0 }) }
  // Desktop: editor | preview+console side by side (not stacked).
  style split = { box: flow({ grow: 1 }).basis(0).size({ min-w: 0, min-h: 0 }) }
  style card = { box: flow({ grow: 1 }).basis(0).size({ min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: border({ color: border, width: 1 }).radius(md).fill({ color: surface-1 }) }
  style toolbar = { box: flow({ shrink: 0 }).space({ pad-x: 2, pad-y: 1 }), paint: border({ color: border, width: 1, edge: b }).fill({ color: surface-1 }) }
  style paneCol = { box: flow({ grow: 1 }).size({ min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
  style monoXs = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, tracking: wide }) }
  style countLabel = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, tracking: wide }) }
  style pgLabel = { paint: ink({ color: primary }), text: type({ family: mono, size: xs, weight: medium, tracking: wide }) }
  style backLink = { box: flow({ display: inline-flex }).space({ pad-x: 1_5 }).size({ h: control-h-sm }), paint: radius(md).ink({ color: muted-foreground }), text: type({ size: sm, weight: medium, wrap: nowrap }).underline(none), when: { hover: { paint: ink({ color: foreground }).fill({ color: hover-bg }) } } }
  style recipeSelect = { box: size({ min-w: 48, max-w: 64 }) }
  style descClamp = { box: size({ min-w: 0, max-w: 64 }).overflow({ all: hidden }), paint: ink({ color: muted-foreground }), text: type({ size: sm, wrap: nowrap }) }
  // Side-by-side pane: fill height of the split; console stays a fixed foot.
  style previewBand = { box: flow({ grow: 1 }).basis(0).size({ min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: border({ color: border, width: 1 }).radius(md).fill({ color: surface-1 }) }
  style previewBandMobile = { box: flow({ shrink: 0 }).size({ h: 40, min-h: 32 }).overflow({ all: hidden }), paint: border({ color: border, width: 1 }).radius(md).fill({ color: surface-1 }) }
  style previewBar = { box: flow({ shrink: 0 }).space({ pad-x: 2, pad-y: 1 }), paint: border({ color: border, width: 1, edge: b }) }
  style previewHost = { box: size({ w: full, h: full, min-h: 0 }).overflow({ all: auto }).space({ pad: 4 }), paint: fill({ color: surface-0 }) }
  style console = { box: flow({ shrink: 0 }).size({ h: 16, min-h: 16 }).space({ pad-x: 2, pad-y: 1 }).overflow({ y: auto }), paint: border({ color: border, width: 1, edge: t }).fill({ color: surface-0 }) }
  style consoleMobile = { box: flow({ shrink: 0 }).size({ h: 14, min-h: 14 }).space({ pad-x: 2, pad-y: 1 }).overflow({ y: auto }), paint: border({ color: border, width: 1, edge: t }).fill({ color: surface-0 }) }
  style monoContents = { box: flow({ display: contents }), text: type({ family: mono, size: xs }) }
  style ts = { paint: ink({ color: muted-foreground }).filter({ opacity: 50 }) }
  style logOk = { paint: ink({ color: success }) }
  style logWarn = { paint: ink({ color: warning }) }
  style logErr = { paint: ink({ color: destructive }) }
  style logInfo = { paint: ink({ color: muted-foreground }) }
  style logErrRow = { box: space({ pad: 1, pad-x: 2 }), paint: fill({ color: muted }).radius(sm) }
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
  // Opaque mount handle — capability firewall (no DOM surface in Pure).
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
  // signal.set treats a bare function as an updater — wrap so the CompileFn
  // itself is stored as the value, not invoked as the updater.
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
      idle: 'muted', typing: 'info', running: 'warning', ok: 'success', error: 'destructive',
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

  /** Push source into the signal AND the upgraded editor host (when ready). */
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

  // Seed editor when recipe changes or host appears. Do NOT read ed.value
  // (would subscribe this effect to the editor's valSig and re-enter).
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

  // Re-run when preview host / compile seam / recipe is ready.
  // Do NOT subscribe to srcValue — keystrokes are debounced via onSourceChange.
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
    {'success' => <row gap="2"><small style(ts)>{ts}</small><small style(logOk)>{msg}</small></row>}
    {'warn' => <row gap="2"><small style(ts)>{ts}</small><small style(logWarn)>{msg}</small></row>}
    {'error' => <row gap="2" style(logErrRow)><small style(ts)>{ts}</small><small style(logErr)>{msg}</small></row>}
    {else => <row gap="2"><small style(ts)>{ts}</small><small style(logInfo)>{msg}</small></row>}
  </match>
</partial>

<when {tablet}>
<column gap="0" style(workbench, playground)>
  <!-- One chrome bar: exit + recipe + theme + run. No site navbar, no SOURCE tax. -->
  <row align="center" justify="between" gap="2" style(toolbar) testid="pg-toolbar">
    <row gap="2" align="center" nowrap style({ box: size({ min-w: 0 }).flow({ grow: 1 }) })>
      <a href="/docs" label="Back to Docs" style(backLink) testid="pg-back">← Docs</a>
      <p style(pgLabel)>PLAYGROUND</p>
      <select
        size="sm"
        bind:value={recipeId}
        label="Recipe"
        testid="pg-recipe"
        style(recipeSelect)
      >
        <for {recipes as r} key={r.id}>
          <option value={r.id}>{r.title}</option>
        </for>
      </select>
      <small style(descClamp)>{recipeDesc()}</small>
    </row>
    <row gap="1" align="center" nowrap style({ box: flow({ shrink: 0 }) })>
      <theme-picker></theme-picker>
      <button variant="ghost" size="sm" on:click={reset}>↺ Reset</button>
      <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
    </row>
  </row>

  <row nowrap gap="1" align="stretch" style(split, { box: space({ pad: half }) })>
    <column gap="0" style(card)>
      <column gap="0" style(paneCol)>
        <sail-code-editor
          pane="source"
          language="sail"
          size="sm"
          chrome="flush"
          wasm-url="/sail_compiler_bg.wasm"
          use:captureEditor
          on-change={onSourceChange}
        ></sail-code-editor>
      </column>
    </column>

    <column gap="0" style(previewBand)>
      <row align="center" justify="between" gap="2" style(previewBar)>
        <row gap="2" align="center">
          <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
          <small style(countLabel)>PREVIEW · {recipeTitle()}</small>
        </row>
        <button variant="ghost" size="sm" on:click={clearConsole}>Clear console</button>
      </row>
      <column gap="0" style(paneCol)>
        <div
          testid="pg-preview"
          label="Playground preview"
          use:captureHost
          style(previewHost)
        ></div>
      </column>
      <column live="polite" label="Playground console" gap="0" style(console)>
        <span style(monoContents)>
          <for {logs() as line} key={line.id}>
            <log-line ts={line.ts} kind={line.kind} msg={line.msg}></log-line>
          </for>
        </span>
      </column>
    </column>
  </row>
</column>
{else =>
<!-- Phone: stacked editor over preview — still one chrome bar. -->
<column gap="0" style(workbenchMobile, playground)>
  <row align="center" justify="between" gap="2" style(toolbar) testid="pg-toolbar">
    <row gap="2" align="center" nowrap style({ box: size({ min-w: 0 }).flow({ grow: 1 }) })>
      <a href="/docs" label="Back to Docs" style(backLink) testid="pg-back">← Docs</a>
      <select size="sm" bind:value={recipeId} label="Recipe" testid="pg-recipe" style(recipeSelect)>
        <for {recipes as r} key={r.id}>
          <option value={r.id}>{r.title}</option>
        </for>
      </select>
    </row>
    <row gap="1" align="center" nowrap style({ box: flow({ shrink: 0 }) })>
      <theme-picker></theme-picker>
      <button variant="ghost" size="sm" on:click={reset}>↺</button>
      <button variant="outline" tone="primary" size="sm" on:click={run}>▶</button>
    </row>
  </row>
  <column gap="0" style(card, { box: flow({ grow: 1 }).size({ min-h: 0 }) })>
    <column gap="0" style(paneCol)>
      <sail-code-editor
        pane="source"
        language="sail"
        size="sm"
        chrome="flush"
        wasm-url="/sail_compiler_bg.wasm"
        use:captureEditor
        on-change={onSourceChange}
      ></sail-code-editor>
    </column>
  </column>
  <column gap="0" style(previewBandMobile)>
    <row align="center" justify="between" gap="2" style(previewBar)>
      <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
      <button variant="ghost" size="sm" on:click={clearConsole}>Clear</button>
    </row>
    <column gap="0" style(paneCol)>
      <div
        testid="pg-preview"
        label="Playground preview"
        use:captureHost
        style(previewHost)
      ></div>
    </column>
    <column live="polite" label="Playground console" gap="0" style(consoleMobile)>
      <span style(monoContents)>
        <for {logs() as line} key={line.id}>
          <log-line ts={line.ts} kind={line.kind} msg={line.msg}></log-line>
        </for>
      </span>
    </column>
  </column>
</column>}</when>
