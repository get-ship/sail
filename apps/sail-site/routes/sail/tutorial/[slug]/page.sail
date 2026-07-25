<!--
  /tutorial/[slug] — the interactive lesson. Two-pane split: the lesson (prose,
  progress stepper, bottom nav) on the LEFT; the playground (editor card with
  template/controller/diff tabs, live preview iframe, output console) on the
  RIGHT.

  Playground state — active tab, status badge, console lines, editor values —
  is signals in the <controller>; the template is a pure read. tutorial-boot
  owns the FFI seams only (wasm compile, iframe postMessage, theme observer);
  the editor's imperative API is reached through bind:this refs. The Sail
  docs segment layout renders /sail/tutorial/* lessons full-bleed
  (sail-docs-rail `workbench`).

  LANG-GAP (renders without these until the vocabulary grows):
  - column clamp: the lesson pane wants exactly minmax(0, 30rem); it uses
    grow:1 + max-w:panel-max-w-md;
  - transform/gradient/text-transform atoms: stepper hover scaleY, the radial
    preview tint, and uppercase-via-CSS (labels are written uppercase in text).
-->
<controller>
import {
  controller,
  derived,
  effect,
  signal,
  bindHtml,
  type Action,
  type ControllerScope,
} from '@ship/sail-runtime'
import { ensureWasm, makeCompile } from '@ship/sail-viewer/wasm-bootstrap'
import type { Scope } from './page.sail'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../../src/tutorial-data'
import {
  createDebouncer,
  listenPreview,
  observeTheme,
  pushScenario,
  reloadPreview,
  runInPreview,
  type CodeEditorHost,
  type CompileFn,
} from '../../../../src/tutorial-boot'

const BY_SLUG: Record<string, TutorialLesson> = {}
for (const l of TUTORIAL_LESSONS) BY_SLUG[l.slug] = l
const ORDERED_LESSONS: TutorialLesson[] = [...TUTORIAL_LESSONS].sort((a, b) => a.order - b.order)

interface StepSegment {
  slug: string
  href: string
  state: 'done' | 'current' | 'todo'
  title: string
}

interface LogLine {
  id: number
  ts: string
  kind: 'info' | 'success' | 'warn' | 'error'
  msg: string
}

// The prose is TRUSTED, pre-rendered HTML (gen-tutorial.mjs converts the
// lesson markdown at build time); bindHtml is the runtime's one blob seam.
const prose: Action<string> = (node, html) => {
  const content = signal(html ?? '')
  const dispose = bindHtml(node, () => content())
  return {
    update(next: string): void {
      content.set(next ?? '')
    },
    destroy: dispose,
  }
}

export default controller((ctx: ControllerScope<Scope>) => {
    const lesson = derived(() => BY_SLUG[ctx.route.params.slug() ?? ''])
    const nextSlug = derived(() => lesson()?.next ?? '')

    // ── FFI refs: the editors, the preview iframe ────────────────────────
    const editorEl = signal<CodeEditorHost | null>(null)
    const iframeEl = signal<HTMLIFrameElement | null>(null)

    // ── Playground state — signals the template reads ────────────────────
    const tab = signal<string>('source')
    const statusTone = signal<string>('muted')
    const statusText = signal<string>('loading…')
    const logs = signal<LogLine[]>([])
    const solutionShown = signal(false)
    // ONE source: the lesson `.sail`, controller block and template together —
    // the file the student edits and the compiler compiles.
    const srcValue = signal('')

    const compile = signal<CompileFn | null>(null)
    ensureWasm('/sail_compiler_bg.wasm')
      .then(() => compile.set(() => makeCompile() as unknown as CompileFn))
      .catch((err) => console.error('[sail-site] wasm compile seam failed:', err))

    // ── Console log (elapsed-stamped, capped) ─────────────────────────────
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

    // ── Run pipeline ──────────────────────────────────────────────────────
    let tagSeq = 0
    let iframeReady = false
    const schedule = createDebouncer(200)

    const runNow = (): void => {
      const c = compile()
      const iframe = iframeEl()
      if (!c || !iframe || !iframeReady) return
      setStatus('running', 'compiling…')
      const safeSlug = (lesson()?.slug || 'lesson').replace(/[^a-z0-9-]/gi, '')
      const result = runInPreview(c, iframe, {
        tag: `tut-${safeSlug}-${++tagSeq}`,
        source: srcValue(),
      })
      if (result.kind === 'err') {
        setStatus('error', 'compile error')
        // The full compiler message lands in the console, destructive-toned, and
        // its line/column are the editor's own — one file, one coordinate space.
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
      setStatus('ok', 'running')
      log('▶ ran ' + fileName(), 'success')
    }

    const applyCode = (source: string): void => {
      srcValue.set(source)
      const e = editorEl()
      if (e) e.value = source
      runNow()
    }

    // ── Template handlers ────────────────────────────────────────────────
    const onSourceChange = (e: Event): void => {
      const v = (e as CustomEvent<{ value: string }>).detail?.value
      if (typeof v !== 'string') return
      srcValue.set(v)
      setStatus('typing', 'editing…')
      schedule(runNow)
    }
    const onTabChange = (e: Event): void => {
      const name = (e as CustomEvent<{ name: string }>).detail?.name
      if (name) tab.set(name)
    }
    const reset = (): void => {
      const l = lesson()
      if (!l) return
      solutionShown.set(false)
      applyCode(l.starterSource)
      log('↺ reset to starter', 'info')
    }
    const toggleSolution = (): void => {
      const l = lesson()
      if (!l) return
      const next = !solutionShown()
      solutionShown.set(next)
      if (next) {
        applyCode(l.solvedSource)
        log('shown solution', 'info')
      } else {
        applyCode(l.starterSource)
        log('hid solution — back to your code', 'info')
      }
    }
    const run = (): void => {
      log('▶ run', 'info')
      runNow()
    }

    // ── Lesson lifecycle ─────────────────────────────────────────────────
    let lastSlug = ''
    effect(() => {
      const l = lesson()
      const iframe = iframeEl()
      if (!l || !iframe || !compile()) return
      if (l.slug === lastSlug) return
      lastSlug = l.slug
      solutionShown.set(false)
      srcValue.set(l.starterSource)
      const ed = editorEl()
      if (ed) ed.value = l.starterSource
      logs.set([])
      log(`loaded ${l.slug}`, 'info')
      iframeReady = false
      reloadPreview(iframe)
    })

    const disposePreview = listenPreview(iframeEl, {
      onConsole: (level, text) => {
        log(`[preview] ${text}`, level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'info')
      },
      onReady: () => {
        const l = lesson()
        const go = (): void => {
          iframeReady = true
          runNow()
        }
        if (l) void pushScenario(iframeEl()!, l.slug).then(go)
        else go()
      },
    })
    const disposeTheme = observeTheme(() => {
      const iframe = iframeEl()
      if (iframe) {
        iframeReady = false
        reloadPreview(iframe)
      }
    })
    effect(() => () => {
      disposePreview()
      disposeTheme()
    })

    // ── Presentation reads ───────────────────────────────────────────────
    // The lesson's real filename — the tab is the file, so it says so.
    const fileName = derived(() => (lesson()?.starterStem ?? 'lesson') + '.sail')

    const steps = derived((): StepSegment[] => {
      const cur = lesson()?.order ?? 0
      return ORDERED_LESSONS.map((l) => ({
        slug: l.slug,
        href: '/sail/tutorial/' + l.slug,
        state: l.order < cur ? 'done' : l.order === cur ? 'current' : 'todo',
        title: 'Lesson ' + l.order + ' — ' + l.title,
      }))
    })

    return {
      tab,
      statusTone,
      statusText,
      logs,
      solutionShown,
      srcValue,
      fileName,
      editorEl,
      iframeEl,
      prose,
      steps,
      onSourceChange,
      onTabChange,
      reset,
      toggleSolution,
      run,
      title: derived(() => lesson()?.title ?? 'Lesson not found'),
      description: derived(() => lesson()?.description ?? ''),
      proseHtml: derived(() => lesson()?.proseHtml ?? ''),
      solvedSource: derived(() => lesson()?.solvedSource ?? ''),
      solutionLabel: derived(() => (solutionShown() ? 'Hide solution' : 'Show solution')),
      lessonNumLabel: derived(() => pad2(lesson()?.order ?? 0)),
      totalLessons: derived(() => ORDERED_LESSONS.length),
      hasNext: derived(() => Boolean(nextSlug())),
      nextHref: derived(() => (nextSlug() ? '/sail/tutorial/' + nextSlug() : '/sail/tutorial')),
      nextTitle: derived(() => (BY_SLUG[nextSlug()]?.title ?? '') + ' →'),
    }
})
</controller>

<row sx={style.h(fill).minH(0).overflow(hidden)} align="stretch">
  <!-- basis-0 on both panes: the wrapping <row> floor would otherwise stack
       the playground BELOW the lesson (each pane demanding its content width);
       a 0 main-basis keeps them side by side, grow distributes the width. -->
  <column sx={style.grow(1).basis(0).minW(0).maxW(panel-max-w-md).h(fill).minH(0).bg(surface-1).borderRWidth(1).borderR(border)}>
    <column sx={style.grow(1).minH(0).overflowY(auto).pad(6)} gap="4">
      <row align="center" justify="between" gap="3">
        <p sx={style.fontFamily(mono).fontSize(xs).fontWeight(medium).tracking(wide).fg(primary)}>— TUTORIAL</p>
        <small sx={style.fontFamily(mono).fontSize(xs).tracking(wide).fg(muted-foreground)}>Lesson {lessonNumLabel()} / {totalLessons()}</small>
      </row>

      <row gap="1" aria-label="Lesson progress">
        {#for step of steps() key={step.slug}}
          {#match step.state}
            {:case 'done'}
              <a href={step.href} title={step.title} sx={style.grow(1).h(1).radius(full).bg(primary)}></a>
            {:case 'current'}
              <a href={step.href} title={step.title} aria-current="step" sx={style.grow(1).h(1).radius(full).bg(primary).opacity(50)}></a>
            {:default}
              <a href={step.href} title={step.title} sx={style.grow(1).h(1).radius(full).bg(surface-2)}></a>
          {/match}
        {/for}
      </row>

      <column gap="2">
        <h1>{title()}</h1>
        <p sx={style.fontSize(lg).fg(muted-foreground)}>{description()}</p>
      </column>

      <div sx={style.maxW(breakpoint-tablet).mx(auto)} use:prose={proseHtml()}></div>
    </column>

    <row align="center" justify="between" gap="3" sx={style.pad(3).padX(6).borderTWidth(1).borderT(border).bg(surface-1)}>
      <a href="/sail/tutorial" sx={style.pad(2).padX(4).radius(md).fontSize(sm).fontWeight(semibold).fg(muted-foreground).when({ hover: style.fg(foreground).bg(surface-2) })}>← All lessons</a>
      {#if hasNext()}
        <a href={nextHref()} sx={style.pad(2).padX(4).radius(md).fontSize(sm).fontWeight(semibold).bg(primary).fg(primary-foreground).overflow(hidden)}>{nextTitle()}</a>
      {/if}
    </row>
  </column>

  <column sx={style.grow(1).basis(0).minW(0).h(fill).minH(0).pad(4).bg(surface-0)} gap="3">
    <row align="center" justify="between" gap="3">
      <p sx={style.fontFamily(mono).fontSize(xs).fontWeight(medium).tracking(wide).fg(muted-foreground)}>— PLAYGROUND</p>
      <row gap="2" align="center">
        <button variant="ghost" size="sm" on:click={reset}>↺ Reset</button>
        <button variant="outline" tone="primary" size="sm" on:click={toggleSolution}>{solutionLabel()}</button>
      </row>
    </row>

    <column sx={style.grow(1).minH(0).borderWidth(1).border(border).radius(lg).bg(surface-1).overflow(hidden)}>
      <row align="center" justify="between" gap="2" sx={style.pad(2).padX(3).borderBWidth(1).borderB(border)}>
        <tabs active="source" size="sm" on:sail-tab-change={onTabChange}>
          <tab-list variant="pill">
            <tab name="source">{fileName()}</tab>
            <tab name="diff">diff</tab>
          </tab-list>
        </tabs>
      </row>
      <column sx={style.grow(1).minH(0).position(relative)}>
        <tab-panel name="source" hidden={tab() !== 'source'} sx={style.grow(1).minH(0)}>
          <sail-code-editor
            data-pane="source"
            language="sail"
            size="md"
            chrome="flush"
            bind:this={editorEl}
            on:sail-change={onSourceChange}
          ></sail-code-editor>
        </tab-panel>
        <tab-panel name="diff" hidden={tab() !== 'diff'} sx={style.grow(1).minH(0).overflowY(auto)}>
          <column gap="3" sx={style.pad(3)}>
            <small sx={style.fontFamily(mono).fontSize(xs).tracking(wide).fg(muted-foreground)}>{fileName()}</small>
            <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
          </column>
        </tab-panel>
      </column>
    </column>

    <column sx={style.grow(1).minH(0).borderWidth(1).border(border).radius(lg).bg(surface-1).overflow(hidden)}>
      <row align="center" justify="between" gap="2" sx={style.pad(2).padX(3).borderBWidth(1).borderB(border)}>
        <row gap="2" align="center">
          <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
          <small sx={style.fontFamily(mono).fontSize(xs).tracking(wide).fg(muted-foreground)}>PREVIEW · LIVE</small>
        </row>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <column sx={style.grow(1).minH(0)}>
        <iframe
          data-base-src="/tut-host.html"
          title="Tutorial preview"
          sandbox="allow-scripts allow-same-origin"
          bind:this={iframeEl}
          sx={style.w(full).h(full).borderWidth(0).bg(surface-0)}
        ></iframe>
      </column>
      <column role="log" aria-live="polite" aria-label="Preview console" gap="0"
              sx={style.borderTWidth(1).borderT(border).bg(surface-0).pad(2).padX(3).overflowY(auto).maxH(9).fontFamily(mono).fontSize(sm)}>
        {#for line of logs() key={line.id}}
          {#match line.kind}
            {:case 'success'}
              <row gap="2"><small sx={style.fg(muted-foreground).opacity(50)}>{line.ts}</small><small sx={style.fg(success)}>{line.msg}</small></row>
            {:case 'warn'}
              <row gap="2"><small sx={style.fg(muted-foreground).opacity(50)}>{line.ts}</small><small sx={style.fg(warning)}>{line.msg}</small></row>
            {:case 'error'}
              <row gap="2" sx={style.bg(muted).radius(sm).pad(1).padX(2)}><small sx={style.fg(muted-foreground).opacity(50)}>{line.ts}</small><small sx={style.fg(destructive)}>{line.msg}</small></row>
            {:default}
              <row gap="2"><small sx={style.fg(muted-foreground).opacity(50)}>{line.ts}</small><small sx={style.fg(muted-foreground)}>{line.msg}</small></row>
          {/match}
        {/for}
      </column>
    </column>
  </column>
</row>
