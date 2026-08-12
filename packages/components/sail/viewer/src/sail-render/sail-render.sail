<!--
  <sail-render> — live mount stage for a SAIL component (source or tag mode).
  Engine work in engine/*.ts; template hosts stage via use:captureStage + error-boundary.
  Compile/source/props arrive via host identity — not JSON attrs.
-->

<scope>
  props: {
    mode: 'source' | 'tag' = 'source'
    'component-name' = ''
    'viewport-width': 'full' | 'desktop' | 'tablet' | 'mobile' = 'full'
  }
</scope>

<controller>
// Lifecycle only — compile+mount+dispose; DOM construction stays in engine/*.ts.

import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-render.sail'
import { attachEngineShadow, mountBySource, type ViewMount } from './engine/mount-source'
import { mountByTag, type TagMount } from './engine/mount-tag'
import { formatCompileError, errorKey, type CompileDiagnostic } from './engine/compile-error'
import type { CompileFn, WasmCompileResult } from '../wasm-bootstrap'

type ErrPhase = 'compile' | 'mount' | 'runtime'


function diagFromResult(r: Extract<WasmCompileResult, { kind: 'err' }>): CompileDiagnostic {
  return { message: r.message, line: r.line, col: r.col, category: r.category }
}
function diagFromError(e: unknown): CompileDiagnostic {
  const message = e instanceof Error ? e.message : String(e)
  const m = /\((\d+):(\d+)\)/.exec(message)
  return { message, line: m ? Number(m[1]) : null, col: m ? Number(m[2]) : null, category: 'compile' }
}

export default controller((ctx: ControllerScope<Scope>) => {
    const stageEl = signal<HostBag | null>(null)
    const captureStage: Action<void> = (node) => {
      stageEl.set(node)
      return { destroy: () => stageEl.set(null) }
    }

    const viewportWidth = (ctx as { viewportWidth?: () => string }).viewportWidth
    const stageMaxWidth = (): string | null => {
      const w = viewportWidth ? viewportWidth() : 'full'
      return w === 'full' || !w ? null : `var(--breakpoint-${w})`
    }
    const compiling = signal(false)
    const compileError = signal<ReturnType<typeof formatCompileError> | null>(null)
    const runtimeError = signal<string | null>(null)

    const compileErrorSummary = signal('')
    const compileErrorBody = signal('')
    mount((host) => {

      attachEngineShadow(host)


      const compile = signal<CompileFn | null>(null)
      const source = signal<string | null>(null)
      const propValues = signal<Record<string, unknown> | null>(null)
      const setupFn = signal<unknown>(null)
      const setupUrl = signal<string | null>(null)
      const slots = signal<string | null>(null)

      // signal.set(fn) treats fn as updater — wrap so function-valued props store as values.
      type AnySig = { (): unknown; set: (u: (p: unknown) => unknown) => void }
      const defineHostProp = (name: string, sig: { (): unknown; set: (v: never) => void }): void => {
        const s = sig as AnySig
        const existing = (host as Record<string, unknown>)[name]
        if (existing !== undefined) s.set(() => existing)
        Object.defineProperty(host, name, {
          configurable: true,
          get: () => s(),
          set: (v: unknown) => s.set(() => v),
        })
      }
      defineHostProp('compile', compile)
      defineHostProp('source', source)
      defineHostProp('propValues', propValues)
      defineHostProp('setup', setupFn)
      defineHostProp('setupUrl', setupUrl)
      defineHostProp('slots', slots)


      let viewMount: ViewMount | null = null
      let tagMount: TagMount | null = null
      let lastViewKey = '' // source identity that produced the last-good view mount
      let lastTagSource = '' // source the current tag node was defined from
      let lastErrorKey = '' // sail-render:error dedup (lastEmitted)

      const disposeView = (): void => {
        if (viewMount) {
          try {
            viewMount.dispose()
          } catch {
            /* dispose is try-wrapped */
          }
          viewMount = null
        }
      }
      const disposeTag = (): void => {
        if (tagMount) {
          try {
            tagMount.dispose()
          } catch {
            /* dispose is try-wrapped */
          }
          tagMount = null
        }
      }

      const emitError = (d: CompileDiagnostic, phase: ErrPhase): void => {
        const key = `${phase}|${errorKey(d)}`
        if (key === lastErrorKey) return // deduped before emit (the lastEmitted guard)
        lastErrorKey = key
        host.emit!('sail-render:error', { message: d.message, line: d.line, col: d.col, phase })
      }

      const showCompileError = (d: CompileDiagnostic, src: string): void => {
        const f = formatCompileError(src, d)
        compileErrorSummary.set(f.location ? `${f.message} (${f.location})` : f.message)
        compileErrorBody.set(f.caretFrame || f.message)
        compileError.set(f)
        emitError(d, 'compile')
      }

      const clearCompileError = (): void => {
        if (compileError() !== null) compileError.set(null)
      }


      const stop = effect(() => {
        const stage = stageEl()
        const fn = compile()
        const src = source()
        const mode = ctx.mode() as 'source' | 'tag'
        const tag = (ctx.componentName() as string) || ''
        const pv = propValues()
        const slotHtml = slots()

        setupFn()
        setupUrl()
        if (!stage || !fn || src == null) return

        if (mode === 'tag') {
          untrack(() => mountTag(stage, fn, tag, src, pv, slotHtml))
        } else {
          untrack(() => mountSource(stage, fn, src))
        }
      })

      function mountSource(stage: HostBag, fn: CompileFn, src: string): void {
        compiling.set(true)
        try {
          // Compile before dispose so last-good survives a bad source.
          const su = setupUrl()
          const sf = setupFn()
          const result = mountBySource(stage, fn, {
            source: src,
            ...(sf != null ? { setup: sf as never } : {}),
            ...(su != null ? { setupUrl: su } : {}),
            preDispose: () => disposeView(),
          })
          viewMount = result.mount
          lastViewKey = src
          clearCompileError()
          lastErrorKey = ''
          host.emit!('sail-render:compiled', {
            js: result.js,
            css: result.css,
            dts: null,
            ssrHtml: null,
          })
          host.emit!('sail-render:mounted', { tag: null })
        } catch (e) {

          const d = diagFromError(e)
          const phase: ErrPhase = /sail-compiler:/.test(d.message) ? 'compile' : 'mount'
          if (phase === 'compile') {
            showCompileError(d, lastViewKey || src)
          } else {
            runtimeError.set(d.message)
            emitError(d, 'mount')
          }
        } finally {
          compiling.set(false)
        }
      }

      function mountTag(
        stage: HostBag,
        fn: CompileFn,
        tag: string,
        src: string,
        pv: Record<string, unknown> | null,
        slotHtml: string | null,
      ): void {
        if (!tag) return

        if (tagMount && src === lastTagSource) {
          tagMount.setProps(pv)
          if (slotHtml != null) tagMount.setSlots(slotHtml)
          return
        }
        // Source change → content-hashed versioned tag (registry is append-only).
        const versioned = tagMount != null
        compiling.set(true)
        try {
          const next = mountByTag(stage, fn, { tag, source: src, props: pv, slots: slotHtml, versioned })
          disposeTag()
          tagMount = next
          lastTagSource = src
          clearCompileError()
          lastErrorKey = ''
          host.emit!('sail-render:compiled', { js: '', css: null, dts: null, ssrHtml: null })

          host.emit!('sail-render:mounted', { tag: next.tag })
        } catch (e) {
          showCompileError(diagFromError(e), src)
        } finally {
          compiling.set(false)
        }
      }

      // error-boundary → sail-render:error + runtime panel.
      const onCaught = (e: DomEv): void => {
        const detail = (e as DomEv).detail as { error?: unknown } | undefined
        const err = detail?.error
        const message = err instanceof Error ? err.message : String(err ?? 'runtime error')
        runtimeError.set(message)
        emitError({ message, line: null, col: null, category: 'runtime' }, 'runtime')
      }
      host.addEventListener('sail-error-boundary:caught', onCaught)

      return () => {
        stop()
        disposeView()
        disposeTag()
        host.removeEventListener('sail-error-boundary:caught', onCaught)
      }
    })

    return { stageEl, captureStage, stageMaxWidth, compiling, compileError, runtimeError, compileErrorSummary, compileErrorBody }
  })

</controller>
<error-boundary fallback-text="The specimen threw while rendering.">
  <column gap="2" style({ box: position(relative).minH(8), paint: fg(foreground) })>    <span style({ box: display(contents), text: fontFamily(body) })>      <if {compiling()}>
      <row align="center" justify="center" style({ box: pad(4) })>
        <sail-spinner size="sm" label="Compiling"></sail-spinner>
      </row>
    </if>

    <if {compileError() !== null}>
      <column gap="2" data-test="compile-error">
        <alert tone="destructive">{compileErrorSummary()}</alert>
        <div role="region" aria-label="text code block" style({ box: overflow(hidden), paint: bg(surface-2).fg(foreground).borderWidth(1).border(border-subtle).radius(md) })>
          <header style({ box: pad(2).padX(3), paint: fg(muted-foreground).bg(muted) })>            <span style({ box: display(contents), text: fontSize(xs).fontWeight(medium).tracking(wide) })>              TEXT
            </span>
</header>
          <separator style({ box: m(0) })></separator>
          <pre style({ box: padY(3).padX(4).overflowX(auto), text: leading(loose) })>{compileErrorBody()}</pre>
        </div>
      </column>
    </if>

    <if {runtimeError() !== null}>
      <column gap="2" data-test="runtime-error">
        <alert tone="destructive">The specimen threw at runtime.</alert>
        <div role="region" aria-label="text code block" style({ box: overflow(hidden), paint: bg(surface-2).fg(foreground).borderWidth(1).border(border-subtle).radius(md) })>
          <header style({ box: pad(2).padX(3), paint: fg(muted-foreground).bg(muted) })>            <span style({ box: display(contents), text: fontSize(xs).fontWeight(medium).tracking(wide) })>              TEXT
            </span>
</header>
          <separator style({ box: m(0) })></separator>
          <pre style({ box: padY(3).padX(4).overflowX(auto), text: leading(loose) })>{runtimeError()}</pre>
        </div>
      </column>
    </if>

    <div use:captureStage use:maxWidth={stageMaxWidth()} style({ box: w(full).ml(auto).mr(auto).maxW(bind(maxW)) })></div>
    </span>
</column>
</error-boundary>
