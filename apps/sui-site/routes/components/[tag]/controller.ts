// /components/[tag] — per-component detail view controller.
//
// Reads the route param (the [tag] dynamic segment), looks up the component in
// the build-time `COMPONENTS_CATALOG`, and drives the "gallery detail" layout:
// a single big live preview stage on top, a row of variant chips (one per value
// of the component's primary enum prop — `tone` or `variant`), a title/meta
// block, and the API reference table.
//
// The preview is ONE mounted instance, not a grid: switching a variant chip
// flips the `activeVariant` signal, and a controller effect re-sets the primary
// prop attribute on the mounted element — no re-mount, no re-render. Seed props
// + default-slot innerHTML come from `DEMO_SEEDS`, applied via `bindHtml` (the
// morph-backed binding, NOT a raw innerHTML write — the sui-controller-purity
// lint forbids that).
//
// Framework-pure: no ship-platform coupling. The catalog CEM data is bundled at
// build time via deck's components.gen pass.

import { controller, signal, effect, bindHtml, ref, type ViewScope } from '@ship/sui-runtime'
import { COMPONENTS_CATALOG } from '../../../components.gen'
import type { Scope } from './page.sui'

// ── Types ────────────────────────────────────────────────────────────────────

/** Shape the components.gen.ts exposes for each prop. `values` is populated for
 *  union/enum props; absent on `string` / `number` / `boolean`. */
export interface PropMeta {
  name: string
  type: string
  default?: unknown
  values?: string[]
  description?: string
}
export interface NamedMeta { name: string; description?: string }

export interface Entry {
  tag: string
  pack: string
  description: string
  props: PropMeta[]
  slots: NamedMeta[]
  events: NamedMeta[]
}

// ── Catalog ingestion ────────────────────────────────────────────────────────

interface CatalogShape { modules?: CatalogModule[] }
interface CatalogModule { package?: string; declarations?: CatalogDeclaration[] }
interface CatalogDeclaration {
  kind?: string
  customElement?: boolean
  tagName?: string
  description?: string
  attributes?: CatalogAttribute[]
  slots?: Array<{ name?: string; description?: string }>
  events?: Array<{ name?: string; description?: string }>
}
interface CatalogAttribute {
  name?: string
  type?: { text?: string }
  default?: unknown
  values?: string[]
  description?: string
}

function isCatalogShape(value: unknown): value is CatalogShape {
  return typeof value === 'object' && value !== null
}
function readCatalogModules(value: unknown): CatalogModule[] {
  if (!isCatalogShape(value)) return []
  const modules = value.modules
  return Array.isArray(modules) ? (modules as CatalogModule[]) : []
}

/** Per-tag lookup table — built once at module load from the bundled catalog. */
const BY_TAG: Map<string, Entry> = (() => {
  const modules = readCatalogModules(COMPONENTS_CATALOG)
  const map = new Map<string, Entry>()
  for (const m of modules) {
    const pkg = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const entry: Entry = {
        tag: d.tagName,
        pack: pkg,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        props: (d.attributes ?? []).map((a): PropMeta => ({
          name: a.name ?? '',
          type: a.type?.text ?? 'string',
          ...(a.default === undefined ? {} : { default: a.default }),
          ...(a.values ? { values: a.values } : {}),
          ...(a.description ? { description: a.description } : {}),
        })),
        slots: (d.slots ?? []).map((s) => ({ name: s.name ?? '(default)', description: s.description })),
        events: (d.events ?? []).map((e) => ({ name: e.name ?? '', description: e.description })),
      }
      map.set(entry.tag, entry)
    }
  }
  return map
})()

// ── Demo seeds ───────────────────────────────────────────────────────────────

/** Per-tag demo seed — default attribute props + default-slot innerHTML so a
 *  bare instance renders something meaningful in the preview stage. */
const DEMO_SEEDS: Record<string, { props?: Record<string, string>; innerHTML?: string }> = {
  'sui-button':        { props: { tone: 'primary', variant: 'solid' }, innerHTML: 'Button' },
  'sui-icon':          { innerHTML: 'star' },
  'sui-badge':         { innerHTML: 'Badge' },
  'sui-chip':          { innerHTML: 'Chip' },
  'sui-card':          { innerHTML: '<h3 slot="title">Card title</h3><p>A SUI card preview.</p>' },
  'sui-alert':         { innerHTML: 'An informational alert.' },
  'sui-label':         { innerHTML: 'Label' },
  'sui-link':          { props: { href: '#' }, innerHTML: 'Link text' },
  'sui-button-group':  { innerHTML: '<sui-button tone="primary">One</sui-button><sui-button tone="secondary">Two</sui-button>' },
  'sui-empty-state':   { innerHTML: 'Nothing here yet.' },
  'sui-spinner':       { props: { label: 'Loading' } },
  'sui-tooltip':       { innerHTML: 'Tooltip text' },
  'sui-progress':      { props: { value: '60' } },
  'sui-avatar':        { props: { name: 'Ada Lovelace' } },
  'sui-account-chip':  { props: { name: 'Ada Lovelace' } },
  'sui-kbd':           { innerHTML: '⌘ K' },
  'sui-separator':     {},
  'sui-toast':         { props: { tone: 'info' }, innerHTML: 'Toast message' },
  'sui-skeleton':      {},
  'sui-skeleton-line': {},
  'sui-accordion':     { innerHTML: '<sui-accordion-item open><span slot="title">Item one</span>Body one.</sui-accordion-item><sui-accordion-item><span slot="title">Item two</span>Body two.</sui-accordion-item>' },
  'sui-tabs':          { innerHTML: '<sui-tab-list><sui-tab name="one">One</sui-tab><sui-tab name="two">Two</sui-tab></sui-tab-list><sui-tab-panel name="one">Panel one</sui-tab-panel><sui-tab-panel name="two">Panel two</sui-tab-panel>' },
  'sui-radio-group':   { innerHTML: '<sui-radio value="a">Alpha</sui-radio><sui-radio value="b">Beta</sui-radio>' },
  'sui-toggle-group':  { innerHTML: '<sui-toggle value="a">Alpha</sui-toggle><sui-toggle value="b">Beta</sui-toggle>' },
  'sui-checkbox':      { innerHTML: 'Accept terms' },
  'sui-switch':        {},
  'sui-slider':        { props: { value: '40' } },
  'sui-input':         { props: { placeholder: 'Type here…' } },
  'sui-textarea':      { innerHTML: 'Pre-filled text' },
  'sui-select':        { innerHTML: '<sui-option value="a">Alpha</sui-option><sui-option value="b">Beta</sui-option>' },
  'sui-combobox':      { innerHTML: '<sui-option value="a">Alpha</sui-option><sui-option value="b">Beta</sui-option>' },
  'sui-field':         { innerHTML: '<sui-label>Field label</sui-label><sui-input placeholder="Field input"></sui-input>' },
  'sui-form':          { innerHTML: '<sui-field label="Name"><sui-input></sui-input></sui-field>' },
  'sui-dialog':        { innerHTML: 'Dialog content' },
  'sui-sheet':         { innerHTML: 'Sheet content' },
  'sui-popover':       { innerHTML: 'Popover content' },
  'sui-dropdown-menu': { innerHTML: '<button>Open</button><div slot="content">Menu</div>' },
  'sui-context-menu':  { innerHTML: 'Right-click here' },
  'sui-tooltip-host':  { innerHTML: 'Hover me' },
  'sui-collapsible':   { innerHTML: '<span slot="trigger">Toggle</span><div>Hidden content.</div>' },
  'sui-hover-card':    { innerHTML: 'Hover over me' },
  'sui-markdown':      { innerHTML: '# Hello' },
  'sui-code':          { innerHTML: 'const x = 1' },
  'sui-codeblock':     { innerHTML: 'const x = 1' },
  'sui-json-view':     { innerHTML: '{"hello": "world"}' },
  'sui-diff':          { innerHTML: '- old\n+ new' },
  'sui-timeline':      { innerHTML: '<sui-timeline-item time="2025">First</sui-timeline-item><sui-timeline-item time="2026">Second</sui-timeline-item>' },
  'sui-tree':          { innerHTML: '<sui-tree-item>Root</sui-tree-item>' },
  'sui-tree-view':     { innerHTML: '<sui-tree-item>Root</sui-tree-item>' },
  'sui-tree-grid':     {},
  'sui-calendar':      { props: { value: '2026-07-01' } },
  'sui-cmdk':          { innerHTML: '<sui-cmdk-input placeholder="Search…"></sui-cmdk-input>' },
  'sui-table':         {},
  'sui-navigation-menu': { innerHTML: '<sui-navigation-item>Home</sui-navigation-item>' },
  'sui-pagination':    { props: { total: '50', page: '1' } },
  'sui-breadcrumbs':   { innerHTML: '<a>One</a><a>Two</a><a>Three</a>' },
  'sui-file-upload':   { innerHTML: 'Drop a file' },
  'sui-color-picker':  { props: { value: '#cc5500' } },
  'sui-date-picker':   { props: { value: '2026-07-01' } },
  'sui-stepper':       { innerHTML: '<sui-step>One</sui-step><sui-step>Two</sui-step>' },
  'sui-stat':          { innerHTML: '<span slot="label">Users</span><span slot="value">1,234</span>' },
  'sui-feature-card':  {},
  'sui-account-menu':  {},
  'sui-project-switcher': {},
  'sui-nav-top':       {},
  'sui-status-dot':    { props: { tone: 'success', label: 'OK' } },
  'sui-code-editor':   { props: { value: '// hello', lang: 'js' } },
  'sui-error-state':   { innerHTML: 'Something went wrong' },
}

// ── Primary axis + variant helpers ───────────────────────────────────────────

/** The "primary axis" prop the variant chips walk: prefer `tone`, then
 *  `variant`, else the enum with the most values. Null when the component has
 *  no enum props (then the preview is just the single default). */
function primaryEnumProp(entry: Entry): PropMeta | null {
  const enums = entry.props.filter((p) => Array.isArray(p.values) && p.values.length > 0)
  if (enums.length === 0) return null
  const named = (n: string): PropMeta | undefined => enums.find((p) => p.name === n)
  return (
    named('tone') ??
    named('variant') ??
    enums.reduce((a, b) => ((b.values?.length ?? 0) > (a.values?.length ?? 0) ? b : a))
  )
}

function defaultVariant(primary: PropMeta | null): string {
  if (!primary) return ''
  const values = primary.values ?? []
  const def = primary.default
  if (typeof def === 'string' && values.includes(def)) return def
  return values[0] ?? ''
}

// ── Controller ───────────────────────────────────────────────────────────────

export default controller<Scope>({
  scope: (ctx: ViewScope<Scope>) => {
    const tag = (): string => ctx.route.params.tag() ?? ''
    const entry = (): Entry | null => BY_TAG.get(tag()) ?? null

    const e0 = BY_TAG.get(typeof ctx.route.params.tag() === 'string' ? (ctx.route.params.tag() as string) : '') ?? null
    const primary0 = e0 ? primaryEnumProp(e0) : null

    const primaryProp = (): string => {
      const en = entry()
      if (!en) return ''
      return primaryEnumProp(en)?.name ?? ''
    }
    const variants = (): string[] => {
      const en = entry()
      if (!en) return []
      return primaryEnumProp(en)?.values ?? []
    }
    const activeVariant = signal<string>(defaultVariant(primary0))
    const setVariant = (v: string): void => { activeVariant.set(v) }
    const stageEl = ref<HTMLElement>(null)

    return { tag, entry, primaryProp, variants, activeVariant, setVariant, stageEl }
  },

  mount(state, host) {
    // Mount a SINGLE bare instance of the registered custom element into the
    // preview stage (the `bind:this` ref). Seeded from DEMO_SEEDS; default-slot
    // innerHTML rendered via `bindHtml`. A controller effect keeps the primary
    // prop attribute in sync with `activeVariant` so clicking a variant chip
    // re-renders the same element in-place — no remount.
    //
    // Startup race: the stage ref lands on first template mount and the
    // customElements registry fills from the page chunk's static imports;
    // either may be last on a cold load, so poll on a 100ms cadence
    // (60 attempts = 6s) before giving up.
    const disposers: Array<() => void> = []
    let el: HTMLElement | null = null
    let pollHandle: ReturnType<typeof setTimeout> | null = null
    let attempts = 0

    const tryMount = (): void => {
      // `stageEl` is a ref (a signal) — the state proxy unwraps signal members,
      // so the raw accessor lives at `state.$signals.stageEl`. `entry` /
      // `primaryProp` are plain scope functions and are called directly.
      const stage = state.$signals.stageEl() as HTMLElement | null
      const e = state.entry()
      if (!stage || !e || !e.tag || !customElements.get(e.tag)) {
        if (attempts++ < 60) pollHandle = setTimeout(tryMount, 100)
        return
      }
      try {
        el = document.createElement(e.tag)
        const seed = DEMO_SEEDS[e.tag] ?? {}
        if (seed.props) {
          for (const [k, v] of Object.entries(seed.props)) el.setAttribute(k, v)
        }
        stage.appendChild(el)
        if (seed.innerHTML) {
          const html = seed.innerHTML
          disposers.push(bindHtml(el, () => html))
        }
        // Keep the primary prop in sync with the active variant chip. Read the
        // raw `activeVariant` signal (via `$signals`) inside the effect so the
        // effect subscribes and re-runs on every chip click.
        const prop = state.primaryProp()
        if (prop) {
          disposers.push(effect(() => {
            const value = state.$signals.activeVariant() as string
            if (el && value) el.setAttribute(prop, value)
          }))
        }
        stage.dataset.mountState = 'live'
      } catch {
        const note = document.createElement('span')
        note.className = 'preview-empty'
        note.textContent = 'not in bundle'
        stage.appendChild(note)
        el = note
        stage.dataset.mountState = 'unavailable'
      }
    }

    const raf = requestAnimationFrame(tryMount)
    return () => {
      cancelAnimationFrame(raf)
      if (pollHandle) clearTimeout(pollHandle)
      for (const dispose of disposers) dispose()
      if (el) el.remove()
    }
  },
})
