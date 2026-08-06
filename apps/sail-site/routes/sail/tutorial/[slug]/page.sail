<!--
  /tutorial/[slug] — the interactive lesson. Two-pane split: the lesson (prose,
  progress stepper, bottom nav) on the LEFT; the playground (editor card with
  template/controller/diff tabs, live preview iframe, output console) on the
  RIGHT.

  Playground state — active tab, status badge, console lines, editor values —
  is signals in the <controller>; the template is a pure read. tutorial-boot
  owns the FFI seams only (wasm compile, iframe postMessage, theme observer);
  the editor's imperative API is reached through use: capture actions that set
  signal refs. The Sail docs segment layout renders /sail/tutorial/* lessons
  full-bleed (sail-docs-rail `workbench`).

  LANG-GAP (renders without these until the vocabulary grows):
  - column clamp: the lesson pane wants exactly minmax(0, 30rem); it uses
    grow:1 + max-w:panel-max-w-md;
  - transform/gradient/text-transform atoms: stepper hover scaleY, the radial
    preview tint, and uppercase-via-CSS (labels are written uppercase in text).
-->
<controller>
import type { Action, ControllerScope } from '@ship/sail-runtime'
import { bindHtml } from '@ship/sail-runtime/internal'
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

    // ── FFI refs: the editors, the preview iframe (via use: capture actions) ──
    const editorEl = signal<CodeEditorHost | null>(null)
    const iframeEl = signal<HTMLIFrameElement | null>(null)
    const captureEditor: Action<void> = (node) => {
      editorEl.set(node as CodeEditorHost)
      return { destroy: () => editorEl.set(null) }
    }
    const captureIframe: Action<void> = (node) => {
      iframeEl.set(node as HTMLIFrameElement)
      return { destroy: () => iframeEl.set(null) }
    }

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
    // signal.set treats a bare function as an updater — wrap so the CompileFn
    // itself (also a function) is stored as the value, not invoked as the updater.
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
    let lastSourceSlug = ''
    let lastPushedKey = ''
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
    // Source seed is independent of wasm / preview iframe. The e2e contract is
    // that sail-code-editor[pane]=source].value holds the lesson `.sail` as
    // soon as the editor upgrades — gating that on compile/iframe left the
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

    let lastPreviewSlug = ''
    effect(() => {
      const l = lesson()
      const iframe = iframeEl()
      if (!l || !iframe || !compile()) return
      if (l.slug === lastPreviewSlug) return
      lastPreviewSlug = l.slug
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
      captureEditor,
      captureIframe,
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
  Full-bleed workbench under site chrome. REBUILD-MARKER-3 h(fill) (not h(full)) — the shell
  is display:contents all the way up, so % height collapses; fill uses the
  chrome-offset calc. No tab-panel: its default padT and orientation stamp
  ate the editor. Tabs drive a signal; panes are plain grow columns.
-->
<!--
  tablet+: side-by-side workbench. Mobile (else / SSR default): stacked.
-->
<when {tablet}>
<row nowrap align="stretch" w="full" gap="0">
  style({ box: h(fill).minH(0).minW(0).overflow(hidden) })

  <!-- Lesson pane -->
  <column shrink>
    style({ box: w(panel-max-w-md).minW(0).maxW(panel-max-w-md).minH(0), paint: bg(surface-1).borderRWidth(1).borderR(border) })

    <column grow gap="3">
      style({ box: minH(0).overflowY(auto).overflowX(hidden).padX(5).padY(4) })

      <row align="center" justify="between" gap="2">
        <p>
          style({ paint: fg(primary), text: fontFamily(mono).fontSize(xs).fontWeight(medium).tracking(wide) })
TUTORIAL</p>
        <small>
          style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) })
{lessonNumLabel()} / {totalLessons()}</small>
      </row>

      <row gap="1" label="Lesson progress">
        <for {steps()} as="step" key={step.slug}>
          <match {step.state}>
            <case {'done'}>
              <a href={step.href} title={step.title} grow>
                style({ box: h(1), paint: radius(full).bg(primary) })
</a>
            </case><case {'current'}>
              <a href={step.href} title={step.title} current="step" grow>
                style({ box: h(1), paint: radius(full).bg(primary).opacity(50) })
</a>
            </case><default>
              <a href={step.href} title={step.title} grow>
                style({ box: h(1), paint: radius(full).bg(surface-2) })
</a>
          </default></match>
        </for>
      </row>

      <column gap="1">
        <h1>
          style({ text: fontSize(2xl).leading(tight) })
{title()}</h1>
        <p>
          style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })
{description()}</p>
      </column>

      <div use:prose={proseHtml()}>
        style({ box: w(full).minW(0).overflowX(hidden) })
</div>
    </column>

    <row align="center" justify="between" gap="2" shrink="0">
      style({ box: padY(2).padX(4), paint: borderTWidth(1).borderT(border).bg(surface-1) })

      <a href="/sail/tutorial">
        style({ box: display(inline-flex).padY(1_5).padX(3), paint: radius(md).fg(muted-foreground), text: fontSize(sm).fontWeight(semibold) }).when(hover, { paint: fg(foreground).bg(surface-2) })
← All lessons</a>
      <if {hasNext()}>
        <a href={nextHref()} variant="solid" tone="primary">{nextTitle()}</a>
      </if>
    </row>
  </column>

  <column grow gap="1">
    style({ box: basis(0).minW(0).minH(0).pad(1), paint: bg(surface-0) })

    <!-- Editor card — file tabs + actions on one thin bar -->
    <column grow gap="0">
      style({ box: basis(0).minH(0).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) })

      <row align="center" justify="between" gap="2" shrink="0">
        style({ box: padY(1).padX(2), paint: borderBWidth(1).borderB(border).bg(surface-1) })

        <tabs active="source" size="sm" on:sail-tab-change={onTabChange}>
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
      <column grow gap="0">
        style({ box: minH(0).overflow(hidden) })

        <if {tab() === 'source'}>
          <column grow gap="0">
            style({ box: minH(0).overflow(hidden) })

            <sail-code-editor
              pane="source"
              language="sail"
              size="sm"
              chrome="flush"
              wasm-url="/sail_compiler_bg.wasm"
              use:captureEditor
              on:sail-change={onSourceChange}
            ></sail-code-editor>
          </column>
        </if>
        <if {tab() === 'diff'}>
          <column grow gap="0">
            style({ box: minH(0).overflow(hidden) })

            <row align="center" gap="2" shrink="0">
              style({ box: padY(1).padX(2), paint: borderBWidth(1).borderB(border) })

              <small>
                style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) })
your code → solution · {fileName()}</small>
            </row>
            <!-- Absolute fill: CE display:contents + content-height flex min
                 let the LCS list blow past the viewport; lock a real scrollport. -->
            <column grow gap="0">
              style({ box: minH(0).position(relative).overflow(hidden) })

              <div>
                style({ box: position(absolute).inset(0).overflowY(auto).overflowX(auto).w(full) })

                <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
              </div>
            </column>
          </column>
        </if>
      </column>
    </column>

    <!-- Preview + console: fixed lower band; console is a real pane, not a 28px strip. -->
    <column shrink="0" gap="0">
      style({ box: h(56).minH(48).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) })

      <row align="center" justify="between" gap="2" shrink="0">
        style({ box: padY(1).padX(2), paint: borderBWidth(1).borderB(border) })

        <row gap="2" align="center">
          <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
          <small>
            style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).tracking(wide) })
PREVIEW</small>
        </row>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <column grow gap="0">
        style({ box: minH(0).overflow(hidden) })

        <iframe
          testid="tut-preview"
          base-src="/tut-host.html"
          title="Tutorial preview"
          sandbox="allow-scripts allow-same-origin"
          use:captureIframe

        >
          style({ box: w(full).h(full).minH(0), paint: borderWidth(0).bg(surface-0) })
</iframe>
      </column>
      <!-- h(16) ≈ 4rem ≈ 5 mono lines; was maxH(7)=28px under a gap-4 column. -->
      <column live="polite" label="Preview console" gap="0" shrink="0"
>
        style({ box: h(16).minH(16).padY(1).padX(2).overflowY(auto), paint: borderTWidth(1).borderT(border).bg(surface-0), text: fontFamily(mono).fontSize(xs) })

        <for {logs()} as="line" key={line.id}>
          <match {line.kind}>
            <case {'success'}>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(success) })
{line.msg}</small></row>
            </case><case {'warn'}>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(warning) })
{line.msg}</small></row>
            </case><case {'error'}>
              <row gap="2">
                style({ box: pad(1).padX(2), paint: bg(muted).radius(sm) })
<small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(destructive) })
{line.msg}</small></row>
            </case><default>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(muted-foreground) })
{line.msg}</small></row>
          </default></match>
        </for>
      </column>
    </column>
  </column>
</row>
<else>
<!-- Mobile: stacked — short lesson header + full-width playground -->
<column gap="0" w="full">
  style({ box: h(fill).minH(0).overflow(hidden) })

  <column shrink="0" gap="2">
    style({ box: padX(4).padY(3).maxH(40).overflowY(auto), paint: borderBWidth(1).borderB(border).bg(surface-1) })

    <row align="center" justify="between" gap="2">
      <p>
        style({ paint: fg(primary), text: fontFamily(mono).fontSize(xs).fontWeight(medium).tracking(wide) })
TUTORIAL · {lessonNumLabel()}/{totalLessons()}</p>
      <a href="/sail/tutorial">
        style({ paint: fg(muted-foreground), text: fontSize(xs) })
All lessons</a>
    </row>
    <h1>
      style({ text: fontSize(xl).leading(tight) })
{title()}</h1>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })
{description()}</p>
    <div use:prose={proseHtml()}>
      style({ box: w(full).minW(0).overflowX(hidden) })
</div>
  </column>
  <column grow gap="1">
    style({ box: basis(0).minH(0).minW(0).pad(1), paint: bg(surface-0) })

    <column grow gap="0">
      style({ box: basis(0).minH(0).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) })

      <row align="center" justify="between" gap="2" shrink="0">
        style({ box: padY(1).padX(2), paint: borderBWidth(1).borderB(border).bg(surface-1) })

        <tabs active="source" size="sm" on:sail-tab-change={onTabChange}>
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
      <column grow gap="0">
        style({ box: minH(0).overflow(hidden) })

        <if {tab() === 'source'}>
          <column grow gap="0">
            style({ box: minH(0).overflow(hidden) })

            <sail-code-editor
              pane="source"
              language="sail"
              size="sm"
              chrome="flush"
              wasm-url="/sail_compiler_bg.wasm"
              use:captureEditor
              on:sail-change={onSourceChange}
            ></sail-code-editor>
          </column>
        </if>
        <if {tab() === 'diff'}>
          <column grow gap="0">
            style({ box: minH(0).overflow(hidden) })

            <scroll-area grow>
              style({ box: minH(0).w(full) })

              <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
            </scroll-area>
          </column>
        </if>
      </column>
    </column>
    <column shrink="0" gap="0">
      style({ box: h(40).minH(32).overflow(hidden), paint: borderWidth(1).border(border).radius(md).bg(surface-1) })

      <row align="center" justify="between" gap="2" shrink="0">
        style({ box: padY(1).padX(2), paint: borderBWidth(1).borderB(border) })

        <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <column grow gap="0">
        style({ box: minH(0).overflow(hidden) })

        <iframe
          testid="tut-preview"
          base-src="/tut-host.html"
          title="Tutorial preview"
          sandbox="allow-scripts allow-same-origin"
          use:captureIframe

        >
          style({ box: w(full).h(full).minH(0), paint: borderWidth(0).bg(surface-0) })
</iframe>
      </column>
      <column live="polite" label="Preview console" gap="0" shrink="0"
>
        style({ box: h(14).minH(14).padY(1).padX(2).overflowY(auto), paint: borderTWidth(1).borderT(border).bg(surface-0), text: fontFamily(mono).fontSize(xs) })

        <for {logs()} as="line" key={line.id}>
          <match {line.kind}>
            <case {'success'}>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(success) })
{line.msg}</small></row>
            </case><case {'warn'}>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(warning) })
{line.msg}</small></row>
            </case><case {'error'}>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(destructive) })
{line.msg}</small></row>
            </case><default>
              <row gap="2"><small>
                style({ paint: fg(muted-foreground).opacity(50) })
{line.ts}</small><small>
                style({ paint: fg(muted-foreground) })
{line.msg}</small></row>
          </default></match>
        </for>
      </column>
    </column>
  </column>
</column>
</else></when>

