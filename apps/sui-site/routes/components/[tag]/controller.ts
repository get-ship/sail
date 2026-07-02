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
import { PREVIEW_SEEDS as DEMO_SEEDS } from '../../../src/sui-catalog/preview-seeds'
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

// Demo seeds are imported from the shared `preview-seeds` module (aliased as
// `DEMO_SEEDS`) — the SAME per-tag props + default-slot innerHTML the
// /components grid mounts, so the detail preview and the gallery card stay in
// lockstep.

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
