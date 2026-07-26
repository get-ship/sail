<!--
  <sail-render> — THE engine primitive: a live, reactive mount stage for a SAIL
  component (build-spec §1 / §Render-engine seam).

  Thin shell. ALL render-engine work (compile + view-mode `__mount` + net-new
  component-mode createElement / prop-apply / slot-inject / versioned-tag swap
  + dispose) lives in the paired controller + `engine/*.ts`. The template only:
    - declares the scalar control props (mode / component-name / viewport-width)
    - hosts the stage container via `use:captureStage` (action) —
      NEVER a post-mount querySelector
    - wraps the fault-prone subtree in <error-boundary> so an agent's `.sail`
      that throws in setup/mount/render is CAUGHT, not a white screen
    - folds the compile-error display (the culled sail-compile-error) and the
      runtime-error panel, plus a <sail-spinner> while compiling

  Data (the bound `compile()`, raw `source`, `propValues`, `setup`, `slots`)
  arrives via `*` identity/snapshot — the host owns discovery. NO
  object/json attribute props (Cardinal: `<prop type=json>` does not exist).
-->

<scope>
  props: {
    mode: 'source' | 'tag' = 'source'
    'component-name' = ''
    'viewport-width': 'full' | 'desktop' | 'tablet' | 'mobile' = 'full'
  }
</scope>



<controller>
// <sail-render> controller — the render-engine lifecycle (build-spec §1).
//
// Controller-purity: this file does NO view-markup DOM construction. It owns
// LIFECYCLE only — wiring host `*` into signals, running the reactive
// compile+mount effect, disposing prior mounts, and emitting events. All
// `createElement` / `createContextualFragment` / `__mount` lives in
// `engine/*.ts` (the legitimate render-engine seam, outside
// `sail-controller-purity` scope). No `innerHTML`/`outerHTML`/`insertAdjacentHTML`.

import { controller, signal, effect, untrack } from '@ship/sail-runtime'
import type { Action, ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './sail-render.sail'
import { attachEngineShadow, mountBySource, type ViewMount } from './engine/mount-source'
import { mountByTag, type TagMount } from './engine/mount-tag'
import { formatCompileError, errorKey, type CompileDiagnostic } from './engine/compile-error'
import type { CompileFn, WasmCompileResult } from '../wasm-bootstrap'

type ErrPhase = 'compile' | 'mount' | 'runtime'

// Pull a structured diagnostic out of an injected-compile `{kind:'err'}`
// result, or out of a thrown `sail-compiler: <msg> (l:c)` Error.
function diagFromResult(r: Extract<WasmCompileResult, { kind: 'err' }>): CompileDiagnostic {
  return { message: r.message, line: r.line, col: r.col, category: r.category }
}
function diagFromError(e: unknown): CompileDiagnostic {
  const message = e instanceof Error ? e.message : String(e)
  const m = /\((\d+):(\d+)\)/.exec(message)
  return { message, line: m ? Number(m[1]) : null, col: m ? Number(m[2]) : null, category: 'compile' }
}

export default controller((ctx: ControllerScope<Scope>) => {
    const stageEl = signal<HTMLElement | null>(null)
    const captureStage: Action<void> = (node) => {
      stageEl.set(node as HTMLElement)
      return { destroy: () => stageEl.set(null) }
    }
    // Stage framing: the reflected viewport-width prop picks a breakpoint
    // token; full means unconstrained.
    const viewportWidth = (ctx as unknown as { viewportWidth?: () => string }).viewportWidth
    const stageMaxWidth = (): string | null => {
      const w = viewportWidth ? viewportWidth() : 'full'
      return w === 'full' || !w ? null : `var(--breakpoint-${w})`
    }
    const compiling = signal(false)
    const compileError = signal<ReturnType<typeof formatCompileError> | null>(null)
    const runtimeError = signal<string | null>(null)
    // Derived display strings for the folded compile-error branch.
    const compileErrorSummary = signal('')
    const compileErrorBody = signal('')
    mount((host) => {
      // Attach the engine shadow root + relocate the compiled template into it
      // (build-spec §Isolation — the engine stage is a shadow on the host).
      attachEngineShadow(host)

      // --- host * → signals (the property-apply path; identity preserved) ---
      const compile = signal<CompileFn | null>(null)
      const source = signal<string | null>(null)
      const propValues = signal<Record<string, unknown> | null>(null)
      const setupFn = signal<unknown>(null)
      const setupUrl = signal<string | null>(null)
      const slots = signal<string | null>(null)

      // Store the property value into the signal WITHOUT the updater-function
      // interpretation: `signal.set(fn)` would CALL `fn(prev)` (the updater form),
      // which is wrong for the function-valued props (`compile`, `setup`). The
      // updater closure `() => value` stores the literal value — function or not.
      type AnySig = { (): unknown; set: (u: (p: unknown) => unknown) => void }
      const defineHostProp = (name: string, sig: { (): unknown; set: (v: never) => void }): void => {
        const s = sig as unknown as AnySig
        const existing = (host as unknown as Record<string, unknown>)[name]
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

      // --- live mount handles + dedup guards ---
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

      // --- the reactive compile+mount effect (source/tag/prop change) ---
      const stop = effect(() => {
        const stage = stageEl()
        const fn = compile()
        const src = source()
        const mode = ctx.mode() as 'source' | 'tag'
        const tag = (ctx.componentName() as string) || ''
        const pv = propValues()
        const slotHtml = slots()
        // Track the view-mode setup/marker so a late-supplied setup re-mounts —
        // the harness may set `source` before `setup`/`setupUrl` land.
        setupFn()
        setupUrl()
        if (!stage || !fn || src == null) return

        if (mode === 'tag') {
          untrack(() => mountTag(stage, fn, tag, src, pv, slotHtml))
        } else {
          untrack(() => mountSource(stage, fn, src))
        }
      })

      function mountSource(stage: HTMLElement, fn: CompileFn, src: string): void {
        compiling.set(true)
        try {
          // Compile FIRST (may throw → last-good specimen survives below). Only
          // once we have a clean compile do we tear down the prior mount and
          // mount the new one — disposing first would race the snapshot diff.
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
          // A bad source-change KEEPS the last-good specimen (do not dispose) and
          // surfaces the diagnostic. A first-mount failure simply shows the panel.
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
        stage: HTMLElement,
        fn: CompileFn,
        tag: string,
        src: string,
        pv: Record<string, unknown> | null,
        slotHtml: string | null,
      ): void {
        if (!tag) return
        // Prop-only change (same source, node already mounted): patch in place —
        // NO recompile, NO recreate (build-spec §Reactivity, prop channel).
        if (tagMount && src === lastTagSource) {
          tagMount.setProps(pv)
          if (slotHtml != null) tagMount.setSlots(slotHtml)
          return
        }
        // A source CHANGE (tag already mounted, src differs) must mint a FRESH
        // definition: the customElements registry is process-global + append-only
        // (no undefine), so re-using the base tag would silently keep the OLD
        // definition (stale render). A content-hashed versioned tag gives a fresh
        // definition for distinct source while the define-once guard collapses
        // identical source — bounding registry growth by distinct-source count.
        // First mount (no prior tagMount) uses the base tag so a host querying by
        // `component-name` finds the registered element.
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
          // Report the tag actually mounted: the base tag on first mount, the
          // content-hashed versioned tag on a source-change swap.
          host.emit!('sail-render:mounted', { tag: next.tag })
        } catch (e) {
          showCompileError(diagFromError(e), src)
        } finally {
          compiling.set(false)
        }
      }

      // --- runtime throw: the inner sail-error-boundary fires a composed
      //     `sail-error-boundary:caught` that crosses the shadow; re-surface it as
      //     a runtime sail-render:error and render the runtime panel. ---
      const onCaught = (e: Event): void => {
        const detail = (e as CustomEvent).detail as { error?: unknown } | undefined
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
  <column gap="2" sx={style.position(relative).minH(8).fontFamily(body).fg(foreground)}>
    <if cond={compiling()}>
      <row align="center" justify="center" sx={style.pad(4)}>
        <sail-spinner size="sm" label="Compiling"></sail-spinner>
      </row>
    </if>

    <if cond={compileError() !== null}>
      <column gap="2" data-test="compile-error">
        <alert tone="destructive">{compileErrorSummary()}</alert>
        <div role="region" aria-label="text code block" sx={style.bg(surface-2).fg(foreground).borderWidth(1).border(border-subtle).radius(md).overflow(hidden)}>
          <header sx={style.pad(2).padX(3).fontSize(xs).fg(muted-foreground).bg(muted).fontWeight(medium).tracking(wide)}>TEXT</header>
          <separator sx={style.m(0)}></separator>
          <pre sx={style.padY(3).padX(4).overflowX(auto).leading(normal-plus)}>{compileErrorBody()}</pre>
        </div>
      </column>
    </if>

    <if cond={runtimeError() !== null}>
      <column gap="2" data-test="runtime-error">
        <alert tone="destructive">The specimen threw at runtime.</alert>
        <div role="region" aria-label="text code block" sx={style.bg(surface-2).fg(foreground).borderWidth(1).border(border-subtle).radius(md).overflow(hidden)}>
          <header sx={style.pad(2).padX(3).fontSize(xs).fg(muted-foreground).bg(muted).fontWeight(medium).tracking(wide)}>TEXT</header>
          <separator sx={style.m(0)}></separator>
          <pre sx={style.padY(3).padX(4).overflowX(auto).leading(normal-plus)}>{runtimeError()}</pre>
        </div>
      </column>
    </if>

    <div use:captureStage use:maxWidth={stageMaxWidth()} sx={style.w(full).ml(auto).mr(auto).maxW(bind(maxW))}></div>
  </column>
</error-boundary>
