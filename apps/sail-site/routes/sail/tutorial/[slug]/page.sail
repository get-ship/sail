<!--
  /tutorial/[slug] — two-pane lesson: prose + stepper left; editor, preview,
  console right. Fill/overflow lives on <sail-fill-shell> (layout). Route is
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
import { bindHtml } from '@ship/sail-runtime/internal'
import { ensureWasm, makeCompile } from '../../../../src/compositions/live-sail-boot'
import type { Scope } from './page.sail'
import { TUTORIAL_LESSONS, type TutorialLesson } from '../../../../src/compositions/tutorial-data'
import {
  createDebouncer,
  runInHost,
  type CodeEditorHost,
  type CompileFn,
  type PreviewMount,
} from '../../../../src/compositions/live-sail-boot'

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

        const tab = signal<string>('source')
    const statusTone = signal<string>('muted')
    const statusText = signal<string>('loading…')
    const logs = signal<LogLine[]>([])
    const solutionShown = signal(false)
    const srcValue = signal('')

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
      const key = lastSourceSlug + '\0' + source.length + '\0' + source.slice(0, 32)
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

<partial log-line props: LogLineProps>
  <match {kind}>
    {'success' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {'warn' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {'error' => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
    {else => <row gap="2"><small>{ts}</small><small>{msg}</small></row>}
  </match>
</partial>

<when {tablet}>
<row nowrap align="stretch" gap="0">
  <column>
    <scroll-area>
      <column gap="3">
        <row align="center" justify="between" gap="2">
          <p>TUTORIAL</p>
          <small>{lessonNumLabel()} / {totalLessons()}</small>
        </row>

        <row gap="1" label="Lesson progress">
          <for {steps() as step} key={step.slug}>
            <match {step.state}>
              {'done' => <a href={step.href} title={step.title}><badge tone="primary" size="sm">·</badge></a>}
              {'current' => <a href={step.href} title={step.title} current="step"><badge tone="accent" size="sm">·</badge></a>}
              {else => <a href={step.href} title={step.title}><badge tone="muted" size="sm">·</badge></a>}
            </match>
          </for>
        </row>

        <column gap="1">
          <h1>{title()}</h1>
          <p>{description()}</p>
        </column>

        <div use:prose={proseHtml()}></div>
      </column>
    </scroll-area>

    <row nowrap> gap="2">
      <a href="/sail/tutorial">← All lessons</a>
      <if {hasNext()}>
        <a href={nextHref()} variant="solid" tone="primary">{nextTitle()}</a>
      </if>
    </row>
  </column>

  <column gap="1">
    <card padding="none">
      <row nowrap> gap="2">
        <tabs active="source" on-tab-change={onTabChange}>
          <tab-list variant="pill">
            <tab name="source" title={fileName()}></tab>
            <tab name="diff" title="diff"></tab>
          </tab-list>
        </tabs>
        <row gap="1" align="center">
          <button variant="ghost" size="sm" on:click={reset}>↺ Reset</button>
          <button variant="outline" tone="primary" size="sm" on:click={toggleSolution}>{solutionLabel()}</button>
        </row>
      </row>
      <if {tab() === 'source'}>
        <sail-code-editor
          pane="source"
          language="sail"
          size="sm"
          chrome="flush"
          wasm-url="/sail_compiler_bg.wasm"
          use:captureEditor
          on-change={onSourceChange}
        ></sail-code-editor>
      </if>
      <if {tab() === 'diff'}>
        <column gap="0">
          <row nowrap> gap="2">
            <small>your code → solution · {fileName()}</small>
          </row>
          <scroll-area>
            <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
          </scroll-area>
        </column>
      </if>
    </card>

    <column gap="0">
      <row nowrap> gap="2">
        <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
        <small>PREVIEW</small>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <card padding="sm">
        <div
          testid="tut-preview"
          label="Tutorial preview"
          use:captureHost
        ></div>
      </card>
      <card padding="sm" live="polite" label="Preview console">
        <for {logs() as entry} key={entry.id}>
          <log-line ts={entry.ts} kind={entry.kind} msg={entry.msg}></log-line>
        </for>
      </card>
    </column>
  </column>
</row>
{else =>
<column gap="2">
  <card padding="sm">
    <row align="center" justify="between" gap="2">
      <p>TUTORIAL · {lessonNumLabel()}/{totalLessons()}</p>
      <a href="/sail/tutorial">All lessons</a>
    </row>
    <h1>{title()}</h1>
    <p>{description()}</p>
    <div use:prose={proseHtml()}></div>
  </card>
  <column gap="1">
    <card padding="none">
      <row nowrap> gap="2">
        <tabs active="source" on-tab-change={onTabChange}>
          <tab-list variant="pill">
            <tab name="source" title={fileName()}></tab>
            <tab name="diff" title="diff"></tab>
          </tab-list>
        </tabs>
        <row gap="1" align="center">
          <button variant="ghost" size="sm" on:click={reset}>↺</button>
          <button variant="outline" tone="primary" size="sm" on:click={toggleSolution}>{solutionLabel()}</button>
        </row>
      </row>
      <if {tab() === 'source'}>
        <sail-code-editor
          pane="source"
          language="sail"
          size="sm"
          chrome="flush"
          wasm-url="/sail_compiler_bg.wasm"
          use:captureEditor
          on-change={onSourceChange}
        ></sail-code-editor>
      </if>
      <if {tab() === 'diff'}>
        <scroll-area>
          <sail-diff from={srcValue()} to={solvedSource()}></sail-diff>
        </scroll-area>
      </if>
    </card>
    <column gap="0">
      <row nowrap> gap="2">
        <badge tone={statusTone()} variant="soft" size="sm">{statusText()}</badge>
        <button variant="outline" tone="primary" size="sm" on:click={run}>▶ Run</button>
      </row>
      <card padding="sm">
        <div
          testid="tut-preview"
          label="Tutorial preview"
          use:captureHost
        ></div>
      </card>
      <card padding="sm" live="polite" label="Preview console">
        <for {logs() as entry} key={entry.id}>
          <log-line ts={entry.ts} kind={entry.kind} msg={entry.msg}></log-line>
        </for>
      </card>
    </column>
  </column>
</column>}</when>
