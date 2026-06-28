// component() — declarative custom-element factory.
//
// Sits on top of `defineComponent`: takes a PropDef-typed declaration
// plus an optional setup, translates it into a `ComponentSchema` +
// `ComponentFactory`, and registers the element. Reflect/bindable
// behavior, style injection (light DOM scoped via `data-component`,
// shadow DOM attached to shadowRoot), and form-association are all
// handled by the wrapped factory so component() can stay declarative
// and the underlying defineComponent stays unchanged.

import { defineComponent, camelOf } from './component'
import type { ComponentSchema } from './component'
import { decoderFor } from './component-decoders'
import { clearFormHooks, installFormHooks } from './component-form'
import type { Decoder } from './decode'
import { effect, signal, type Signal } from './signal'
import type { MountSetup, MountModule } from './component-mount'

/** Declarative description of a component prop. */
export interface PropDef<T> {
  type: 'string' | 'bool' | 'number' | 'union' | 'object' | 'array'
  default?: T
  reflect?: boolean
  bindable?: boolean
  values?: readonly string[]
}

interface PropOpts<T> {
  default?: T
  reflect?: boolean
  bindable?: boolean
}

/**
 * Builders for declarative `PropDef` entries used by `component()`.
 *
 * Each builder returns a `PropDef<T>` describing the prop's type, an
 * optional default value, and whether the prop reflects to the host
 * attribute or participates in two-way binding.
 *
 * @example
 * ```ts
 * component('my-toggle', {
 *   props: {
 *     open: prop.bool({ default: false, reflect: true }),
 *     size: prop.union(['sm', 'md', 'lg'] as const, { default: 'md' }),
 *   },
 *   setup: ({ props }) => ({ open: props.open }),
 * })
 * ```
 */
export const prop = {
  /** String-valued prop. Default `''`. */
  string(opts: PropOpts<string> = {}): PropDef<string> {
    return { type: 'string', ...opts }
  },
  /** Boolean prop using HTML attribute-presence semantics. */
  bool(opts: PropOpts<boolean> = {}): PropDef<boolean> {
    return { type: 'bool', ...opts }
  },
  /** Numeric prop. Empty/missing decode to `NaN`. */
  number(opts: PropOpts<number> = {}): PropDef<number> {
    return { type: 'number', ...opts }
  },
  /**
   * Enum-style prop. Values outside `values` (including missing) decode
   * to `default` if provided, otherwise the first allowed value.
   */
  union<V extends readonly string[]>(
    values: V,
    opts: PropOpts<V[number]> = {},
  ): PropDef<V[number]> {
    return { type: 'union', values, ...opts }
  },
  /** JSON-parsed object prop. Bad/missing input falls back to `default`. */
  object<T>(opts: PropOpts<T> = {}): PropDef<T> {
    return { type: 'object', ...opts }
  },
  /** JSON-parsed array prop. Bad/missing input falls back to `default ?? []`. */
  array<T>(opts: PropOpts<T[]> = {}): PropDef<T[]> {
    return { type: 'array', ...opts }
  },
}

/** The setup callback's context. Mirrors `ComponentFactory` in shape. */
export interface ComponentSetupContext {
  /** Map of camelCased prop names to their reactive signals. */
  props: Record<string, Signal<unknown>>
  /** The host element (or its shadow root's host — same node either way). */
  host: HTMLElement
  /**
   * Push cleanup callbacks onto this array; they all run on disconnect
   * regardless of order.
   */
  cleanups: Array<() => void>
  /** Present when `formAssociated: true` — the host's ElementInternals. */
  internals?: ElementInternals
}

/** Declarative shape passed to `component()`. */
export interface ComponentDef<Bindings = unknown> {
  /** CSS injected scoped to this component (or attached to shadowRoot). */
  styles?: string
  /** When true, attach a shadow root and inject styles into it. */
  shadow?: boolean
  /** When true, declare `static formAssociated = true` on the element. */
  formAssociated?: boolean
  /**
   * Name of the prop (kebab-case attribute) whose signal participates
   * in the host's form value. The runtime calls
   * `internals.setFormValue(String(propSignal()))` whenever the signal
   * changes, plus auto-wires `formResetCallback` to restore the prop's
   * declared default and `formStateRestoreCallback` to decode the
   * persisted state through the prop's decoder.
   */
  formValue?: string
  /**
   * Name of the bool-typed prop (kebab-case attribute) whose signal is
   * driven by the browser's `formDisabledCallback(disabled)` call.
   */
  formDisabled?: string
  /** Prop declarations keyed by attribute name (kebab-case). */
  props: Record<string, PropDef<unknown>>
  /**
   * Optional setup. Called once on connect with prop signals + host,
   * receives a `cleanups` array to push teardowns onto, and (for
   * form-associated components) the host's `ElementInternals`.
   */
  setup?: (ctx: ComponentSetupContext) => Bindings
}

const encodeAttr = (def: PropDef<unknown>, value: unknown): string | null => {
  if (value === undefined || value === null) return null
  switch (def.type) {
    case 'string':
      return value === '' ? null : String(value)
    case 'number':
      return Number.isNaN(value as number) ? null : String(value)
    case 'bool':
      return value ? '' : null
    case 'union':
      return String(value)
    case 'object':
    case 'array':
      try { return JSON.stringify(value) } catch { return null }
  }
}

const injectScopedStyles = (tag: string, css: string): void => {
  const id = `ship-set-style-${tag}`
  if (document.getElementById(id)) return
  const style = document.createElement('style')
  style.id = id
  // CSS nesting wraps every rule in the scope selector. The user's
  // styles ride inside without further preprocessing.
  style.textContent = `[data-component="${tag}"] { ${css} }`
  document.head.appendChild(style)
}

const seedSignal = (
  host: HTMLElement,
  attr: string,
  def: PropDef<unknown>,
  decode: Decoder<unknown>,
): Signal<unknown> => {
  const raw = host.getAttribute(attr)
  const initial = raw !== null ? decode(raw) : (def.default as unknown)
  return signal(initial)
}

const buildSchema = (def: ComponentDef): ComponentSchema => {
  const observedAttrs: string[] = []
  const props: ComponentSchema['props'] = {}
  for (const [attr, propDef] of Object.entries(def.props ?? {})) {
    observedAttrs.push(attr)
    props[attr] = { camel: camelOf(attr), decode: decoderFor(propDef) }
  }
  // `formValue` / `formDisabled` imply form-associated even if the
  // author omitted the explicit flag — the compiler emits both, but
  // hand-written defs deserve the same DWIM.
  const associated =
    def.formAssociated === true ||
    def.formValue !== undefined ||
    def.formDisabled !== undefined
  const schema: ComponentSchema = { observedAttrs, props }
  if (associated) schema.formAssociated = true
  return schema
}

/**
 * `component(...)` — the one factory a compiled `.sui` exports. Two shapes,
 * disjoint on the first argument so the compiler (and the rare direct caller)
 * can never confuse them:
 *
 *   • `component(setup)` — tagless MOUNT form. Returns `{ setup }`; the host
 *     (deck / `attach()`) mounts it into a node. No custom-element registration.
 *     This is what the compiler emits for a tagless `.sui` (a deck page).
 *   • `component(tag, def)` — REGISTER a `<tag>` custom element from a
 *     declarative definition (this overload, documented below). A hyphenated
 *     tag is the only thing that turns a component into a registered element.
 *
 * `component(tag, def)` desugars `def` into the lower-level `ComponentSchema` +
 * `ComponentFactory` shape that `defineComponent` consumes. Props become
 * observed attributes with per-type decoders; `styles` are injected
 * (scoped via `[data-component="<tag>"]` in light DOM, attached to the
 * shadow root in shadow DOM); `formAssociated` is forwarded; and `setup`
 * runs on connect with prop signals plus a `cleanups` array for
 * teardown.
 *
 * Idempotent — calling twice with the same tag is a no-op.
 *
 * @param tag Custom element tag name (must contain a hyphen).
 * @param def Declarative description (props, styles, shadow, setup, …).
 *
 * @example
 * ```ts
 * component('my-counter', {
 *   props: {
 *     'initial-count': prop.number({ default: 0, reflect: true }),
 *   },
 *   setup: ({ props, cleanups }) => {
 *     const count = props.initialCount as Signal<number>
 *     return { count }
 *   },
 * })
 * ```
 *
 * @see defineComponent for the lower-level imperative API.
 */
export function component<Bindings>(setup: MountSetup<Bindings>): MountModule<Bindings>
export function component<Bindings>(tag: string, def: ComponentDef<Bindings>): void
export function component<Bindings>(
  tagOrSetup: string | MountSetup<Bindings>,
  maybeDef?: ComponentDef<Bindings>,
): MountModule<Bindings> | void {
  // One function argument → the tagless MOUNT form: a component mounted into a
  // host (deck page / synthetic render), not registered as a custom element.
  // Returns the `{ setup }` module `attach()` consumes. No tag, no define.
  if (typeof tagOrSetup === 'function') {
    return { setup: tagOrSetup }
  }
  // Two arguments (tag + def) → REGISTER a custom element.
  const tag = tagOrSetup
  const def = maybeDef as ComponentDef<Bindings>
  const schema = buildSchema(def as ComponentDef)
  const propEntries = Object.entries(def.props ?? {})

  defineComponent(tag, schema, (host, propSignals, internals) => {
    let shadowAttached: ShadowRoot | null = null
    if (def.shadow) {
      shadowAttached = host.shadowRoot ?? host.attachShadow({ mode: 'open' })
      if (def.styles) {
        const style = document.createElement('style')
        style.textContent = def.styles
        shadowAttached.appendChild(style)
      }
    } else if (def.styles) {
      injectScopedStyles(tag, def.styles)
    }

    const cleanups: Array<() => void> = []

    for (const [attr, propDef] of propEntries) {
      const camel = camelOf(attr)
      const decode = schema.props[attr]!.decode
      const sig = seedSignal(host, attr, propDef, decode)
      propSignals[camel] = sig
      if (propDef.reflect) {
        let first = true
        const dispose = effect(() => {
          const v = sig()
          // Skip the initial pass — we don't want to overwrite the host
          // attribute the user already provided (or the default-derived
          // null) on first connect.
          if (first) {
            first = false
            return
          }
          const next = encodeAttr(propDef, v)
          if (next === null) host.removeAttribute(attr)
          else host.setAttribute(attr, next)
        })
        cleanups.push(dispose)
      }
    }

    // Wire form association BEFORE setup runs so author setup code can
    // still reach `internals` for setValidity/checkValidity without
    // colliding with the auto-wiring. The setFormValue effect is
    // captured into the local cleanup list so disconnect tears it down.
    if (def.formValue !== undefined || def.formDisabled !== undefined) {
      const stopValue = installFormHooks(host, internals, def as ComponentDef, propSignals)
      if (typeof stopValue === 'function') cleanups.push(stopValue)
    }

    if (def.setup) {
      const ctx: ComponentSetupContext = { props: propSignals, host, cleanups }
      if (internals) ctx.internals = internals
      def.setup(ctx)
    }

    return () => {
      clearFormHooks(host)
      for (const c of cleanups.splice(0)) {
        try { c() } catch (e) { console.error(`[ship-set:${tag}] cleanup threw:`, e) }
      }
    }
  })
}
