/**
 * Map COMPONENTS_CATALOG + project file paths → ComponentRecord[].
 * Pure — no ops, no listeners. Packs only; never dumps L1_CATALOG.
 */
import type {
  CatalogAttribute,
  CatalogDeclaration,
  ComponentProp,
  ComponentRecord,
  ComponentsCatalog,
} from './types'

function attrType(a: CatalogAttribute): string {
  if (!a.type) return 'unknown'
  if (typeof a.type === 'string') return a.type
  return a.type.text || 'unknown'
}

function mapProps(attrs: CatalogAttribute[] | undefined): ComponentProp[] {
  if (!Array.isArray(attrs)) return []
  return attrs
    .filter((a) => a?.name)
    .map((a) => ({
      name: String(a.name),
      type: attrType(a),
      defaultValue: a.default != null ? String(a.default) : '',
      required: !!a.required,
      description: a.description ? String(a.description) : '',
    }))
}

function mapSlots(decl: CatalogDeclaration): string[] {
  const raw = decl.slots
  if (!Array.isArray(raw)) return []
  return raw
    .map((s) => (typeof s === 'string' ? s : s?.name ? String(s.name) : ''))
    .filter(Boolean)
}

function displayFromTag(tag: string, name?: string): string {
  if (name && name.trim()) return name.trim()
  return tag
    .split('-')
    .filter(Boolean)
    .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
    .join(' ')
}

/** Pack records from app components.gen CEM aggregate. */
export function recordsFromCatalog(
  catalog: ComponentsCatalog | null | undefined,
): ComponentRecord[] {
  const modules = catalog?.modules
  if (!Array.isArray(modules)) return []
  const out: ComponentRecord[] = []
  const seen = new Set<string>()
  for (const mod of modules) {
    const pkg = mod.package ? String(mod.package) : ''
    const modPath = mod.path ? String(mod.path) : ''
    const decls = Array.isArray(mod.declarations) ? mod.declarations : []
    for (const decl of decls) {
      const tag = decl.tagName ? String(decl.tagName).trim() : ''
      if (!tag) continue
      if (decl.customElement === false) continue
      // Dedupe by tag — first module wins (stable pack id).
      if (seen.has(tag)) continue
      seen.add(tag)
      const props = mapProps(decl.attributes)
      const slots = mapSlots(decl)
      const pathDisplay = pkg && modPath ? `${pkg}/${modPath}` : modPath || pkg || ''
      out.push({
        id: tag,
        displayName: displayFromTag(tag, decl.name),
        origin: 'pack',
        package: pkg || '—',
        path: pathDisplay,
        layer: 'L2',
        description: decl.description ? String(decl.description) : '',
        props,
        slots,
        issueCount: 0,
        ssrTierSummary: 'unknown',
        hasPreview: false,
        threadCount: 0,
        // Pack CEM paths are not project source — Open in Edit disabled.
        canOpenInEdit: false,
      })
    }
  }
  return out.sort((a, b) => a.id.localeCompare(b.id))
}

/**
 * Project-authored component candidates from ListFiles (or seed paths).
 * Heuristic only — no first-class project component index op (F15 candidate).
 */
export function isProjectComponentPath(path: string): boolean {
  const n = path.replace(/\\/g, '/')
  if (!n.endsWith('.sail')) return false
  if (n.includes('/routes/')) return false
  if (/\/page\.sail$/.test(n)) return false
  if (n.includes('/test/') || n.includes('/fixtures/')) return false
  // App-authored surfaces under src/
  if (n.includes('/src/')) return true
  // Workspace-relative without routes (e.g. packages later)
  if (!n.includes('/node_modules/') && /\/[a-z0-9][\w-]*\.sail$/i.test(n)) {
    // Only if not a bare page
    const base = n.split('/').pop() || ''
    return base !== 'page.sail' && !base.startsWith('_')
  }
  return false
}

/** Derive stable id from path: themes-surface.sail → themes-surface */
export function projectComponentId(path: string): string {
  const n = path.replace(/\\/g, '/')
  const base = n.split('/').pop() || path
  return base.replace(/\.sail$/i, '') || path
}

export function recordsFromProjectPaths(paths: string[]): ComponentRecord[] {
  const out: ComponentRecord[] = []
  const seen = new Set<string>()
  for (const path of paths) {
    if (!isProjectComponentPath(path)) continue
    const id = projectComponentId(path)
    if (!id || seen.has(id)) continue
    seen.add(id)
    out.push({
      id,
      displayName: displayFromTag(id),
      origin: 'project',
      package: '—',
      path,
      layer: 'app',
      description: '',
      props: [],
      slots: [],
      issueCount: 0,
      ssrTierSummary: 'unknown',
      hasPreview: false,
      threadCount: 0,
      canOpenInEdit: true,
    })
  }
  return out.sort((a, b) => a.id.localeCompare(b.id))
}

/** Merge project + pack; project wins on id collision. */
export function mergeComponentRecords(
  project: ComponentRecord[],
  packs: ComponentRecord[],
): ComponentRecord[] {
  const byId = new Map<string, ComponentRecord>()
  for (const p of packs) byId.set(p.id, p)
  for (const p of project) byId.set(p.id, p)
  return [...byId.values()].sort((a, b) => {
    if (a.origin !== b.origin) return a.origin === 'project' ? -1 : 1
    return a.id.localeCompare(b.id)
  })
}

export function filterRecords(
  records: ComponentRecord[],
  query: string,
): ComponentRecord[] {
  const q = query.trim().toLowerCase()
  if (!q) return records
  return records.filter((r) => {
    const hay = [r.id, r.displayName, r.package, r.path, r.description]
      .join(' ')
      .toLowerCase()
    return hay.includes(q)
  })
}

export function recordById(
  records: ComponentRecord[],
  id: string,
): ComponentRecord | null {
  if (!id) return null
  return records.find((r) => r.id === id) ?? null
}

/**
 * Project path authority for catalog loads.
 * - `replace`: gen-owned ListFiles/seed settle (full authority).
 * - `union`: edit explorer tick — may only add paths, never drop ListFiles results.
 */
export function mergeProjectPathAuthority(
  auth: string[],
  incoming: string[],
  mode: 'replace' | 'union',
): { paths: string[]; changed: boolean } {
  if (mode === 'replace') {
    const next = [...incoming]
    const same =
      next.length === auth.length && next.every((p, i) => p === auth[i])
    return { paths: next, changed: !same }
  }
  const set = new Set(auth)
  let changed = false
  for (const p of incoming) {
    if (!p || set.has(p)) continue
    set.add(p)
    changed = true
  }
  if (!changed) return { paths: auth, changed: false }
  return { paths: [...set], changed: true }
}

/** Strip IR label noise → tagName candidate (`<column>` / `<sail-tree> x` → tag). */
export function extractTagName(modelTag: string | undefined, label: string): string {
  if (modelTag?.trim()) return modelTag.trim()
  const m = (label || '').match(/^<?([a-zA-Z][\w-]*)/)
  return m?.[1] ?? ''
}
