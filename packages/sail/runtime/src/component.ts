// Custom-element wrapper. The ship-set compiler emits one
// `defineComponent('my-tag', factory)` call per .set file. The factory
// is the user's script + binding wiring; defineComponent registers
// the wrapper class with `customElements.define`.
//
// See .artifacts/specs/ship-set/03-runtime-architecture.md § Component wrapper.

import type { Disposer } from './bind'
import type { Signal } from './signal'
import { withScope, type EffectScope } from './effect-scope'
import { connectElementScope, disconnectElementScope } from './element-scope'
import { withHost, untrack } from './signal'
import { attachHostApi } from './host-api'
import { findNearestBoundary } from './error-boundary'
import { dispatchSuiError } from './error-overlay'

// The declarative `component()` factory lives in component-factory.ts to
// keep this file under the 300-line cap. Re-exported here so the public
// API can be imported from a single module.
export {
  component,
  prop,
  type PropDef,
  type ComponentDef,
  type ComponentSetupContext,
} from './component-factory'

/**
 * The factory the compiler emits per component. Receives the host
 * element, a place to attach prop signals, and (for form-associated
 * components) the ElementInternals instance attached at construction
 * time. Returns a cleanup function that runs on disconnect.
 */
export type ComponentFactory = (
  host: HTMLElement,
  propSignals: Record<string, Signal<unknown>>,
  internals?: ElementInternals,
) => Disposer

/**
 * Form-association lifecycle hooks the factory may install on the host
 * via `host.__shipSetFormHooks = {...}`. The form-associated class
 * shape invokes them from the matching browser callbacks, runs them
 * BEFORE dispatching the `ship-set:form-*` event so listeners observe
 * the post-hook state.
 */
export interface FormHooks {
  reset?: () => void
  restore?: (state: string | null) => void
  disabled?: (disabled: boolean) => void
}

interface FormHookHolder {
  __shipSetFormHooks?: FormHooks
}

/**
 * Schema the compiler emits alongside the factory — drives prop decode
 * + observed-attributes registration.
 */
export interface ComponentSchema {
  /** Prop attribute names (kebab-case) the element observes. */
  observedAttrs: string[]
  /** Map prop attribute name → camelCase property name + decoder. */
  props: Record<
    string,
    {
      camel: string
      decode: (raw: string | null) => unknown
    }
  >
  /**
   * When true, the emitted custom element declares
   * `static formAssociated = true` and calls `attachInternals()` at
   * construction. The factory receives the ElementInternals as its
   * third argument so script-block code can call setFormValue,
   * setValidity, checkValidity, and friends.
   */
  formAssociated?: boolean
}

/**
 * Register a ship-set component as a custom element. The wrapper class
 * observes the schema's attributes, decodes incoming changes into prop
 * signals, runs `factory` on connect, and runs the returned cleanup on
 * disconnect.
 *
 * Re-define = HMR HOT-SWAP (#40): calling again with the SAME tag swaps the
 * factory/schema behind the (once-registered) wrapper class and re-renders every
 * live instance in place — no full document reload. (Custom elements can't be
 * re-registered, hence the stable-wrapper + mutable-registry indirection.) A
 * FOREIGN collision — a tag registered OUTSIDE defineComponent — still warns
 * once and no-ops, since that's a real bug, not an HMR cycle.
 *
 * The ship-set compiler emits exactly one call per `.set` file. Authors
 * shouldn't need to reach for this directly.
 *
 * @param tag Custom element tag name (must contain a hyphen).
 * @param schema Observed-attribute list + per-attribute decoder map +
 *   optional `formAssociated` flag.
 * @param factory Per-instance setup; receives the host, prop-signal map,
 *   and (when form-associated) ElementInternals. Returns a disposer
 *   that runs on disconnect.
 *
 * @example
 * ```ts
 * defineComponent('my-counter', {
 *   observedAttrs: ['initial'],
 *   props: { initial: { camel: 'initial', decode: decodeNumber } },
 * }, (host, props) => {
 *   const count = props.initial as Signal<number>
 *   return () => {} // cleanup
 * })
 * ```
 *
 * @see camelOf for attribute → prop name translation.
 */
const warnedDuplicateTags = new Set<string>()

/** A live component registration. The wrapper class is registered ONCE with the
 *  browser (custom elements can't be re-registered), but the per-instance
 *  `factory` + `schema` behind it are mutable — so an HMR re-define can swap
 *  them and re-render every live instance in place (component hot-swap, #40). */
interface ComponentReg {
  schema: ComponentSchema
  factory: ComponentFactory
  /** Connected instances, so a re-define can re-render them. */
  instances: Set<{ __suiReRun(): void }>
}
const liveComponents = new Map<string, ComponentReg>()

export function defineComponent(
  tag: string,
  schema: ComponentSchema,
  factory: ComponentFactory,
): void {
  const existing = liveComponents.get(tag)
  if (existing) {
    // ── HMR hot-swap (#40) ── our tag, re-defined (a re-imported module). Swap
    // the impl + re-render every live instance in place. The wrapper class stays
    // registered; observedAttributes is frozen at first define, so adding/
    // removing observed attrs still needs a full reload — but render/factory
    // changes hot-swap with no full document reload.
    existing.factory = factory
    existing.schema = schema
    for (const el of existing.instances) {
      try {
        el.__suiReRun()
      } catch (err) {
        console.error(`[ship-set:${tag}] HMR re-run threw:`, err)
      }
    }
    return
  }
  if (customElements.get(tag)) {
    // A FOREIGN registration: this tag exists but we didn't define it (another
    // module reached for `customElements.define` directly). Keep the warn-once
    // no-op so a real collision doesn't go unnoticed.
    if (!warnedDuplicateTags.has(tag)) {
      warnedDuplicateTags.add(tag)
      console.warn(
        `[sail-runtime] customElements.define("${tag}", …) called twice. ` +
          `The first registration wins; this call is a no-op. Likely cause: ` +
          `two modules independently registering the same tag.`,
      )
    }
    return
  }
  // First registration: the wrapper class registers once; its `factory`/`schema`
  // live in a mutable registry behind it so a later re-define can hot-swap.
  const reg: ComponentReg = { schema, factory, instances: new Set() }
  liveComponents.set(tag, reg)

  // Two class shapes — form-associated requires `static formAssociated`
  // to be present at class-definition time so the browser registers
  // ElementInternals support. The non-associated path skips the
  // constructor + internals.
  if (schema.formAssociated) {
    class ShipSetFormElement extends HTMLElement {
      static formAssociated = true
      static get observedAttributes(): string[] {
        return reg.schema.observedAttrs
      }

      private cleanup: Disposer | undefined
      private scope: EffectScope | undefined
      private propSignals: Record<string, Signal<unknown>> = {}
      /** Prop values set via JS property before `connectedCallback` populated
       *  `propSignals` — buffered by the prop accessors, flushed on connect.
       *  Mirrors the non-form class below so `el.value = 'x'` from a parent's
       *  `prop:value={…}` binding (which fires before the child's
       *  connectedCallback runs) survives into the prop signal instead of
       *  being silently dropped. Without this, a `<sail-pagination page-size="25">`
       *  sets `value="25"` on its inner `<sail-select>` but the value arrives
       *  before the select's setup runs, so the prop signal stays at its
       *  default `""` and the host's `el.value` is empty. */
      private pendingProps: Record<string, unknown> | undefined
      private internals: ElementInternals

      constructor() {
        super()
        this.internals = this.attachInternals()
      }

      private runSetup() {
        // Captured into a local: `this.scope` can be cleared mid-call if the
        // element is synchronously disconnected + reconnected while its own
        // setup is still running (a parent's DOM reconciliation moving this
        // node re-enters disconnectedCallback -> teardown() -> `this.scope =
        // undefined`, re-entrant on the SAME call stack as this runSetup —
        // observed with sail-component-viewer's example-swap re-insertion).
        // Operate on the local so the ambient-mount wiring below still runs
        // against the scope THIS call created, not whatever `this.scope`
        // happens to hold by the time we get there.
        // Parented into the DOM-mirroring element-scope tree (element-scope.ts):
        // nearest ancestor sail host's scope, else the per-document client root —
        // a bare root-per-element would give every element its own `useStore`
        // instance instead of the shared nearest-or-root one.
        const scope = connectElementScope(this)
        this.scope = scope
        try {
          this.cleanup = withScope(scope, () =>
            withHost(this, () => reg.factory(this, this.propSignals, this.internals)),
          )
          // Ambient mount() callbacks the factory (or a composition it called)
          // registered on this.scope — the element is already connected by the
          // time connectedCallback runs, so "after connect" IS now. Run inside
          // withScope so a mount() callback's own onCleanup()/live()/data()
          // calls still enroll on this.scope, not a dangling one.
          withScope(scope, () => scope.runMounts(this))
        } catch (err) {
          const boundary = findNearestBoundary(this.parentNode)
          if (boundary) {
            boundary.catch(err, 'mount')
          } else {
            console.error(`[ship-set:${tag}] setup threw:`, err)
            dispatchSuiError(this, err, 'setup')
          }
        }
        // Flush any prop values set via JS property BEFORE the factory
        // populated propSignals (pre-connect writes buffered by the accessors;
        // the #P0.2 sail-pagination form-value bug without this flush).
        if (this.pendingProps) {
          for (const k in this.pendingProps) this.propSignals[k]?.set(this.pendingProps[k])
          this.pendingProps = undefined
        }
      }

      private teardown() {
        try {
          this.cleanup?.()
        } catch (e) {
          console.error(`[ship-set:${tag}] cleanup threw:`, e)
        }
        this.cleanup = undefined
        const scope = this.scope
        if (scope) {
          // Un-stamp before dispose so a descendant connecting mid-teardown
          // never adopts a dying scope as its parent.
          disconnectElementScope(this, scope)
          try {
            scope.dispose()
          } catch (e) {
            console.error(`[ship-set:${tag}] scope dispose threw:`, e)
          }
        }
        this.scope = undefined
      }

      connectedCallback() {
        this.dataset.component = tag
        attachHostApi(this)
        reg.instances.add(this)
        this.runSetup()
      }

      disconnectedCallback() {
        reg.instances.delete(this)
        this.teardown()
      }

      // HMR (#40): re-render this live instance with the current `reg.factory`.
      // Tears down, clears the rendered DOM (cleanup only disposes bindings; the
      // new factory would otherwise duplicate it), re-runs. State resets.
      __suiReRun() {
        this.teardown()
        ;(this.shadowRoot ?? this).replaceChildren()
        this.propSignals = {}
        this.runSetup()
      }

      attributeChangedCallback(attrName: string, _old: string, value: string) {
        const meta = reg.schema.props[attrName]
        if (!meta) return
        const sig = this.propSignals[meta.camel]
        if (sig) sig.set(meta.decode(value))
      }

      // Form-association lifecycle hooks. The browser dispatches these
      // on the host. The runtime fans them out two ways:
      //   1. Element-level events (`ship-set:form-*`) for ad-hoc
      //      listeners attached by author code or tests.
      //   2. Factory-installed handlers stored under
      //      `__shipSetFormHooks` (set by component-factory.ts when the
      //      author declared `formValue:` / `formDisabled:` on the
      //      component def). These run BEFORE the event so signal
      //      writes are visible to listeners that read the prop.
      formResetCallback() {
        const hooks = (this as unknown as FormHookHolder).__shipSetFormHooks
        try { hooks?.reset?.() } catch (e) {
          console.error(`[ship-set:${tag}] formResetCallback threw:`, e)
        }
        this.dispatchEvent(new CustomEvent('ship-set:form-reset'))
      }
      formStateRestoreCallback(state: string | null, mode: string) {
        const hooks = (this as unknown as FormHookHolder).__shipSetFormHooks
        try { hooks?.restore?.(state) } catch (e) {
          console.error(`[ship-set:${tag}] formStateRestoreCallback threw:`, e)
        }
        this.dispatchEvent(
          new CustomEvent('ship-set:form-restore', { detail: { state, mode } }),
        )
      }
      formDisabledCallback(disabled: boolean) {
        const hooks = (this as unknown as FormHookHolder).__shipSetFormHooks
        try { hooks?.disabled?.(disabled) } catch (e) {
          console.error(`[ship-set:${tag}] formDisabledCallback threw:`, e)
        }
        this.dispatchEvent(
          new CustomEvent('ship-set:form-disabled', { detail: { disabled } }),
        )
      }
    }
    // Mirror the prop-accessor loop from the non-form class below
    // (#307): form-associated components must expose `el.value` (and any
    // other declared prop) as a prototype getter that reads the prop
    // signal, so user code like `formEl.value` returns the current value
    // instead of `undefined`. Without this, `<sail-select value="x">` is
    // invisible to plain DOM code that reads `el.value` (the form-association
    // path goes through `ElementInternals.setFormValue`, not the property).
    installPropAccessors(ShipSetFormElement.prototype, schema)
    customElements.define(tag, ShipSetFormElement)
    return
  }

  class ShipSetElement extends HTMLElement {
    static get observedAttributes(): string[] {
      return reg.schema.observedAttrs
    }

    private cleanup: Disposer | undefined
    private scope: EffectScope | undefined
    private propSignals: Record<string, Signal<unknown>> = {}
    /** Prop values set via JS property before `connectedCallback` populated
     *  `propSignals` — buffered by the prop accessors, flushed on connect. */
    private pendingProps: Record<string, unknown> | undefined

    private runSetup() {
      // Captured into a local: `this.scope` can be cleared mid-call if the
      // element is synchronously disconnected + reconnected while its own
      // setup is still running (a parent's DOM reconciliation moving this
      // node re-enters disconnectedCallback -> teardown() -> `this.scope =
      // undefined`, re-entrant on the SAME call stack as this runSetup —
      // observed with sail-component-viewer's example-swap re-insertion).
      // Operate on the local so the ambient-mount wiring below still runs
      // against the scope THIS call created, not whatever `this.scope`
      // happens to hold by the time we get there.
      // Parented into the DOM-mirroring element-scope tree (element-scope.ts):
      // nearest ancestor sail host's scope, else the per-document client root —
      // a bare root-per-element would give every element its own `useStore`
      // instance instead of the shared nearest-or-root one.
      const scope = connectElementScope(this)
      this.scope = scope
      try {
        this.cleanup = withScope(scope, () =>
          withHost(this, () => reg.factory(this, this.propSignals)),
        )
        // Ambient mount() callbacks the factory (or a composition it called)
        // registered on this.scope — the element is already connected by the
        // time connectedCallback runs, so "after connect" IS now. Run inside
        // withScope so a mount() callback's own onCleanup()/live()/data()
        // calls still enroll on this.scope, not a dangling one.
        withScope(scope, () => scope.runMounts(this))
        // Apply any prop set via JS property before connection created the
        // signals (the prop accessors buffer pre-connect writes here).
        if (this.pendingProps) {
          for (const k in this.pendingProps) this.propSignals[k]?.set(this.pendingProps[k])
          this.pendingProps = undefined
        }
      } catch (err) {
        const boundary = findNearestBoundary(this.parentNode)
        if (boundary) {
          boundary.catch(err, 'mount')
        } else {
          console.error(`[ship-set:${tag}] setup threw:`, err)
          dispatchSuiError(this, err, 'setup')
        }
      }
    }

    private teardown() {
      try {
        this.cleanup?.()
      } catch (e) {
        console.error(`[ship-set:${tag}] cleanup threw:`, e)
      }
      this.cleanup = undefined
      const scope = this.scope
      if (scope) {
        // Un-stamp before dispose so a descendant connecting mid-teardown
        // never adopts a dying scope as its parent.
        disconnectElementScope(this, scope)
        try {
          scope.dispose()
        } catch (e) {
          console.error(`[ship-set:${tag}] scope dispose threw:`, e)
        }
      }
      this.scope = undefined
    }

    connectedCallback() {
      this.dataset.component = tag
      attachHostApi(this)
      reg.instances.add(this)
      this.runSetup()
    }

    disconnectedCallback() {
      reg.instances.delete(this)
      this.teardown()
    }

    // HMR (#40): re-render this live instance with the current `reg.factory`.
    // Tears down, clears the rendered DOM (cleanup only disposes bindings; the
    // new factory would otherwise duplicate it), re-runs. State resets.
    __suiReRun() {
      this.teardown()
      ;(this.shadowRoot ?? this).replaceChildren()
      this.propSignals = {}
      this.runSetup()
    }

    attributeChangedCallback(attrName: string, _old: string, value: string) {
      const meta = reg.schema.props[attrName]
      if (!meta) return
      const sig = this.propSignals[meta.camel]
      if (sig) sig.set(meta.decode(value))
    }
  }

  // JS property accessors for declared props, delegating to the prop signal —
  // so `el.<camel> = v` (what the compiler's `prop:` binding emits) reaches the
  // signal exactly like the attribute path, instead of writing a dead expando.
  // Without these, `prop:open` on a prebuilt primitive silently no-ops (the
  // bug behind dialogs not opening). Skip names already on the prototype chain
  // (native HTMLElement props like `hidden`/`title`) so we never shadow built-in
  // behavior — those keep using the attribute path. Reads are untracked: a
  // property read is a point-in-time value, not a reactive subscription.
  installPropAccessors(ShipSetElement.prototype, schema)
  customElements.define(tag, ShipSetElement)
}

/** Host the prop-signal accessors. Mirrors the shape the classes expose
 *  through their `propSignals` (form-class) or `propSignals` + `pendingProps`
 *  (non-form class) instance fields. */
type PropHost = {
  propSignals: Record<string, Signal<unknown>>
  pendingProps?: Record<string, unknown>
}

/**
 * Install a `get`/`set` accessor on `prototype` for every prop in `schema`,
 * reading from / writing to the instance's `propSignals` map. Pre-connect
 * `set` writes buffer into `pendingProps` so a prop set on the element
 * before `connectedCallback` (i.e. before the factory has built the signal)
 * reaches the signal on first connect. Skip names already on the prototype
 * chain (native HTMLElement props like `hidden`/`title`) so we never shadow
 * built-in behaviour. Reads are untracked: a property read is a point-in-time
 * value, not a reactive subscription.
 */
export function installPropAccessors(
  prototype: object,
  schema: ComponentSchema,
): void {
  // Every prop gets an accessor — attribute-backed AND property-only props
  // alike — so `el.prop = …` bridges to the signal in all cases. Attribute
  // observation (observedAttrs) is the narrower, attribute-only subset.
  for (const attr of Object.keys(schema.props)) {
    const meta = schema.props[attr]
    if (!meta) continue
    const camel = meta.camel
    if (camel in (prototype as Record<string, unknown>)) continue
    Object.defineProperty(prototype, camel, {
      configurable: true,
      enumerable: false,
      get(this: PropHost) {
        const sig = this.propSignals[camel]
        return sig ? untrack(() => sig()) : undefined
      },
      set(this: PropHost, value: unknown) {
        const sig = this.propSignals[camel]
        if (sig) sig.set(value)
        else (this.pendingProps ??= {})[camel] = value
      },
    })
  }
}

/**
 * Convert a kebab-case HTML attribute name to its camelCase prop name.
 * Single-word attributes pass through unchanged.
 *
 * @param attr Attribute name, lowercased and possibly hyphenated.
 * @returns Camel-cased identifier suitable for a JS property name.
 *
 * @example
 * ```ts
 * camelOf('fetch-url') // 'fetchUrl'
 * camelOf('min-chars') // 'minChars'
 * camelOf('count')     // 'count'
 * ```
 *
 * @see defineComponent — schema entries map kebab → camel via this rule.
 */
export function camelOf(attr: string): string {
  return attr.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())
}
