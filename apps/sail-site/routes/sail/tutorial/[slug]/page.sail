<!--
  /tutorial/[slug] — two-pane lesson: prose + stepper left; editor, preview,
  console right. Playground state is signals; tutorial-boot owns wasm + same-
  document preview mount; use: capture actions hold editor/host refs. Workbench
  chrome via sail-docs-rail. Language gaps: no column clamp atom (uses max-w
  panel token); no transform/gradient stepper polish.
-->
<controller>
import type { Action, ControllerScope } from '@ship/sail-runtime'
import { bindHtml } from '@ship/sail-runtime/internal'
import { ensureWasm, makeCompile } from '@ship/sail-viewer/wasm-bootstrap'
import type { Scope } from './page.sail'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../../src/compositions/tutorial-data'
import {
  createDebouncer,
  runInHost,
  type CodeEditorHost,
  type CompileFn,
  type PreviewMount,
} from '../../../../src/compositions/tutorial-boot'

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

        const editorEl = signal<CodeEditorHost | null>(null)
    const hostEl = signal<HTMLElement | null>(null)
    let previewMount: PreviewMount | null = null
    const captureEditor: Action<void> = (node) => {
      editorEl.set(node as CodeEditorHost)
      return { destroy: () => editorEl.set(null) }
    }
    const captureHost: Action<void> = (node) => {
      hostEl.set(node as HTMLElement)
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

        const tab = signal<string>('source')
    const statusTone = signal<string>('muted')
    const statusText = signal<string>('loading…')
    const logs = signal<LogLine[]>([])
    const solutionShown = signal(false)
    // ONE source: the lesson `.sail`, controller block and template together —
    // the file the student edits and the compiler compiles.
    const srcValue = signal('')

    const compile = signal<CompileFn | null>(null)
    // signal.set treats a bare function as an updater — wrap so the CompileFn
    // itself (also a function) is stored as the value, not invoked as the updater.
    ensureWasm('/sail_compiler_bg.wasm')
      .then(() => compile.set(() => makeCompile() as unknown as CompileFn))
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
    let lastSourceSlug = ''
    let lastPushedKey = ''
    const schedule = createDebouncer(200)

    const runNow = (): void => {
      const c = compile()
      const host = hostEl()
      if (!c || !host) return
      setStatus('running', 'compiling…')
      const safeSlug = (lesson()?.slug || 'lesson').replace(/[^a-z0-9-]/gi, '')
      const { result, mount, ready } = runInHost(
        c,
        host,
        {
          tag: `tut-${safeSlug}-${++tagSeq}`,
          source: srcValue(),
        },
        previewMount,
      )
      previewMount = mount
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
      // Inline module mount is async under CSP; settle status when the host paints.
      if (ready) {
        void ready
          .then((m) => {
            previewMount = m
            setStatus('ok', 'running')
            log('▶ ran ' + fileName(), 'success')
          })
          .catch((err: unknown) => {
            const message = err instanceof Error ? err.message : String(err)
            setStatus('error', 'compile error')
            log('mount: ' + message, 'error')
          })
      } else {
        setStatus('ok', 'running')
        log('▶ ran ' + fileName(), 'success')
      }
    }

    /** Push source into the signal AND the upgraded editor host (when ready). */
    const applyCode = (source: string): void => {
      srcValue.set(source)
      lastPushedKey = ''
      const e = editorEl()
      if (e && typeof e.setDiagnostics === 'function') {
        e.value = source
        lastPushedKey = lastSourceSlug + '\0' + source.length + '\0' + source.slice(0, 32)
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
    const onTabChange = (detail: { name: string }): void => {
      const name = detail?.name
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

        // Source seed is independent of wasm / preview host. The e2e contract is
    // that sail-code-editor[pane]=source].value holds the lesson `.sail` as
    // soon as the editor upgrades — gating that on compile/host left the
    // editor empty whenever the host ref raced the one-shot write, or when
    // wasm was still loading.
    //
    // Do NOT read `ed.value` inside this effect: the host getter falls through
    // to the editor's valSig, which would subscribe this effect to the editor
    // and re-enter on every programmatic write (main-thread hang).
    effect(() => {
      const l = lesson()
      if (!l) return
      if (l.slug !== lastSourceSlug) {
        lastSourceSlug = l.slug
        solutionShown.set(false)
        srcValue.set(l.starterSource)
        lastPushedKey = ''
        logs.set([])
        log(`loaded ${l.slug}`, 'info')
      }
      const ed = editorEl()
      const source = srcValue()
      if (!ed || !source) return
      // Re-push when the host ref appears or the lesson source changes — never
      // when the editor's own valSig ticks (user typing / CM internal sync).
      const key = lastSourceSlug + '\0' + source.length + '\0' + source.slice(0, 32)
      if (key === lastPushedKey) return
      // captureEditor can fire before the CE's connectedCallback publishes host.value
      // (defineProperty). Writing then is a dead expando that mount overwrites with
      // ''. Wait until setDiagnostics exists — that runs in the same mount() as the
      // value property — then assign and mark pushed.
      const push = (): void => {
        if (editorEl() !== ed) return
        if (typeof ed.setDiagnostics !== 'function') {
          requestAnimationFrame(push)
          return
        }
        ed.value = source
        lastPushedKey = key
      }
      push()
    })

    // Re-run when the preview host, compile seam, or lesson slug is ready.
    // Do NOT subscribe to srcValue here — keystrokes are debounced via
    // onSourceChange → schedule(runNow); applyCode / run call runNow directly.
    let lastPreviewSlug = ''
    effect(() => {
      const l = lesson()
      const host = hostEl()
      const c = compile()
      if (!l || !host || !c) return
      if (l.slug === lastPreviewSlug && previewMount) return
      lastPreviewSlug = l.slug
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
      captureEditor,
      captureHost,
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

<!--
  h(fill) not h(full): shell is display:contents up the tree, so % height collapses.
  Tabs drive a signal; panes are plain grow columns (tab-panel pad ate the editor).
  tablet+: side-by-side; mobile/SSR else: stacked.
-->

<scope>
  style workbench = { box: w(full).h(fill).minH(0).minW(0).overflow(hidden) }
  style workbenchMobile = { box: w(full).h(fill).minH(0).overflow(hidden) }
  style lessonPane = { box: shrink(1).w(panel-max-w-md).minW(0).maxW(panel-max-w-md).minH(0), paint: bg(surface-1).borderRWidth(1).borderR(border) }
  style lessonScroll = { box: grow(1).minH(0).overflowY(auto).overflowX(hidden).padX(5).padY(4) }
  style tutLabel = { paint: fg(primary), text: fontFamily(mono).fontSize(xs).fontWeight(medium).tracking(wide) }
  style countLabel = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) }
  style stepDone = { box: grow(1).h(1), paint: radius(full).bg(primary) }
  style stepCurrent = { box: grow(1).h(1), paint: radius(full).bg(primary).opacity(50) }
  style stepTodo = { box: grow(1).h(1), paint: radius(full).bg(surface-2) }
  style title = { text: fontSize(2xl).leading(tight) }
  style titleSm = { text: fontSize(xl).leading(tight) }
  style desc = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style descSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style proseHost = { box: w(full).minW(0).overflowX(hidden) }
  style lessonFoot = { box: shrink(0).padY(2).padX(4), paint: borderTWidth(1).borderT(border).bg(surface-1) }
  style allLessons = { box: display(inline-flex).padY(1_5).padX(3), paint: radius(md).fg(muted-foreground), text: fontSize(sm).fontWeight(semibold), when: { hover: { paint: fg(foreground).bg(surface-2) } } }
  style allLessonsSm = { paint: fg(muted-foreground), text: fontSize(xs) }
  style playground = { box: grow(1).basis(0).minW(0).minH(0).pad(1), paint: bg(surface-0) }
  style card = { box: grow(1).basis(0).minH(0).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) }
  style cardBar = { box: shrink(0).padY(1).padX(2), paint: borderBWidth(1).borderB(border).bg(surface-1) }
  style paneCol = { box: grow(1).minH(0).overflow(hidden) }
  style diffBar = { box: shrink(0).padY(1).padX(2), paint: borderBWidth(1).borderB(border) }
  style monoXs = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) }
  style diffShell = { box: grow(1).minH(0).position(relative).overflow(hidden) }
  style diffAbs = { box: position(absolute).inset(0).overflowY(auto).overflowX(auto).w(full) }
  style previewBand = { box: shrink(0).h(56).minH(48).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) }
  style previewBandMobile = { box: shrink(0).h(40).minH(32).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) }
  style previewBar = { box: shrink(0).padY(1).padX(2), paint: borderBWidth(1).borderB(border) }
  style previewHost = { box: w(full).h(full).minH(0).overflow(auto).pad(4), paint: bg(surface-0) }
  style console = { box: shrink(0).h(16).minH(16).padY(1).padX(2).overflowY(auto), paint: borderTWidth(1).borderT(border).bg(surface-0) }
  style consoleMobile = { box: shrink(0).h(14).minH(14).padY(1).padX(2).overflowY(auto), paint: borderTWidth(1).borderT(border).bg(surface-0) }
  style monoContents = { box: display(contents), text: fontFamily(mono).fontSize(xs) }
  style ts = { paint: fg(muted-foreground).opacity(50) }
  style logOk = { paint: fg(success) }
  style logWarn = { paint: fg(warning) }
  style logErr = { paint: fg(destructive) }
  style logInfo = { paint: fg(muted-foreground) }
  style logErrRow = { box: pad(1).padX(2), paint: bg(muted).radius(sm) }
  style mobileHead = { box: shrink(0).padX(4).padY(3).maxH(40).overflowY(auto), paint: borderBWidth(1).borderB(border).bg(surface-1) }
  style growMin = { box: grow(1).minH(0).overflow(hidden) }

  type LogLineProps = {
    ts: string
    kind: string
    msg: string
  }
</scope>

<partial log-line props: LogLineProps>
  <match {kind}>
    {'success' => <row gap="2"><small style(ts)>{ts}</small><small style(logOk)>{msg}</small></row>}
    {'warn' => <row gap="2"><small style(ts)>{ts}</small><small style(logWarn)>{msg}</small></row>}
    {'error' => <row gap="2" style(logErrRow)><small style(ts)>{ts}</small><small style(logErr)>{msg}</small></row>}
    {else => <row gap="2"><small style(ts)>{ts}</small><small style(logInfo)>{msg}</small></row>}
  </match>
</partial>

<when {tablet}>
<row nowrap align="stretch" gap="0" style(workbench)>
  <!-- Lesson pane -->
  <column style(lessonPane)>
    <column gap="3" style(lessonScroll)>
      <row align="center" justify="between" gap="2">
        <p style(tutLabel)>TUTORIAL</p>
        <small style(countLabel)>{lessonNumLabel()} / {totalLessons()}</small>
      </row>

      <row gap="1" label="Lesson progress">
        <for {steps() as step} key={step.slug}>
          <match {step.state}>
            {'done' => <a href={step.href} title={step.title} style(stepDone)></a>}{'current' => <a href={step.href} title={step.title} current="step" style(stepCurrent)></a>}{else => <a href={step.href} title={step.title} style(stepTodo)></a>}</match>
        </for>
      </row>

      <column gap="1">
        <h1 style(title)>{title()}</h1>
        <p style(desc)>{description()}</p>
      </column>

      <div use:prose={proseHtml()} style(proseHost)></div>
    </column>

    <row align="center" justify="between" gap="2" style(lessonFoot)>
      <a href="/sail/tutorial" style(allLessons)>← All lessons</a>
      <if {hasNext()}>
        <a href={nextHref()} variant="solid" tone="primary">{nextTitle()}</a>
      </if>
    </row>
  </column>

  <column gap="1" style(playground)>
    <!-- Editor card — file tabs + actions on one thin bar -->
    <column gap="0" style(card)>
      <row align="center" justify="between" gap="2" style(cardBar)>
        <tabs active="source" on-tab-change={onTabChange}>
          <tab-list variant="pill">
            <tab name="source">{fileName()}</tab>
            <tab name="diff">diff</tab>
          </tab-list>
        </tabs>
        <row gap="1" align="center">
          <button variant="ghost" size="sm" on:click={reset}>↺ Reset</button>
          <button variant="outline" tone="primary" size="sm" on:click={toggleSolution}>{solutionLabel()}</button>
        </row>
      </row>
      <!--
        One pane at a time via if (not hidden=). Boolean hidden="false" is still
        present in HTML and keeps display:none — both panes fought for grow and
        collapsed the editor to ~0. Remount re-pushes via the existing editor
        effect when captureEditor fires again.
      -->
      <column gap="0" style(paneCol)>
        <if {tab() === 'source'}>
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
        </if>
        <if {tab() === 'diff'}>
          <column gap="0" style(paneCol)>
            <row align="center" gap="2" style(diffBar)>
              <small style(countLabel)>your code → solution · {fileName()}</small>
            </row>
            <!-- Absolute fill: CE display:contents + content-height flex min
                 let the LCS list blow past the viewport; lock a real scrollport. -->
            <column gap="0" style(diffShell)>
              <div style(diffAbs)>
                <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
              </div>
            </column>
          </column>
        </if>
      </column>
    </column>

    <!-- Preview + console: fixed lower band; console is a real pane, not a 28px strip. -->
    <column gap="0" style(previewBand)>
      <row align="center" justify="between" gap="2" style(diffBar)>
        <row gap="2" align="center">
          <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
          <small style(countLabel)>PREVIEW</small>
        </row>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <column gap="0" style(paneCol)>
        <div
          testid="tut-preview"
          label="Tutorial preview"
          use:captureHost style(previewHost)></div>
      </column>
      <!-- h(16) ≈ 4rem ≈ 5 mono lines; was maxH(7)=28px under a gap-4 column. -->
      <column live="polite" label="Preview console" gap="0" style(console)>
        <span style(monoContents)>
          <for {logs() as line} key={line.id}>
            <log-line ts={line.ts} kind={line.kind} msg={line.msg}></log-line>
          </for>
        </span>
      </column>
    </column>
  </column>
</row>
{else => <!-- Mobile: stacked — short lesson header + full-width playground -->
<column gap="0" style(workbenchMobile)>
  <column gap="2" style(mobileHead)>
    <row align="center" justify="between" gap="2">
      <p style(tutLabel)>TUTORIAL · {lessonNumLabel()}/{totalLessons()}</p>
      <a href="/sail/tutorial" style(allLessonsSm)>All lessons</a>
    </row>
    <h1 style(titleSm)>{title()}</h1>
    <p style(descSm)>{description()}</p>
    <div use:prose={proseHtml()} style(proseHost)></div>
  </column>
  <column gap="1" style(playground)>
    <column gap="0" style(card)>
      <row align="center" justify="between" gap="2" style(cardBar)>
        <tabs active="source" on-tab-change={onTabChange}>
          <tab-list variant="pill">
            <tab name="source">{fileName()}</tab>
            <tab name="diff">diff</tab>
          </tab-list>
        </tabs>
        <row gap="1" align="center">
          <button variant="ghost" size="sm" on:click={reset}>↺</button>
          <button variant="outline" tone="primary" size="sm" on:click={toggleSolution}>{solutionLabel()}</button>
        </row>
      </row>
      <column gap="0" style(paneCol)>
        <if {tab() === 'source'}>
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
        </if>
        <if {tab() === 'diff'}>
          <column gap="0" style(paneCol)>
            <scroll-area style(paneCol)>
              <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
            </scroll-area>
          </column>
        </if>
      </column>
    </column>
    <column gap="0" style(previewBandMobile)>
      <row align="center" justify="between" gap="2" style(diffBar)>
        <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <column gap="0" style(paneCol)>
        <div
          testid="tut-preview"
          label="Tutorial preview"
          use:captureHost style(previewHost)></div>
      </column>
      <column live="polite" label="Preview console" gap="0" style(consoleMobile)>
        <span style(monoContents)>
          <for {logs() as line} key={line.id}>
            <log-line ts={line.ts} kind={line.kind} msg={line.msg}></log-line>
          </for>
        </span>
      </column>
    </column>
  </column>
</column>}</when>
