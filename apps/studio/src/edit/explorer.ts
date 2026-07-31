/**
 * Explorer builders — file tree + element tree from SailInspect nodes.
 * Pure: no DOM assessment (T-I1).
 */

import type {
  ElementRow,
  FileEntry,
  FileRow,
  TreeNode,
} from './types'

const LAYER_BADGE: Record<string, string> = {
  'l1-keyword': 'L1',
  'l1-element': 'L1',
  l1: 'L1',
  l2: 'L2',
  'l2-component': 'L2',
  userland: 'user',
  user: 'user',
  pack: 'pack',
  unknown: '?',
}

const GROUP_ORDER = ['routes', 'src', 'packs', 'other'] as const
const GROUP_LABEL: Record<string, string> = {
  routes: 'routes',
  src: 'src',
  packs: 'packs',
  other: 'other',
}

export type FileTreeGroup = {
  id: string
  label: string
  files: FileRow[]
}

export function layerBadge(layer: string): string {
  return LAYER_BADGE[layer] ?? (layer.slice(0, 4) || '?')
}

export function filterFiles(files: FileEntry[], filter: string): FileEntry[] {
  const q = filter.trim().toLowerCase()
  if (!q) return files
  return files.filter(
    (f) => f.name.toLowerCase().includes(q) || f.path.toLowerCase().includes(q),
  )
}

/** Flat file rows for Sail `<for>` — grouped routes / src / packs. */
export function fileRows(
  files: FileEntry[],
  filter: string,
  activePath: string | null,
): FileRow[] {
  return fileTreeGroups(files, filter, activePath).flatMap((g) => g.files)
}

/**
 * Display label under a group root (disambiguates multi-`page.sail`).
 * `apps/x/routes/foo/page.sail` + group routes → `foo/page.sail`.
 */
export function fileDisplayName(path: string, group: string): string {
  const p = path.replace(/\\/g, '/')
  const re = new RegExp(`(?:^|/)${group}/(.+)$`)
  const m = p.match(re)
  if (m?.[1]) return m[1]
  return p.split('/').pop() ?? p
}

/** Stable DOM id for aria-activedescendant tree items. */
export function pathDomId(path: string): string {
  return `file-${path.replace(/[^a-zA-Z0-9_-]+/g, '-')}`
}

/**
 * Pure keyboard focus move for a flat tree path list.
 * Keys: ArrowDown | ArrowUp | Home | End. Other keys return current.
 */
export function nextTreeFocus(
  paths: string[],
  current: string | null,
  key: string,
): string | null {
  if (!paths.length) return null
  const cur = current && paths.includes(current) ? current : paths[0]!
  const idx = Math.max(0, paths.indexOf(cur))
  if (key === 'ArrowDown') return paths[Math.min(paths.length - 1, idx + 1)]!
  if (key === 'ArrowUp') return paths[Math.max(0, idx - 1)]!
  if (key === 'Home') return paths[0]!
  if (key === 'End') return paths[paths.length - 1]!
  return cur
}

/** Directory groups for collapsible explorer tree. */
export function fileTreeGroups(
  files: FileEntry[],
  filter: string,
  activePath: string | null,
): FileTreeGroup[] {
  const filtered = filterFiles(files, filter)
  const out: FileTreeGroup[] = []
  for (const g of GROUP_ORDER) {
    const bucket = filtered.filter((f) => f.group === g)
    if (!bucket.length) continue
    out.push({
      id: g,
      label: GROUP_LABEL[g] ?? g,
      files: bucket.map((f) => ({
        path: f.path,
        name: fileDisplayName(f.path, f.group),
        group: f.group,
        checks: f.checks,
        checksLabel: f.checks > 0 ? String(f.checks) : '',
        hasChecks: f.checks > 0,
        selected: f.path === activePath,
        indent: 1,
      })),
    })
  }
  return out
}

export function groupHeaders(files: FileEntry[], filter: string): string[] {
  const filtered = filterFiles(files, filter)
  const seen = new Set<string>()
  for (const f of filtered) seen.add(f.group)
  return GROUP_ORDER.filter((g) => seen.has(g))
}

/** Walk tree in document order from roots; depth for indent. */
export function elementRows(
  nodes: TreeNode[],
  roots: string[],
  selectedId: string | null,
): ElementRow[] {
  const byId = new Map(nodes.map((n) => [n.id, n]))
  const out: ElementRow[] = []

  const walk = (id: string, depth: number) => {
    const n = byId.get(id)
    if (!n) return
    const cf = n.control_flow
    const controlFlow = cf
      ? `<${cf.kind}>${cf.expr ? ` ${cf.expr}` : ''}`
      : undefined
    const attrs = n.attrs ?? {}
    const nameHint =
      attrs['name'] || attrs['data-sail-host'] || attrs['id'] || ''
    const label = controlFlow
      ? controlFlow
      : nameHint
        ? `<${n.tag}> ${nameHint}`
        : `<${n.tag}>`
    out.push({
      id: n.id,
      tag: n.tag,
      layer: n.layer,
      layerBadge: layerBadge(n.layer),
      depth,
      label,
      selected: n.id === selectedId,
      spanLine: n.span?.line ?? 0,
      controlFlow,
    })
    for (const c of n.children ?? []) walk(c, depth + 1)
  }

  for (const r of roots) walk(r, 0)
  const visited = new Set(out.map((r) => r.id))
  for (const n of nodes) {
    if (!visited.has(n.id)) walk(n.id, 0)
  }
  return out
}

/** Parse SailInspect/Tree JSON into typed nodes. */
export function parseTreeNodes(raw: unknown[]): TreeNode[] {
  if (!Array.isArray(raw)) return []
  return raw.map((r) => {
    const o = r as Record<string, unknown>
    const span = (o.span ?? {}) as Record<string, unknown>
    const cf = o.control_flow as Record<string, unknown> | undefined
    const attrsRaw = o.attrs
    let attrs: Record<string, string> | undefined
    if (attrsRaw && typeof attrsRaw === 'object') {
      attrs = {}
      for (const [k, v] of Object.entries(attrsRaw as Record<string, unknown>)) {
        attrs[k] = String(v ?? '')
      }
    }
    return {
      id: String(o.id ?? ''),
      file: String(o.file ?? ''),
      span: {
        start: Number(span.start ?? 0),
        end: Number(span.end ?? 0),
        line: Number(span.line ?? 0),
        col: Number(span.col ?? 0),
        end_line: Number(span.end_line ?? span.line ?? 0),
        end_col: Number(span.end_col ?? span.col ?? 0),
      },
      tag: String(o.tag ?? ''),
      layer: String(o.layer ?? 'unknown'),
      attrs,
      control_flow: cf
        ? { kind: String(cf.kind ?? ''), expr: cf.expr != null ? String(cf.expr) : undefined }
        : undefined,
      children: Array.isArray(o.children) ? o.children.map(String) : [],
    }
  })
}

export function pathToFileEntry(path: string, checks = 0): FileEntry {
  const name = path.split('/').pop() ?? path
  let group = 'other'
  if (path.includes('/routes/') || path.startsWith('routes/')) group = 'routes'
  else if (path.includes('/src/') || path.startsWith('src/')) group = 'src'
  else if (path.includes('/packs/') || path.startsWith('packs/')) group = 'packs'
  return { path, name, group, checks }
}

/**
 * Map a .sail workspace path to the served app route for live iframe preview.
 * `routes/page.sail` → `/`; `routes/foo/bar/page.sail` → `/foo/bar`.
 * Dynamic segments (`[tag]`, `[...slug]`) → null (no live URL; use audit).
 * Non-route files return null.
 */
export function pathToLiveRoute(path: string | null | undefined): string | null {
  if (!path) return null
  const p = path.replace(/\\/g, '/')
  if (/(?:^|\/)routes\/page\.sail$/.test(p)) return '/'
  const m = p.match(/(?:^|\/)routes\/(.+)\/page\.sail$/)
  if (!m?.[1]) return null
  const segs = m[1].split('/').filter(Boolean)
  if (!segs.length) return '/'
  // Unresolved dynamic params are not real served URLs.
  if (segs.some((s) => /^\[[^\]]+\]$/.test(s) || /^\[\.\.\.[^\]]+\]$/.test(s))) {
    return null
  }
  return `/${segs.join('/')}`
}

/** Discover .sail files under project dirs (Node/fs). Browser: pass seed paths. */
export async function discoverSailFiles(
  projectRoot: string,
  readDir?: (dir: string) => Promise<string[] | null>,
): Promise<FileEntry[]> {
  if (!readDir) return []
  const roots = ['routes', 'src', 'packs']
  const out: FileEntry[] = []
  const walk = async (rel: string, group: string) => {
    const abs = projectRoot ? `${projectRoot.replace(/\/$/, '')}/${rel}` : rel
    const names = await readDir(abs)
    if (!names) return
    for (const name of names) {
      if (name.startsWith('.')) continue
      const child = rel ? `${rel}/${name}` : name
      if (name.endsWith('.sail')) {
        out.push({
          path: projectRoot
            ? `${projectRoot.replace(/\/$/, '')}/${child}`.replace(/^\.\//, '')
            : child,
          name,
          group,
          checks: 0,
        })
      } else if (!name.includes('.')) {
        await walk(child, group)
      }
    }
  }
  for (const g of roots) await walk(g, g)
  return out
}
