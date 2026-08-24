// component() — declarative custom-element factory.
//
// Sits on top of `defineComponent`: takes a PropDef-typed declaration
// plus an optional setup, translates it into a `ComponentSchema` +
// `ComponentFactory`, and registers the element. Reflect/bindable
// behavior, style injection (light DOM scoped via `data-component` —
// the default), optional `shadow: true` only for foreign-content
// containment wrappers, and form-association are all handled by the
// wrapped factory so component() can stay declarative and the
// underlying defineComponent stays unchanged.

import { writeCssVar } from './bind'
import { defineComponent, camelOf } from './component'
import type { ComponentSchema } from './component'
import { decoderFor } from './component-decoders'
import { clearFormHooks, installFormHooks } from './component-form'
import type { Decoder } from './decode'
import { effect, signal, type Signal } from './signal'
import type { MountSetup, MountModule } from './component-mount'
import { mountComponent } from './compose'
import { navigate } from './navigate'
import type { RouteSignals } from './route-signals'
import { projectSlots } from './slot-project'
import { ensureLightContracts } from './styles/contracts-seam'
import { hostRoute } from './host-route'

/** Declarative description of a component prop. */
export interface PropDef<T> {
  type: 'string' | 'bool' | 'number' | 'union' | 'property'
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
  /**
   * A plain reactive property — any TypeScript type. NOT an attribute: it's a
   * signal + `el.prop` accessor, set via `el.prop = …` or `prop:` binding.
   * This is where object/array/custom-typed props live (identity preserved, no
   * JSON, no attribute). The compiler emits this for any non-primitive prop.
   */
  property<T>(opts: PropOpts<T> = {}): PropDef<T> {
    return { type: 'property', ...opts }
  },
}

/** The setup callback's context. Mirrors `ComponentFactory` in shape. */
export interface ComponentSetupContext {
  /** Map of camelCased prop names to their reactive signals. */
  props: Record<string, Signal<unknown>>
  /** The host element (the custom element itself — Sail is light DOM). */
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
  /** CSS injected scoped to this component (`@scope` donut in light DOM). */
  styles?: string
  /**
   * Residual: when true, attach a shadow root (foreign-content containment —
   * tiptap, third-party widgets). Not the L1 product model; default is light DOM.
   */
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
    case 'property':
      // Never reflected — a property prop is not an attribute.
      return null
  }
}

const injectScopedStyles = (tag: string, css: string): void => {
  const id = `sail-style-${tag}`
  if (document.getElementById(id)) return
  const style = document.createElement('style')
  style.id = id
  // Scope from this component host down through the light-DOM tree.
  //
  // Do NOT use a `to ([data-component])` lower boundary (the old "donut").
  // Light-DOM composition projects host-authored children into nested
  // components via <slot> (specimen | knobs | stage on sail-component-viewer).
  // Those nodes remain descendants of the nested host in the DOM, but they
  // are authored and styled by the PARENT .sail file — a lower boundary
  // made every host Style utility (s-w7 swatches, knob widths, …) a no-op
  // inside the viewer, collapsing tone dots to 0×0.
  //
  // Nested components still inject their own scoped sheets; same-specificity
  // atomic utilities match, and later sheets win when both apply. The
  // compiler rewrites :host → :scope / .sail-root for this wrapper.
  //
  // Must share `@layer sail` with element-skins.css. An unlayered @scope
  // restamp of `.s-bgmuted` beats layered `:checked` / `[tone]` fills
  // regardless of specificity (doc-switch crushed the track).
  style.textContent = `@layer sail { @scope ([data-component="${tag}"]) { ${css} } }`
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
    // Property props are plain reactive properties, not attributes: they get a
    // signal + accessor (below / in installPropAccessors) but are never
    // observed or decoded from an attribute.
    if (propDef.type !== 'property') observedAttrs.push(attr)
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
 * `component(...)` — the one factory a compiled `.sail` exports. Two shapes,
 * disjoint on the first argument so the compiler (and the rare direct caller)
 * can never confuse them:
 *
 *   • `component(setup)` — tagless MOUNT form. Returns `{ setup }`; the host
 *     (deck / `attach()`) mounts it into a node. No custom-element registration.
 *     This is what the compiler emits for a tagless `.sail` (a deck page).
 *   • `component(tag, def)` — REGISTER a `<tag>` custom element from a
 *     declarative definition (this overload, documented below). A hyphenated
 *     tag is the only thing that turns a component into a registered element.
 *
 * `component(tag, def)` desugars `def` into the lower-level `ComponentSchema` +
 * `ComponentFactory` shape that `defineComponent` consumes. Props become
 * observed attributes with per-type decoders; `styles` are injected
 * scoped via `[data-component="<tag>"]` (light DOM default); `formAssociated`
 * is forwarded; and `setup` runs on connect with prop signals plus a
 * `cleanups` array for teardown.
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
export function component<Bindings>(
  tag: string,
  def: ComponentDef<Bindings>,
  module: MountModule<Bindings>,
): void
export function component<Bindings>(
  tagOrSetup: string | MountSetup<Bindings>,
  maybeDef?: ComponentDef<Bindings>,
  maybeModule?: MountModule<Bindings>,
): MountModule<Bindings> | void {
  // One function argument → the tagless MOUNT form: a component mounted into a
  // host (deck page / internal child), not registered as a custom element.
  // Returns the `{ setup }` module the host (`attach` / `mountComponent`) consumes.
  if (typeof tagOrSetup === 'function') {
    return { setup: tagOrSetup }
  }
  // (tag, def, module) → REGISTER a custom element backed by `module`. The
  // wrapper seeds prop signals, injects styles, wires form association, then
  // mounts the SAME module via mountComponent on connect — the one engine the
  // page and internal-composition paths also use.
  const tag = tagOrSetup
  const def = maybeDef as ComponentDef<Bindings>
  const module = maybeModule as MountModule<Bindings>
  const schema = buildSchema(def as ComponentDef)
  const propEntries = Object.entries(def.props ?? {})

  defineComponent(tag, schema, (host, propSignals, internals) => {
    let shadowAttached: ShadowRoot | null = null
    // Light DOM (the default architecture): capture the host's original
    // children BEFORE the template renders — they are the slot content the
    // projection pass places after mount.
    let lightChildren: Node[] | null = null
    if (def.shadow) {
      shadowAttached = host.shadowRoot ?? host.attachShadow({ mode: 'open' })
      if (def.styles) {
        const style = document.createElement('style')
        style.textContent = def.styles
        shadowAttached.appendChild(style)
      }
    } else {
      // Capture ONCE per host element, not once per connect. The connect
      // handler re-runs when control flow ({#if}/{#match}) detaches and
      // re-attaches the SAME host node; at that point the host's children
      // are the previous render's own template output, and capturing THAT
      // as "authored content" makes projection try to place the rendered
      // wrapper inside its own <slot> — HierarchyRequestError, page dead
      // (the /components filter crash). The authored nodes stay alive here
      // by reference across the round-trip; projection moves them into the
      // fresh render's slots.
      const h = host as HTMLElement & { __suiLightCapture?: Node[] }
      lightChildren = h.__suiLightCapture ??= Array.from(host.childNodes)
      // Model A: the host is a transparent pass-through — its `.sail-root`
      // template child is the visual root. `display: contents` removes the
      // host's own box so a block-rooted component lays out correctly (an
      // inline custom-element host would otherwise fail to contain a block
      // child). The host stays in the DOM — and, in modern engines, in the a11y
      // tree — carrying `data-component` + reflected attrs for the `@scope`
      // donut and `:scope[state] .sail-root` selectors.
      writeCssVar(host, 'display', 'contents')
      ensureLightContracts()
      if (def.styles) injectScopedStyles(tag, def.styles)
    }

    const cleanups: Array<() => void> = []

    for (const [attr, propDef] of propEntries) {
      const camel = camelOf(attr)
      const decode = schema.props[attr]!.decode
      const sig = seedSignal(host, attr, propDef, decode)
      propSignals[camel] = sig
      if (propDef.reflect) {
        // Reflect the initial value too: variant-scoped skin CSS is
        // attribute-gated (`[variant='drop'].s-variant-drop-…`), and a host
        // mounted without the attribute (default-only use) would never match
        // its own skin rules. Only skip when the user already set one.
        const initialRaw = host.getAttribute(attr)
        if (initialRaw === null) {
          const initialNext = encodeAttr(propDef, sig())
          if (initialNext !== null) host.setAttribute(attr, initialNext)
        }
        let first = false
        const dispose = effect(() => {
          const v = sig()
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

    // Mount the module via the single engine: `setup(ctx)` seeds $scope from
    // `ctx.props`, then `__mount(host)` renders and the controller lifecycle
    // (LIFECYCLE_KEY) wires. `internals` rides along for form-associated
    // controllers. The returned disposer tears the child down on disconnect.
    //
    // `host` MUST ride on `ctx`: `resolveController` passes it as the
    // controller scope factory's 2nd arg (imperative mount of a third-party
    // editor, focus an input, etc.). The runtime no longer auto-publishes
    // shared signals on the element — that's scope-homed now via
    // `useStore` / `provideContext` (see `./store` / `./effect-scope`), so
    // `host` here is for imperative reach, not shared-state publication.
    //
    // `route` / `navigate` match the page mount contract: compiled setup
    // does `$scope.route = route`. The host publishes the live RouteSignals
    // on `__shipRouteSignals` (same port pattern as `__shipRouterPort`).
    // Both CE class paths (form-associated + normal) share this factory,
    // so both see the same object — reused chrome updates on navigate
    // without remounting. Isolated unit tests leave the port unset.
    const route = globalThis.__shipRouteSignals as RouteSignals | undefined
    const ctx: {
      props: Record<string, Signal<unknown>>
      host: HTMLElement
      navigate: typeof navigate
      route?: RouteSignals
      internals?: ElementInternals
    } = {
      props: propSignals,
      host,
      navigate,
    }
    if (route) ctx.route = route
    if (internals) ctx.internals = internals
    // Deck injects ctx.route for layout CEs (and any other CE whose host
    // registered a route). Pages already receive route via mount(); this
    // is the CE connect path that used to leave route undefined.
    const injected = hostRoute()
    if (injected) {
      ctx.route = injected.route
      ctx.navigate = injected.navigate
    }
    // `mountComponent` treats the module opaquely (it only reads `setup`); the
    // cast erases the per-component `Bindings` to the default module shape.
    cleanups.push(mountComponent(module as MountModule, host, ctx))
    // Light DOM: the template has rendered into the host — place the
    // captured original children into their <slot> wrappers. Projection is
    // LIVE for the component's lifetime: when control flow swaps a branch
    // containing a <slot> back in, the new slot arrives empty while the
    // captured nodes still exist (held here) — re-project so slotted content
    // survives {#if}/{#match} round-trips (the error-boundary retry flow).
    if (lightChildren) {
      // Projection sources: the connect-time capture, plus any slot-attributed
      // direct children that appeared since (a controller may synthesize
      // content, e.g. the boundary's fallback wrapper). Collected fresh per
      // projection so synthesized content is held and placed like authored
      // content instead of rendering unslotted.
      const extras: Node[] = []
      const project = (): void => {
        for (const child of Array.from(host.children)) {
          if (
            child.tagName !== 'TEMPLATE' &&
            child.getAttribute('slot') &&
            !lightChildren.includes(child) &&
            !extras.includes(child) &&
            child.closest('slot') === null
          ) {
            extras.push(child)
          }
        }
        projectSlots(host, [...lightChildren, ...extras])
      }
      project()
      const h = host as HTMLElement & { __suiReproject?: () => void }
      h.__suiReproject = project
      cleanups.push(() => {
        delete h.__suiReproject
      })
    }

    return () => {
      clearFormHooks(host)
      for (const c of cleanups.splice(0)) {
        try { c() } catch (e) { console.error(`[sail:${tag}] cleanup threw:`, e) }
      }
    }
  })
}
