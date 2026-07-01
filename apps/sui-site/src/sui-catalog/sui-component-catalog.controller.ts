// <sui-component-catalog> — `/components` index. Reads the build-time
// `COMPONENTS_CATALOG` (emitted by `deck build`'s cem pass into
// `apps/sui-site/components.gen.ts`) and groups every component by source
// pack. Framework-pure — no ship-platform coupling.
//
// After the template renders the per-component preview slots, the controller
// live-mounts a bare instance of every tag the bundle actually registered
// (`customElements.get`). A tag that isn't in the bundle still lists with
// its metadata — coverage over polish.
//
// The catalog grid is content-driven: per-tag cell sizing (compact /
// normal / wide / tall) + demo seed (default props + slot content)
// are looked up in `PREVIEW_SEEDS` so big components (calendar, table,
// tree, code-editor) get the room they need and small ones (status-dot,
// badge, switch) stay compact. Tags absent from the map fall through to
// a bare createElement with no slot content.

import { controller, bindHtml, derived } from '@ship/sui-runtime'
import type { Scope } from './sui-component-catalog.sui'
import { COMPONENTS_CATALOG } from '../../components.gen'

interface PropMeta { name: string; type: string; default?: unknown }
interface NamedMeta { name: string; description?: string }
export interface ComponentEntry {
  tag: string
  pkg: string
  description: string
  props: PropMeta[]
  slots: NamedMeta[]
  events: NamedMeta[]
  size: 'compact' | 'normal' | 'wide' | 'tall'
}
export interface PackGroup {
  id: string
  pkg: string
  count: number
  components: ComponentEntry[]
}

interface CemAttr { name?: string; type?: { text?: string }; default?: unknown }
interface CemDecl {
  kind?: string
  customElement?: boolean
  tagName?: string
  description?: string
  attributes?: CemAttr[]
  slots?: Array<{ name?: string; description?: string }>
  events?: Array<{ name?: string; description?: string }>
  members?: unknown[]
}
interface CemModule { package?: string; declarations?: CemDecl[] }

/** Framework `@ship/sui-*` packs sort before app/domain `@ship/*` packs;
 *  ties break alphabetically. A stable, readable pack order for the browser. */
function packRank(pkg: string): number {
  return pkg.startsWith('@ship/sui-') ? 0 : 1
}

/** Per-tag cell size + preview seed. A bare `document.createElement` on
 *  most primitives renders nothing meaningful (no slotted content, no
 *  useful default props), so the seed map supplies both:
 *  - `size` → cell className (`compact` / `normal` / `wide` / `tall`)
 *  - `props` → setAttribute values for the bare element
 *  - `innerHTML` → default slot content (rendered via bindHtml, NOT
 *    a raw innerHTML write which the sui-controller-purity lint forbids)
 *  Tags absent from the map fall through to a bare createElement with
 *  no slot content. */
export interface PreviewSeed {
  size: 'compact' | 'normal' | 'wide' | 'tall'
  props?: Record<string, string>
  innerHTML?: string
}
export const PREVIEW_SEEDS: Record<string, PreviewSeed> = {
  'sui-button':        { size: 'normal', props: { tone: 'primary', variant: 'solid' }, innerHTML: 'Button' },
  'sui-icon':          { size: 'compact', innerHTML: 'star' },
  'sui-badge':         { size: 'compact', innerHTML: 'Badge' },
  'sui-chip':          { size: 'compact', innerHTML: 'Chip' },
  'sui-card':          { size: 'wide', innerHTML: '<h3 slot="title">Card title</h3><p>A SUI card preview.</p>' },
  'sui-alert':         { size: 'normal', innerHTML: 'An informational alert.' },
  'sui-label':         { size: 'compact', innerHTML: 'Label' },
  'sui-link':          { size: 'normal', props: { href: '#' }, innerHTML: 'Link text' },
  'sui-button-group':  { size: 'normal', innerHTML: '<sui-button tone="primary">One</sui-button><sui-button tone="secondary">Two</sui-button>' },
  'sui-empty-state':   { size: 'normal', innerHTML: 'Nothing here yet.' },
  'sui-spinner':       { size: 'compact', props: { label: 'Loading' } },
  'sui-tooltip':       { size: 'compact', innerHTML: 'Tooltip text' },
  'sui-progress':      { size: 'normal', props: { value: '60' } },
  'sui-avatar':        { size: 'compact', props: { name: 'Ada Lovelace' } },
  'sui-account-chip':  { size: 'compact', props: { name: 'Ada Lovelace' } },
  'sui-kbd':           { size: 'compact', innerHTML: '⌘ K' },
  'sui-separator':     { size: 'compact' },
  'sui-toast':         { size: 'normal', props: { tone: 'info' }, innerHTML: 'Toast message' },
  'sui-skeleton':      { size: 'normal' },
  'sui-skeleton-line': { size: 'normal' },
  'sui-accordion':     { size: 'wide', innerHTML: '<sui-accordion-item open><span slot="title">Item one</span>Body one.</sui-accordion-item><sui-accordion-item><span slot="title">Item two</span>Body two.</sui-accordion-item>' },
  'sui-tabs':          { size: 'wide', innerHTML: '<sui-tab-list><sui-tab name="one">One</sui-tab><sui-tab name="two">Two</sui-tab></sui-tab-list><sui-tab-panel name="one">Panel one</sui-tab-panel><sui-tab-panel name="two">Panel two</sui-tab-panel>' },
  'sui-radio-group':   { size: 'normal', innerHTML: '<sui-radio value="a">Alpha</sui-radio><sui-radio value="b">Beta</sui-radio>' },
  'sui-toggle-group':  { size: 'normal', innerHTML: '<sui-toggle value="a">Alpha</sui-toggle><sui-toggle value="b">Beta</sui-toggle>' },
  'sui-checkbox':      { size: 'compact', innerHTML: 'Accept terms' },
  'sui-switch':        { size: 'compact' },
  'sui-slider':        { size: 'normal', props: { value: '40' } },
  'sui-input':         { size: 'normal', props: { placeholder: 'Type here…' } },
  'sui-textarea':      { size: 'wide', innerHTML: 'Pre-filled text' },
  'sui-select':        { size: 'normal', innerHTML: '<sui-option value="a">Alpha</sui-option><sui-option value="b">Beta</sui-option>' },
  'sui-combobox':      { size: 'normal', innerHTML: '<sui-option value="a">Alpha</sui-option><sui-option value="b">Beta</sui-option>' },
  'sui-field':         { size: 'wide', innerHTML: '<sui-label>Field label</sui-label><sui-input placeholder="Field input"></sui-input>' },
  'sui-form':          { size: 'wide', innerHTML: '<sui-field label="Name"><sui-input></sui-input></sui-field>' },
  'sui-dialog':        { size: 'wide', innerHTML: 'Dialog content' },
  'sui-sheet':         { size: 'wide', innerHTML: 'Sheet content' },
  'sui-popover':       { size: 'wide', innerHTML: 'Popover content' },
  'sui-dropdown-menu': { size: 'normal', innerHTML: '<button>Open</button><div slot="content">Menu</div>' },
  'sui-context-menu':  { size: 'normal', innerHTML: 'Right-click here' },
  'sui-tooltip-host':  { size: 'compact', innerHTML: 'Hover me' },
  'sui-collapsible':   { size: 'normal', innerHTML: '<span slot="trigger">Toggle</span><div>Hidden content.</div>' },
  'sui-hover-card':    { size: 'normal', innerHTML: 'Hover over me' },
  'sui-markdown':      { size: 'wide', innerHTML: '# Markdown' },
  'sui-code':          { size: 'normal', innerHTML: 'const x = 1' },
  'sui-codeblock':     { size: 'normal', innerHTML: 'const x = 1' },
  'sui-json-view':     { size: 'wide', innerHTML: '{"hello": "world"}' },
  'sui-diff':          { size: 'wide', innerHTML: '- old\n+ new' },
  'sui-timeline':      { size: 'tall', innerHTML: '<sui-timeline-item time="2025">First</sui-timeline-item><sui-timeline-item time="2026">Second</sui-timeline-item>' },
  'sui-tree':          { size: 'tall', innerHTML: '<sui-tree-item>Root</sui-tree-item>' },
  'sui-tree-view':     { size: 'tall', innerHTML: '<sui-tree-item>Root</sui-tree-item>' },
  'sui-tree-grid':     { size: 'tall' },
  'sui-calendar':      { size: 'tall', props: { value: '2026-07-01' } },
  'sui-cmdk':          { size: 'wide', innerHTML: '<sui-cmdk-input placeholder="Search…"></sui-cmdk-input>' },
  'sui-table':         { size: 'tall' },
  'sui-navigation-menu': { size: 'wide', innerHTML: '<sui-navigation-item>Home</sui-navigation-item>' },
  'sui-pagination':    { size: 'normal', props: { total: '50', page: '1' } },
  'sui-breadcrumbs':   { size: 'normal', innerHTML: '<a>One</a><a>Two</a><a>Three</a>' },
  'sui-file-upload':   { size: 'normal', innerHTML: 'Drop a file' },
  'sui-color-picker':  { size: 'compact', props: { value: '#cc5500' } },
  'sui-date-picker':   { size: 'normal', props: { value: '2026-07-01' } },
  'sui-stepper':       { size: 'normal', innerHTML: '<sui-step>One</sui-step><sui-step>Two</sui-step>' },
  'sui-stat':          { size: 'normal', innerHTML: '<span slot="label">Users</span><span slot="value">1,234</span>' },
  'sui-feature-card':  { size: 'wide' },
  'sui-account-menu':  { size: 'normal' },
  'sui-project-switcher': { size: 'normal' },
  'sui-nav-top':       { size: 'wide' },
  'sui-status-dot':    { size: 'compact', props: { tone: 'success', label: 'OK' } },
  'sui-code-editor':   { size: 'wide', props: { value: '// hello', lang: 'js' } },
  'sui-error-state':   { size: 'normal', innerHTML: 'Something went wrong' },
}

/** Project the generated CEM into pack-grouped component entries. Pure +
 *  synchronous — the catalog is bundled data, computed once at module load.
 *  Each entry's `size` comes from `PREVIEW_SEEDS[tag]?.size ?? 'normal'`
 *  (the default covers everything in the CEM that doesn't have an explicit
 *  override). */
function buildGroups(): PackGroup[] {
  const modules = (COMPONENTS_CATALOG as { modules?: CemModule[] }).modules ?? []
  const byPack = new Map<string, ComponentEntry[]>()
  for (const m of modules) {
    const pkg = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const entry: ComponentEntry = {
        tag: d.tagName,
        pkg,
        description: ((d.description ?? '').split('\n')[0]?.trim()) ?? '',
        props: (d.attributes ?? []).map((a): PropMeta => ({
          name: a.name ?? '',
          type: a.type?.text ?? 'string',
          ...(a.default === undefined ? {} : { default: a.default }),
        })),
        slots: (d.slots ?? []).map((s): NamedMeta => ({ name: s.name ?? '(default)', description: s.description })),
        events: (d.events ?? []).map((e): NamedMeta => ({ name: e.name ?? '', description: e.description })),
        size: PREVIEW_SEEDS[d.tagName]?.size ?? 'normal',
      }
      const list = byPack.get(pkg) ?? []
      list.push(entry)
      byPack.set(pkg, list)
    }
  }
  return [...byPack.entries()]
    .map(([pkg, list]): PackGroup => ({
      id: pkg.replace(/[^a-z0-9]+/gi, '-'),
      pkg,
      count: list.length,
      components: list.sort((a, b) => a.tag.localeCompare(b.tag)),
    }))
    .sort((a, b) => packRank(a.pkg) - packRank(b.pkg) || a.pkg.localeCompare(b.pkg))
}

const GROUPS: PackGroup[] = buildGroups()
const TOTAL_COMPONENTS = GROUPS.reduce((n, g) => n + g.count, 0)

export default controller<Scope>({
  scope: () => ({
    groups: derived<PackGroup[]>(() => GROUPS),
    totalComponents: derived<number>(() => TOTAL_COMPONENTS),
    totalPacks: derived<number>(() => GROUPS.length),
  }),

  mount(_state, host) {
    // For each preview slot in the rendered grid, mount a single bare
    // instance of the registered custom element. The same one-element
    // harness the original catalog used, extended with: (1) a per-tag
    // seed (props + innerHTML) so non-trivial primitives actually show
    // something, and (2) a `scope="preview"` shim on `<sui-theme-picker>`
    // so its mount-time `applyTheme()` doesn't flip the document theme
    // (the picker exposes `scope` for exactly this case — see
    // packages/components/sui/primitives/src/sui-theme-picker for the
    // controller).
    //
    // Seed innerHTML is rendered via the runtime's `bindHtml` (the
    // morph-backed binding — NOT a raw `el.innerHTML = …` which the
    // sui-controller-purity rule forbids). The dispose returned by
    // bindHtml is collected in `binds` and called on unmount.
    const root = (host.shadowRoot ?? host) as ParentNode
    const mounted: Element[] = []
    const binds: Array<() => void> = []
    const raf = requestAnimationFrame(() => {
      const slots = root.querySelectorAll<HTMLElement>('[data-preview-for]')
      slots.forEach((slot) => {
        const tag = slot.getAttribute('data-preview-for')
        if (!tag || !customElements.get(tag)) {
          const note = document.createElement('span')
          note.className = 'preview-empty'
          note.textContent = 'not in bundle'
          slot.appendChild(note)
          slot.dataset.previewState = 'unavailable'
          mounted.push(note)
          return
        }
        try {
          const el = document.createElement(tag)
          const seed = PREVIEW_SEEDS[tag]
          if (seed?.props) {
            for (const [k, v] of Object.entries(seed.props)) el.setAttribute(k, v)
          }
          // The picker is the one primitive whose mount has document-level
          // side effects (`applyTheme()` writes `data-theme` + `.dark` on
          // documentElement). The picker exposes a `scope` prop that, when
          // non-empty, isolates those writes to its host subtree. Setting
          // `scope="preview"` here keeps the catalog from flipping the
          // document theme on every page load.
          if (tag === 'sui-theme-picker') el.setAttribute('scope', 'preview')
          slot.appendChild(el)
          if (seed?.innerHTML) {
            const seedHtml = seed.innerHTML
            binds.push(bindHtml(el, () => seedHtml))
          }
          slot.dataset.previewState = 'live'
          mounted.push(el)
        } catch {
          const note = document.createElement('span')
          note.className = 'preview-empty'
          note.textContent = 'not in bundle'
          slot.appendChild(note)
          slot.dataset.previewState = 'unavailable'
          mounted.push(note)
        }
      })
    })
    return () => {
      cancelAnimationFrame(raf)
      for (const unbind of binds) unbind()
      for (const el of mounted) el.remove()
    }
  },
})
