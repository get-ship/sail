// /components/[tag] controller — the per-component detail view.
//
// Reads the route param (the [tag] dynamic segment), looks up the component
// in the build-time `COMPONENTS_CATALOG`, and projects it into a variant
// gallery: one live example per CEM enum value, plus the canonical usage
// snippet, then the API table.
//
// The variant gallery is the preview surface — there is no separate single-
// element stage. The view-source shows every value of the primary enum prop
// rendered at natural size, packed by <sui-grid>, each card carrying the
// code so the eye can compare variant + snippet side by side.
//
// Framework-pure: no ship-platform coupling. The catalog CEM data is bundled
// at build time via deck's `generate_components_gen` pass.

import { controller, bindHtml, ref, type ViewScope } from '@ship/sui-runtime'
import { COMPONENTS_CATALOG } from '../../../components.gen'
import { code as codeSnippets } from '../snippets'
import type { Scope } from './page.sui'

// ── Types ──────────────────────────────────────────────────────────────────

/** Shape the components.gen.ts exposes for each prop. `values` is populated
 *  for union/enum props; absent on `string` / `number` / `boolean`. The
 *  variant gallery walks the values array, not the type string. */
export interface PropMeta {
  name: string
  type: string
  default?: unknown
  values?: string[]
}
export interface NamedMeta { name: string; description?: string }

/** One entry in the catalog — narrow, read-only here. The table iterates
 *  the raw arrays; no derivation needed. */
export interface Entry {
  tag: string
  pkg: string
  description: string
  props: PropMeta[]
  slots: NamedMeta[]
  events: NamedMeta[]
}

/** A single card in the variant gallery. The mount step in the controller
 *  reads `props` + `innerHTML` and instantiates a bare custom element;
 *  `code` is the rendered HTML for the variant, shown in a <sui-code>
 *  block under the live preview. `label` is the small mono caption
 *  (e.g. "tone: primary"). `isCanonical` flags the snippet-driven example
 *  so the layout can give it the same span as a derived variant. */
export interface Variant {
  key: string
  label: string
  props: Record<string, string>
  innerHTML: string
  code: string
  isCanonical: boolean
  /** When true, the preview gets a tall min-height (the natural size
   *  of calendar / table / tree variants). The CSS uses `data-tall`
   *  on the variant card to bump the preview floor. */
  tall: boolean
}

// ── Catalog ingestion ──────────────────────────────────────────────────────

/** A module row from the generated catalog — narrowed via the *real* shape
 *  the catalog exposes. We walk it with a type guard rather than an inline
 *  cast (no `(x as { modules: ... }).modules` — the access is checked). */
interface CatalogShape {
  schemaVersion?: string
  modules?: CatalogModule[]
}
interface CatalogModule {
  package?: string
  declarations?: CatalogDeclaration[]
}
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
}

function isCatalogShape(value: unknown): value is CatalogShape {
  return typeof value === 'object' && value !== null
}

function readCatalogModules(value: unknown): CatalogModule[] {
  if (!isCatalogShape(value)) return []
  if (!('modules' in value)) return []
  const modules = value.modules
  return Array.isArray(modules) ? (modules as CatalogModule[]) : []
}

/** Per-tag lookup table — built once at module load from the bundled
 *  catalog. Pure data transform, no per-render cost. */
const BY_TAG: Map<string, Entry> = (() => {
  const modules = readCatalogModules(COMPONENTS_CATALOG)
  const map = new Map<string, Entry>()
  for (const m of modules) {
    const pkg = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const entry: Entry = {
        tag: d.tagName,
        pkg,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        props: (d.attributes ?? []).map((a): PropMeta => ({
          name: a.name ?? '',
          type: a.type?.text ?? 'string',
          ...(a.default === undefined ? {} : { default: a.default }),
          ...(a.values ? { values: a.values } : {}),
        })),
        slots: (d.slots ?? []).map((s) => ({ name: s.name ?? '(default)', description: s.description })),
        events: (d.events ?? []).map((e) => ({ name: e.name ?? '', description: e.description })),
      }
      map.set(entry.tag, entry)
    }
  }
  return map
})()

// ── Demo seeds ─────────────────────────────────────────────────────────────

/** Per-tag demo seed — default attribute props + default-slot innerHTML.
 *  Verbatim from the prior hand-rolled controller so the visual surface
 *  matches the rest of the site. */
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

// ── Variant gallery derivation ─────────────────────────────────────────────

/** Map a sui-* tag to its snippet key (the `code` map strips the prefix). */
const SNIPPET_KEY_FOR_TAG = (tag: string): string => tag.replace(/^sui-/, '')

/** Render a variant's HTML into a single-line snippet string. The output
 *  is the exact markup the consumer pastes, so what-you-see is what-
 *  you-paste. Attributes are sorted for stable diffs. */
function renderVariantCode(tag: string, props: Record<string, string>, inner: string): string {
  const keys = Object.keys(props).sort()
  const attrStr = keys.map((k) => [k, '="', props[k], '"'].join('')).join(' ')
  const open = attrStr ? '<' + tag + ' ' + attrStr + '>' : '<' + tag + '>'
  if (!inner) return [open, '<', '/', tag, '>'].join('')
  return [open, inner, '<', '/', tag, '>'].join('')
}

/** Build the variant gallery for one component entry. The first enum prop
 *  is the primary "axis" — we render one example per value, with all
 *  other enum props pinned to their CEM default. If the component has no
 *  enum props we fall back to a single canonical variant driven by the
 *  hand-curated `code` snippet. */
function buildVariants(entry: Entry): Variant[] {
  const seed = DEMO_SEEDS[entry.tag] ?? {}
  const seedProps = seed.props ?? {}
  const seedInner = seed.innerHTML ?? ''
  const snippetKey = SNIPPET_KEY_FOR_TAG(entry.tag)
  const canonicalCode = codeSnippets[snippetKey] ?? ''

  // Tags that render big specimens — bump the variant card's preview
  // height to fit the natural size. The names mirror the catalog's
  // PREVIEW_SEEDS `size: 'tall'` set so the two surfaces agree.
  const TALL_TAGS = new Set([
    'sui-calendar', 'sui-timeline', 'sui-tree', 'sui-tree-view',
    'sui-tree-grid', 'sui-table',
  ])
  const isTall = TALL_TAGS.has(entry.tag)

  const enumProps = entry.props.filter((p) => Array.isArray(p.values) && p.values.length > 0)

  // No enum props — single canonical example.
  if (enumProps.length === 0) {
    const props = { ...seedProps }
    return [{
      key: 'canonical',
      label: 'Default',
      props,
      innerHTML: seedInner,
      code: canonicalCode || renderVariantCode(entry.tag, props, seedInner),
      isCanonical: true,
      tall: isTall,
    }]
  }

  // Pick the primary axis: the enum with the most values wins. Bigger
  // value sets are usually the visually impactful "color" axis (tone,
  // variant, type) — the right one to gallery-ize.
  const primary = enumProps.reduce((a, b) =>
    (b.values?.length ?? 0) > (a.values?.length ?? 0) ? b : a,
  )

  const primaryValues = primary.values ?? []
  const otherDefaults: Record<string, string> = {}
  for (const p of enumProps) {
    if (p.name === primary.name) continue
    const def = p.default
    if (typeof def === 'string') otherDefaults[p.name] = def
  }

  return primaryValues.map((value) => {
    const props: Record<string, string> = { ...seedProps, ...otherDefaults, [primary.name]: value }
    return {
      key: `${primary.name}-${value}`,
      label: `${primary.name}: ${value}`,
      props,
      innerHTML: seedInner,
      code: renderVariantCode(entry.tag, props, seedInner),
      isCanonical: false,
      tall: isTall,
    }
  })
}

// ── Controller ─────────────────────────────────────────────────────────────
export default controller<Scope>({
  scope: (ctx: ViewScope<Scope>) => {
    const tag = (): string => ctx.route.params.tag() ?? ''
    const entry = (): Entry | null => BY_TAG.get(tag()) ?? null
    const variants = (): Variant[] => {
      const e = entry()
      return e ? buildVariants(e) : []
    }
    const variantsJson = (): string => JSON.stringify(variants())
    const gridEl = ref<HTMLElement>(null)
    return { tag, entry, variants, variantsJson, gridEl }
  },

  mount(state, host) {
    // Mount a bare instance of the registered custom element into every
    // [data-mount-for] slot the template renders, one per variant. The
    // seed innerHTML (if any) is rendered via the runtime's `bindHtml`
    // (the morph-backed binding, NOT a raw `el.innerHTML = …` which the
    // controller-purity rule forbids). Variants are passed as a JSON
    // blob in `data-variants` on the grid host so the mount step can
    // re-derive the same list without a separate channel — single
    // source of truth in `buildVariants`.
    //
    // Two startup races to bridge: (1) the grid ref is set by the
    // template's `bind:this` on first mount, (2) the `customElements`
    // registry is filled in by the page chunk's static imports. Either
    // may land first on a cold load, so we poll on a 100ms cadence
    // (60 attempts = 6s) before falling back to the "not in bundle"
    // empty state.
    const mounted: Element[] = []
    const binds: Array<() => void> = []
    let pollHandle: ReturnType<typeof setTimeout> | null = null
    let attempts = 0
    const tryMount = (): void => {
      const root = (host.shadowRoot ?? host) as ParentNode
      const grid = state.$signals.gridEl()
      const e = state.entry()
      if (!grid || !e || !e.tag || !customElements.get(e.tag)) {
        if (attempts++ < 60) {
          pollHandle = setTimeout(tryMount, 100)
        }
        return
      }
      const slots = root.querySelectorAll<HTMLElement>('[data-mount-for]')
      slots.forEach((slot) => {
        const key = slot.getAttribute('data-mount-for')
        const variant = state.variants().find((v) => v.key === key)
        if (!variant) return
        try {
          const el = document.createElement(e.tag)
          for (const [k, v] of Object.entries(variant.props)) el.setAttribute(k, v)
          slot.appendChild(el)
          if (variant.innerHTML) {
            const html = variant.innerHTML
            binds.push(bindHtml(el, () => html))
          }
          slot.dataset.mountState = 'live'
          mounted.push(el)
        } catch {
          const note = document.createElement('span')
          note.className = 'preview-empty'
          note.textContent = 'not in bundle'
          slot.appendChild(note)
          slot.dataset.mountState = 'unavailable'
          mounted.push(note)
        }
      })
    }
    const raf = requestAnimationFrame(tryMount)
    return () => {
      cancelAnimationFrame(raf)
      clearTimeout(pollHandle)
      for (const unbind of binds) unbind()
      for (const el of mounted) el.remove()
    }
  },
})
