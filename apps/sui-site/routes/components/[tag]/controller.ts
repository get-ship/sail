// /components/[tag] controller — the per-component detail view.
//
// Reads the route param (the [tag] dynamic segment), looks up the component
// in the build-time `COMPONENTS_CATALOG`, and exposes a single `entry()`
// signal that drives the template. The live-preview mount lives in the
// controller: a one-element `document.createElement(tag)` placed into the
// `stageEl` ref the template binds (`bind:this`). The seed map supplies
// default attribute props + slotted innerHTML for primitives that need
// pre-populated content to render anything visible. The mount uses
// `bindHtml` (the runtime's morph-backed binding), NOT a raw `el.innerHTML`
// write, which the controller-purity lint forbids.
//
// Framework-pure: no ship-platform coupling. The catalog CEM data is bundled
// at build time via deck's `generate_components_gen` pass.

import { controller, bindHtml, ref, type ViewScope } from '@ship/sui-runtime'
import { COMPONENTS_CATALOG } from '../../../components.gen'
import type { Scope } from './page.sui'

// Shape the components.gen.ts exposes for each declaration — narrow + read-
// only here. The table iterates the raw arrays; no derivation needed.
interface PropMeta { name: string; type: string; default?: unknown }
interface NamedMeta { name: string; description?: string }
export interface Entry {
  tag: string
  pkg: string
  description: string
  props: PropMeta[]
  slots: NamedMeta[]
  events: NamedMeta[]
}

// Per-tag lookup table built once at module load — pure data transform over
// the bundled catalog, no per-render cost.
const BY_TAG: Map<string, Entry> = (() => {
  const modules = (COMPONENTS_CATALOG as { modules?: Array<{ package?: string; declarations?: Array<{
    kind?: string
    customElement?: boolean
    tagName?: string
    description?: string
    attributes?: Array<{ name?: string; type?: { text?: string }; default?: unknown }>
    slots?: Array<{ name?: string; description?: string }>
    events?: Array<{ name?: string; description?: string }>
  }> }> }).modules ?? []
  const map = new Map<string, Entry>()
  for (const m of modules) {
    const pkg = m.package ?? 'workspace'
    for (const d of m.declarations ?? []) {
      if (d.kind !== 'class' || !d.customElement || !d.tagName) continue
      const entry: Entry = {
        tag: d.tagName,
        pkg,
        description: (d.description ?? '').split('\n')[0]?.trim() ?? '',
        props: (d.attributes ?? []).map((a) => ({
          name: a.name ?? '',
          type: a.type?.text ?? 'string',
          ...(a.default === undefined ? {} : { default: a.default }),
        })),
        slots: (d.slots ?? []).map((s) => ({ name: s.name ?? '(default)', description: s.description })),
        events: (d.events ?? []).map((e) => ({ name: e.name ?? '', description: e.description })),
      }
      map.set(entry.tag, entry)
    }
  }
  return map
})()

// Demo seed content — see the explanation in the file header.
export const DEMO_SEEDS: Record<string, { props?: Record<string, string>; innerHTML?: string }> = {
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

export default controller<Scope>({
  scope: (ctx: ViewScope<Scope>) => {
    const tag = (): string => ctx.route.params.tag() ?? ''
    const entry = (): Entry | null => BY_TAG.get(tag()) ?? null
    const seed = (): { props?: Record<string, string>; innerHTML?: string } | null =>
      DEMO_SEEDS[tag()] ?? null
    const stageEl = ref<HTMLElement>(null)
    return { tag, entry, seed, stageEl }
  },

  mount(state, host) {
    // Mount a single bare instance of the registered custom element into
    // the `stageEl` ref (the template's `bind:this`). The same one-
    // element harness the catalog grid uses per cell — except here
    // it's one focused preview instead of 74 in a grid. Attribute
    // props come from the seed map; default slot content uses the
    // runtime's `bindHtml` (the morph-backed binding, NOT a raw
    // `el.innerHTML = …` which the controller-purity rule forbids).
    //
    // Two startup races to bridge: (1) the cell ref is set by the
    // template's `bind:this` on first mount, (2) the `customElements`
    // registry is filled in by the page chunk's static imports. Either
    // may land first on a cold load, so we poll on a 100ms cadence
    // (60 attempts = 6s) before falling back to the "not in bundle"
    // empty state.
    let mounted: Element | null = null
    let stopBind: (() => void) | null = null
    let pollHandle: ReturnType<typeof setTimeout> | null = null
    let attempts = 0
    const tryMount = (): void => {
      const cell = state.$signals.stageEl()
      const e = state.entry()
      const s = state.seed()

      if (cell && e && e.tag && customElements.get(e.tag)) {
        try {
          const el = document.createElement(e.tag)
          if (s && s.props) {
            for (const [k, v] of Object.entries(s.props)) el.setAttribute(k, v)
          }
          cell.appendChild(el)
          if (s && s.innerHTML) {
            const seedHtml = s.innerHTML
            stopBind = bindHtml(el, () => seedHtml)
          }
          mounted = el
          return
        } catch {
          return
        }
      }
      if (attempts++ < 60) {
        pollHandle = setTimeout(tryMount, 100)
        return
      }
      if (cell) {
        const note = document.createElement('span')
        note.className = 'preview-empty'
        note.textContent = 'not in bundle'
        cell.appendChild(note)
      }
    }
    const raf = requestAnimationFrame(tryMount)
    return () => {
      cancelAnimationFrame(raf)
      clearTimeout(pollHandle)
      stopBind?.()
      if (mounted) mounted.remove()
    }
  },
})
