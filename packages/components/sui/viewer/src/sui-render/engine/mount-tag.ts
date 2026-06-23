// Component-mode mount (build-spec §Mount by catalog tag — NET-NEW engine
// code; the harness component branch is define-only). The createElement +
// prop-apply + slot-inject + versioned-tag swap + dispose all live here.
//
// Reuses ONLY the injected `compile()`'s component-mode define side-effect
// (`component(tag, def)` → `customElements.define`). A define-once guard on
// `customElements.get(tag)` means a tag already in the (process-global,
// append-only) registry is NOT redefined — redefining throws
// `NotSupportedError`.
//
// Source-change re-render uses a CONTENT-HASHED versioned tag
// (`base + '-v' + hash(source)`): identical source RE-USES the prior tag (the
// define-once guard hits), so repeated identical edits do NOT grow the
// registry (the load-bearing growth bound). Only genuinely new source mints a
// new tag.

import { evalComponentModule } from './eval-module'
import { applyProps, applyProp } from './prop-apply'
import type { CompileFn } from '../../wasm-bootstrap'

export interface TagMountOpts {
  /** The base custom-element tag the specimen registers under. */
  tag: string
  /** The `.sui` source compiled in component mode. */
  source: string
  /** Scalar + structured props applied to the createElement host. */
  props?: Record<string, unknown> | null
  /** Composed slot inner-HTML injected into the host (light DOM). */
  slots?: string | null
  /**
   * Mint a CONTENT-HASHED versioned tag (`base-v<hash(source)>`) instead of the
   * base tag. Set on a source-CHANGE re-mount: the custom-elements registry is
   * process-global + append-only (no undefine), so reusing the base tag after
   * the source changes would silently keep the OLD definition (stale render).
   * A versioned tag gives a FRESH definition for distinct source, while the
   * define-once guard collapses identical source to the prior registration —
   * bounding registry growth by distinct-source count. Defaults to false (the
   * first mount uses the base tag so the host can `createElement(base)`).
   */
  versioned?: boolean
}

export interface TagMount {
  el: HTMLElement
  /** The tag actually defined+instantiated (base on first mount; versioned on
   *  a source-change swap). */
  tag: string
  setProp: (name: string, value: unknown) => void
  setProps: (props: Record<string, unknown> | null | undefined) => void
  setSlots: (html: string | null) => void
  dispose: () => void
}

/** A small, stable content hash of the source — keys the versioned tag so
 *  identical source re-uses the prior registration (registry-growth bound). */
function hashSource(source: string): string {
  let h = 5381
  for (let i = 0; i < source.length; i++) {
    h = (h * 33) ^ source.charCodeAt(i)
  }
  return (h >>> 0).toString(36)
}

/** Compile + define a tag exactly once (the define-once guard). The first
 *  define uses the base tag; subsequent distinct sources use a versioned tag.
 *  Returns the tag name that is now registered. */
function ensureDefined(compile: CompileFn, baseTag: string, source: string, versioned: boolean): string {
  const tag = versioned ? `${baseTag}-v${hashSource(source)}` : baseTag
  if (customElements.get(tag)) return tag
  const compiled = compile(source, { mode: 'component', componentName: tag })
  if (compiled.kind === 'err') {
    const where = compiled.line != null ? ` (${compiled.line}:${compiled.col ?? 0})` : ''
    throw new Error(`sui-compiler: ${compiled.message}${where}`)
  }
  evalComponentModule(compiled.js)
  return tag
}

/** Inject composed slot inner-HTML into the host's light DOM via
 *  `Range.createContextualFragment` (the legitimate render-engine DOM seam —
 *  an engine `*.ts` module, NOT a `.controller.ts`; outside the
 *  `sui-controller-purity` rule's scope). */
function injectSlots(el: HTMLElement, html: string | null): void {
  // Remove engine-injected slot content from a prior set (marked nodes only).
  for (const prev of Array.from(el.querySelectorAll(':scope > [data-sui-render-slot]'))) {
    prev.remove()
  }
  if (!html) return
  const wrap = document.createElement('div')
  wrap.setAttribute('data-sui-render-slot', '')
  const range = document.createRange()
  range.selectNodeContents(wrap)
  wrap.appendChild(range.createContextualFragment(html))
  el.appendChild(wrap)
}

/**
 * Mount a named component into the stage host. Compiles+defines (guarded),
 * createElements the node, applies props (scalar→attr / structured→property),
 * injects slots, and appends. Returns a handle that patches props/slots IN
 * PLACE (no remount) and disposes the node.
 */
export function mountByTag(
  stage: HTMLElement | ShadowRoot,
  compile: CompileFn,
  opts: TagMountOpts,
): TagMount {
  const tag = ensureDefined(compile, opts.tag, opts.source, opts.versioned ?? false)
  const el = document.createElement(tag)
  applyProps(el, opts.props)
  ;(stage as HTMLElement).appendChild(el)
  injectSlots(el, opts.slots ?? null)

  return {
    el,
    tag,
    setProp: (name, value) => applyProp(el, name, value),
    setProps: (props) => applyProps(el, props),
    setSlots: (html) => injectSlots(el, html),
    dispose: () => {
      el.remove()
    },
  }
}
