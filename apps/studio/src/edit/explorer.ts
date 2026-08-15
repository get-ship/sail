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
  // App / pack hosts are the third layer in studio badges (L1 keyword · L2 pack · L3 app).
  userland: 'L3',
  user: 'L3',
  pack: 'L2',
  unknown: '?',
}

/** Nested node shape for `<sail-tree nodes>` / createTable identity data. */
export type SailTreeNode = {
  id: string
  label: string
  icon?: string
  /**
   * Row meta for sail-tree `node-trailing` (e.g. dirty `●`).
   * Keep labels short — never fold meta into `label`.
   */
  trailing?: string | undefined
  children?: SailTreeNode[] | undefined
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
 * Display / trie path under a group root (disambiguates multi-`page.sail`).
 * `apps/x/routes/foo/page.sail` + group routes → `foo/page.sail`.
 * Group `other`: project-relative path when under `apps/<name>/`, else full path
 * (never bare basename alone when nested — tree nesting needs the segments).
 */
export function fileDisplayName(path: string, group: string): string {
  const p = path.replace(/\\/g, '/')
  if (group !== 'other') {
    const re = new RegExp(`(?:^|/)${group}/(.+)$`)
    const m = p.match(re)
    if (m?.[1]) return m[1]
  }
  // Project-relative under apps/<pkg>/… so other/ nests test/, package.json, etc.
  const underApp = p.match(/^apps\/[^/]+\/(.+)$/)
  if (underApp?.[1]) return underApp[1]
  // Bare project root files (routes/… already handled above for known groups).
  if (!p.includes('/')) return p
  // Fall back to full relative path (not basename) so nested siblings stay distinct.
  return p.replace(/^\.\//, '')
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

/** Intermediate dir while assembling a group-relative path trie. */
type FileSegNode = {
  name: string
  /** Set when this segment is a leaf file (source path = openDocument id). */
  filePath?: string
  children: Map<string, FileSegNode>
}

function insertFileSeg(root: FileSegNode, relPath: string, filePath: string): void {
  const segs = relPath.replace(/\\/g, '/').split('/').filter(Boolean)
  if (!segs.length) return
  let cur = root
  for (let i = 0; i < segs.length; i++) {
    const seg = segs[i]!
    let child = cur.children.get(seg)
    if (!child) {
      child = { name: seg, children: new Map() }
      cur.children.set(seg, child)
    }
    if (i === segs.length - 1) child.filePath = filePath
    else cur = child
  }
}

function fileIconForName(name: string): string {
  if (name.endsWith('.sail')) return 'file-code-2'
  if (/\.(ts|tsx|js|jsx|mjs|cjs)$/.test(name)) return 'file-code'
  return 'file'
}

function fileSegToSailTree(
  node: FileSegNode,
  groupId: string,
  rel: string,
  dirty?: Set<string>,
): SailTreeNode {
  const kids = [...node.children.values()]
    .sort((a, b) => {
      // Directories before files, then alpha — denser IDE-style tree.
      const aIsDir = a.children.size > 0 || !a.filePath
      const bIsDir = b.children.size > 0 || !b.filePath
      if (aIsDir !== bIsDir) return aIsDir ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    .map((c) => {
      const childRel = rel ? `${rel}/${c.name}` : c.name
      return fileSegToSailTree(c, groupId, childRel, dirty)
    })

  if (node.filePath) {
    return {
      id: node.filePath,
      // Basename under nested dirs; path-relative insert already disambiguates
      // multi-`page.sail` via parent dir rows (never flat sibling page.sail list).
      label: node.name,
      icon: fileIconForName(node.name),
      trailing: dirty?.has(node.filePath) ? '●' : undefined,
      children: kids.length ? kids : undefined,
    }
  }
  return {
    id: `dir:${groupId}/${rel}`,
    label: node.name,
    icon: 'folder',
    children: kids.length ? kids : undefined,
  }
}

/**
 * Full project file tree for the Files tab — group roots → nested dirs → files.
 * Labels stay short (basename / dir name only). Dirty goes in `trailing` for
 * sail-tree `node-trailing` (not label cram). Filter prunes via `filterFiles`.
 */
export function fileSailTreeNodes(
  files: FileEntry[],
  filter: string,
  activePath: string | null,
  opts?: {
    dirtyPaths?: Set<string>
  },
): SailTreeNode[] {
  const groups = fileTreeGroups(files, filter, activePath)
  const dirty = opts?.dirtyPaths
  return groups.map((g) => {
    const root: FileSegNode = { name: g.id, children: new Map() }
    for (const f of g.files) insertFileSeg(root, f.name, f.path)
    const children = [...root.children.values()]
      .sort((a, b) => {
        const aIsDir = a.children.size > 0 || !a.filePath
        const bIsDir = b.children.size > 0 || !b.filePath
        if (aIsDir !== bIsDir) return aIsDir ? -1 : 1
        return a.name.localeCompare(b.name)
      })
      .map((c) => fileSegToSailTree(c, g.id, c.name, dirty))
    return {
      id: `group:${g.id}`,
      label: g.label,
      icon: 'folder',
      children,
    }
  })
}

/**
 * Nested element nodes for the Components tab sail-tree.
 * Labels stay short (tag / control-flow / name hint). Layer badge lives in
 * `trailing` (not label cram — same 557 B lesson as Files dirty dots).
 * Active-doc tree only; project-wide needs a multi-file index (not inventable).
 */
export function elementSailTreeNodes(
  nodes: TreeNode[],
  roots: string[],
): SailTreeNode[] {
  const byId = new Map(nodes.map((n) => [n.id, n]))
  const walk = (id: string): SailTreeNode | null => {
    const n = byId.get(id)
    if (!n) return null
    const cf = n.control_flow
    const controlFlow = cf
      ? `<${cf.kind}>${cf.expr ? ` ${cf.expr}` : ''}`
      : undefined
    const attrs = n.attrs ?? {}
    const nameHint =
      attrs['name'] || attrs['data-sail-host'] || attrs['id'] || ''
    // Dense label: control-flow expr, else <tag> + optional host/name/id.
    const label = controlFlow
      ? controlFlow
      : nameHint
        ? `<${n.tag}> ${nameHint}`
        : `<${n.tag}>`
    const badge = layerBadge(n.layer)
    const kids = (n.children ?? [])
      .map((c) => walk(c))
      .filter((c): c is SailTreeNode => c != null)
    return {
      id: n.id,
      label,
      // Layer badge as trail meta (L1 keyword · L2 pack · L3 app).
      trailing: badge || undefined,
      // Density cue by layer — leaf-friendly lucide names.
      icon:
        badge === 'L2' ? 'star' : badge === 'L3' ? 'inbox' : 'circle',
      children: kids.length ? kids : undefined,
    }
  }
  const out: SailTreeNode[] = []
  const seen = new Set<string>()
  for (const r of roots) {
    const n = walk(r)
    if (n) {
      out.push(n)
      const mark = (node: SailTreeNode) => {
        seen.add(node.id)
        for (const c of node.children ?? []) mark(c)
      }
      mark(n)
    }
  }
  for (const n of nodes) {
    if (seen.has(n.id)) continue
    const t = walk(n.id)
    if (t) out.push(t)
  }
  return out
}

/**
 * Map a routes `page.sail` path to its URL path shape (including dynamics).
 * `routes/page.sail` → `/`; `routes/edit/page.sail` → `/edit`;
 * `routes/components/[tag]/page.sail` → `/components/[tag]`.
 * Layouts and non-page files → null.
 */
export function pathToRoutePath(path: string | null | undefined): string | null {
  if (!path) return null
  const p = path.replace(/\\/g, '/')
  if (/(?:^|\/)routes\/page\.sail$/.test(p)) return '/'
  const m = p.match(/(?:^|\/)routes\/(.+)\/page\.sail$/)
  if (!m?.[1]) return null
  const segs = m[1].split('/').filter(Boolean)
  if (!segs.length) return '/'
  return `/${segs.join('/')}`
}

/**
 * Map a .sail workspace path to the served app route for live iframe preview.
 * Same as pathToRoutePath, but dynamic segments (`[tag]`, `[...slug]`) → null
 * (no concrete live URL). Non-route files return null.
 */
export function pathToLiveRoute(path: string | null | undefined): string | null {
  const route = pathToRoutePath(path)
  if (!route) return null
  const segs = route === '/' ? [] : route.slice(1).split('/')
  // Unresolved dynamic params are not real served URLs.
  if (segs.some((s) => /^\[[^\]]+\]$/.test(s) || /^\[\.\.\.[^\]]+\]$/.test(s))) {
    return null
  }
  return route
}

/** Trie node while assembling routeSailTreeNodes. */
type RouteSegNode = {
  routePath: string
  filePath?: string
  children: Map<string, RouteSegNode>
}

function routeSegToSailTree(
  node: RouteSegNode,
  dirty?: Set<string>,
): SailTreeNode {
  const kids = [...node.children.values()]
    .sort((a, b) => a.routePath.localeCompare(b.routePath))
    .map((c) => routeSegToSailTree(c, dirty))
  const id = node.filePath ?? `route:${node.routePath}`
  const base = node.routePath
  const label =
    node.filePath && dirty?.has(node.filePath) ? `● ${base}` : base
  return {
    id,
    label,
    icon: node.filePath ? 'file-code-2' : 'folder',
    children: kids.length ? kids : undefined,
  }
}

/**
 * Route hierarchy for the Routes tab — pages as routes (path labels), not file paths.
 * Top-level siblings (`/`, `/edit`, `/sail`…); multi-segment pages nest under prefixes.
 * Leaf/page ids are source paths (selection opens the file). Intermediate segments
 * without a page use `route:<path>` ids (expand only).
 */
export function routeSailTreeNodes(
  files: FileEntry[],
  filter: string,
  opts?: {
    dirtyPaths?: Set<string>
  },
): SailTreeNode[] {
  const dirty = opts?.dirtyPaths
  const q = filter.trim().toLowerCase()
  const root: RouteSegNode = {
    routePath: '/',
    children: new Map(),
  }

  // Insert all route pages; filter prunes the built tree (keeps ancestors of hits).
  for (const f of files) {
    if (f.group !== 'routes') continue
    const route = pathToRoutePath(f.path)
    if (!route) continue
    if (route === '/') {
      root.filePath = f.path
      continue
    }
    const segs = route.slice(1).split('/').filter(Boolean)
    let cur = root
    let acc = ''
    for (const seg of segs) {
      acc += `/${seg}`
      let child = cur.children.get(seg)
      if (!child) {
        child = { routePath: acc, children: new Map() }
        cur.children.set(seg, child)
      }
      cur = child
    }
    cur.filePath = f.path
  }

  const out: SailTreeNode[] = []
  if (root.filePath) {
    out.push({
      id: root.filePath,
      label: dirty?.has(root.filePath) ? '● /' : '/',
      icon: 'file-code-2',
    })
  }
  for (const child of [...root.children.values()].sort((a, b) =>
    a.routePath.localeCompare(b.routePath),
  )) {
    out.push(routeSegToSailTree(child, dirty))
  }

  if (!q) return out

  // Prune: keep a node if its label/id matches or any descendant does.
  const prune = (nodes: SailTreeNode[]): SailTreeNode[] => {
    const kept: SailTreeNode[] = []
    for (const n of nodes) {
      const kids = prune(n.children ?? [])
      const self =
        n.label.toLowerCase().includes(q) || n.id.toLowerCase().includes(q)
      if (self) {
        // Self match: keep full original subtree (re-resolve from n).
        kept.push(n)
      } else if (kids.length) {
        kept.push({ ...n, children: kids })
      }
    }
    return kept
  }
  return prune(out)
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
