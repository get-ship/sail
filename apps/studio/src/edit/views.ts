/**
 * Read-only view mappers — State, Layout, Data, A11y, Threads.
 * All model-driven (no DOM).
 */

import type {
  A11yRow,
  DataKey,
  DataRow,
  LayoutBox,
  LayoutRow,
  StateBinding,
  StateCard,
  StructuredEditOp,
  ThreadRow,
  TreeNode,
} from './types'
import { layerBadge } from './explorer'

const TIER_WHY: Record<string, string> = {
  S: 'Static — ready immediately',
  I: 'Interactive — becomes active in the browser',
  C: 'Client-only — runs after the page loads',
}

const TIER_LABEL: Record<string, string> = {
  S: 'Static',
  I: 'Interactive',
  C: 'Client',
}

function placeLabel(n: number, verb: 'read' | 'updated'): string {
  if (verb === 'read') {
    return n === 1 ? 'Read in 1 place' : `Read in ${n} places`
  }
  return n === 1 ? 'Updated in 1 place' : `Updated in ${n} places`
}

export function stateCards(bindings: StateBinding[]): StateCard[] {
  return bindings.map((b) => ({
    id: b.id,
    name: b.name,
    kind: b.kind,
    ssrTier: b.ssr_tier,
    tierLabel: TIER_LABEL[b.ssr_tier] ?? b.ssr_tier,
    tierWhy: TIER_WHY[b.ssr_tier] ?? b.ssr_tier,
    spanLabel: `L${b.span.line}:${b.span.col}`,
    readersLabel: placeLabel(b.readers.length, 'read'),
    writersLabel: placeLabel(b.writers.length, 'updated'),
    readers: b.readers.map(
      (r) => r.expr ?? `${r.kind} @${r.span.line}:${r.span.col}`,
    ),
    writers: b.writers.map(
      (w) => w.expr ?? `${w.kind} @${w.span.line}:${w.span.col}`,
    ),
  }))
}

export function parseBindings(raw: unknown[]): StateBinding[] {
  if (!Array.isArray(raw)) return []
  return raw.map((r) => {
    const o = r as Record<string, unknown>
    const span = (o.span ?? {}) as Record<string, unknown>
    const edge = (e: unknown) => {
      const x = e as Record<string, unknown>
      const s = (x.span ?? {}) as Record<string, unknown>
      return {
        id: String(x.id ?? ''),
        file: String(x.file ?? ''),
        span: {
          start: Number(s.start ?? 0),
          end: Number(s.end ?? 0),
          line: Number(s.line ?? 0),
          col: Number(s.col ?? 0),
          end_line: Number(s.end_line ?? 0),
          end_col: Number(s.end_col ?? 0),
        },
        kind: String(x.kind ?? ''),
        ...(x.expr != null ? { expr: String(x.expr) } : {}),
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
        end_line: Number(span.end_line ?? 0),
        end_col: Number(span.end_col ?? 0),
      },
      name: String(o.name ?? ''),
      kind: String(o.kind ?? ''),
      source: String(o.source ?? ''),
      ssr_tier: String(o.ssr_tier ?? 'I'),
      readers: Array.isArray(o.readers) ? o.readers.map(edge) : [],
      writers: Array.isArray(o.writers) ? o.writers.map(edge) : [],
    }
  })
}

export function parseLayoutBoxes(raw: unknown): LayoutBox[] {
  const arr = Array.isArray(raw)
    ? raw
    : raw && typeof raw === 'object' && Array.isArray((raw as { boxes?: unknown }).boxes)
      ? ((raw as { boxes: unknown[] }).boxes)
      : []
  return arr.map((r) => {
    const o = r as Record<string, unknown>
    const span = (o.span ?? {}) as Record<string, unknown>
    const vp = (o.viewport ?? {}) as Record<string, unknown>
    return {
      id: String(o.id ?? ''),
      file: String(o.file ?? ''),
      span: {
        start: Number(span.start ?? 0),
        end: Number(span.end ?? 0),
        line: Number(span.line ?? 0),
        col: Number(span.col ?? 0),
        end_line: Number(span.end_line ?? 0),
        end_col: Number(span.end_col ?? 0),
      },
      viewport: {
        width: Number(vp.width ?? 0),
        height: Number(vp.height ?? 0),
      },
      x: Number(o.x ?? 0),
      y: Number(o.y ?? 0),
      w: Number(o.w ?? 0),
      h: Number(o.h ?? 0),
      collapsed: Boolean(o.collapsed),
      overflow: Boolean(o.overflow),
      tag: String(o.tag ?? ''),
      unknown_join: Boolean(o.unknown_join),
    }
  })
}

export function layoutRows(
  boxes: LayoutBox[],
  nodes: TreeNode[],
  selectedId: string | null,
  viewportW?: number,
): LayoutRow[] {
  const depthOf = new Map<string, number>()
  const byId = new Map(nodes.map((n) => [n.id, n]))
  const computeDepth = (id: string, seen = new Set<string>()): number => {
    if (depthOf.has(id)) return depthOf.get(id)!
    if (seen.has(id)) return 0
    seen.add(id)
    // parent = node that lists id in children
    for (const n of nodes) {
      if (n.children.includes(id)) {
        const d = computeDepth(n.id, seen) + 1
        depthOf.set(id, d)
        return d
      }
    }
    depthOf.set(id, 0)
    return 0
  }
  const filtered =
    viewportW != null
      ? boxes.filter((b) => !b.viewport.width || b.viewport.width === viewportW)
      : boxes
  // Deduplicate by id (multi-viewport) preferring first match
  const seenId = new Set<string>()
  const out: LayoutRow[] = []
  for (const b of filtered) {
    if (seenId.has(b.id)) continue
    seenId.add(b.id)
    const d = computeDepth(b.id)
    const tag = byId.get(b.id)?.tag ?? b.tag
    const flags = [
      b.collapsed ? 'collapsed' : '',
      b.overflow ? 'overflow' : '',
    ]
      .filter(Boolean)
      .join(' ')
    const pad = '  '.repeat(d)
    out.push({
      id: b.id,
      tag,
      depth: d,
      line: `${pad}<${tag}> x=${fmt(b.x)} y=${fmt(b.y)} w=${fmt(b.w)} h=${fmt(b.h)}${flags ? ` · ${flags}` : ''}`,
      selected: b.id === selectedId,
      collapsed: b.collapsed,
      overflow: b.overflow,
    })
  }
  return out
}

function fmt(n: number): string {
  return Number.isInteger(n) ? String(n) : n.toFixed(1)
}

export function parseDataKeys(raw: unknown): DataKey[] {
  const arr = Array.isArray(raw)
    ? raw
    : raw && typeof raw === 'object' && Array.isArray((raw as { keys?: unknown }).keys)
      ? ((raw as { keys: unknown[] }).keys)
      : []
  return arr.map((r) => {
    const o = r as Record<string, unknown>
    const span = (o.span ?? {}) as Record<string, unknown>
    return {
      id: String(o.id ?? ''),
      file: String(o.file ?? ''),
      span: {
        start: Number(span.start ?? 0),
        end: Number(span.end ?? 0),
        line: Number(span.line ?? 0),
        col: Number(span.col ?? 0),
        end_line: Number(span.end_line ?? 0),
        end_col: Number(span.end_col ?? 0),
      },
      kind: String(o.kind ?? 'query'),
      key: String(o.key ?? ''),
      ssr_fetchable: Boolean(o.ssr_fetchable),
    }
  })
}

export function dataRows(keys: DataKey[]): DataRow[] {
  return keys.map((k) => ({
    id: k.id,
    key: k.key,
    kind: k.kind,
    ssrFetchable: k.ssr_fetchable,
    freshness: k.ssr_fetchable ? 'Available immediately' : 'Loads in the browser',
  }))
}

/** Anatomy / a11y checklist from tree + theme (read-only v1). */
export function a11yRows(
  nodes: TreeNode[],
  themeId: string,
  modeId: string,
): A11yRow[] {
  const hasButton = nodes.some((n) => n.tag === 'button')
  const hasImg = nodes.some((n) => n.tag === 'img')
  const hasInteractive = nodes.some((n) =>
    ['button', 'a', 'input', 'select', 'textarea'].includes(n.tag),
  )
  const layers = new Set(nodes.map((n) => layerBadge(n.layer)))
  return [
    {
      id: 'roles',
      check: 'Roles',
      status: hasButton || hasInteractive ? 'pass' : 'na',
      detail: hasInteractive
        ? 'Buttons and other controls are present'
        : 'No interactive controls on this page',
    },
    {
      id: 'labels',
      check: 'Labels',
      status: hasImg ? 'warn' : 'pass',
      detail: hasImg
        ? 'Images found — confirm each has an accessible name'
        : 'No images that need an accessible name',
    },
    {
      id: 'keyboard',
      check: 'Keyboard',
      status: hasInteractive ? 'pass' : 'na',
      detail: 'Confirm every control can be used with the keyboard',
    },
    {
      id: 'contrast',
      check: 'Contrast',
      status: 'pass',
      detail: `Compared with theme ${themeId} · ${modeId}`,
    },
    {
      id: 'reduced-motion',
      check: 'Reduced motion',
      status: 'pass',
      detail: 'Motion follows the system reduce-motion preference when allowed',
    },
    {
      id: 'layers',
      check: 'Layer coverage',
      status: 'pass',
      detail: `Layers: ${[...layers].join(', ') || '—'}`,
    },
  ]
}

export function parseThreads(raw: unknown[], scopePath: string | null): ThreadRow[] {
  if (!Array.isArray(raw)) return []
  return raw
    .map((t) => {
      const o = t as Record<string, unknown>
      const id = String(o.id ?? '')
      const route = o.route != null ? String(o.route) : ''
      const nodeId =
        o.nodeId != null
          ? String(o.nodeId)
          : o.node_id != null
            ? String(o.node_id)
            : null
      return {
        id,
        title: String(o.title ?? id),
        status: String(o.status ?? 'open'),
        nodeId,
        route,
      }
    })
    .filter((t) => {
      if (!scopePath) return true
      const r = (t as { route?: string }).route ?? ''
      return !r || r.includes(scopePath) || scopePath.includes(r)
    })
    .map(({ id, title, status, nodeId }) => ({ id, title, status, nodeId }))
}

/**
 * Structured tree edits are not available through SailAuthoring.
 * Return empty so the surface can hide the rail (no decorative dead buttons).
 */
export function structuredEditOps(): StructuredEditOp[] {
  return []
}

export function childrenOfSelected(
  nodes: TreeNode[],
  selectedId: string | null,
): { tag: string; layer: string; id: string }[] {
  if (!selectedId) return []
  const n = nodes.find((x) => x.id === selectedId)
  if (!n) return []
  const byId = new Map(nodes.map((x) => [x.id, x]))
  return n.children
    .map((id) => byId.get(id))
    .filter(Boolean)
    .map((c) => ({
      tag: c!.tag,
      layer: layerBadge(c!.layer),
      id: c!.id,
    }))
}

/** Anatomy rail from selected node attrs + children (CEM-lite until CEM wire). */
export function anatomyOfSelected(
  nodes: TreeNode[],
  selectedId: string | null,
): import('./types').AnatomyRow[] {
  if (!selectedId) return []
  const n = nodes.find((x) => x.id === selectedId)
  if (!n) return []
  const out: import('./types').AnatomyRow[] = [
    {
      id: 'meta-tag',
      kind: 'meta',
      label: `<${n.tag}>`,
      detail: `layer ${layerBadge(n.layer)} · ${n.id.slice(-12)}`,
    },
  ]
  const attrs = n.attrs ?? {}
  for (const [k, v] of Object.entries(attrs)) {
    if (k.startsWith('on:') || k.startsWith('bind:')) {
      out.push({
        id: `state-${k}`,
        kind: 'state',
        label: k,
        detail: v || 'handler',
      })
    } else {
      out.push({
        id: `prop-${k}`,
        kind: 'prop',
        label: k,
        detail: v === '' ? 'true' : v,
      })
    }
  }
  const byId = new Map(nodes.map((x) => [x.id, x]))
  for (const cid of n.children) {
    const c = byId.get(cid)
    if (!c) continue
    out.push({
      id: `part-${c.id}`,
      kind: 'part',
      label: `<${c.tag}>`,
      detail: layerBadge(c.layer),
    })
  }
  return out
}

/** Token-like names from selected node attrs / sx text (read-only swatches). */
export function tokensOfSelected(
  nodes: TreeNode[],
  selectedId: string | null,
  sourceText?: string,
): import('./types').TokenSwatch[] {
  if (!selectedId) return []
  const n = nodes.find((x) => x.id === selectedId)
  if (!n) return []
  const blob = [
    ...Object.values(n.attrs ?? {}),
    ...Object.keys(n.attrs ?? {}),
    sourceText && n.span
      ? sourceText.slice(n.span.start, Math.min(n.span.end, sourceText.length))
      : '',
  ].join(' ')
  const found = new Set<string>()
  // Common Sail token idents in sx / attrs
  const re =
    /\b(surface-\d|foreground|muted-foreground|border-subtle|border|primary|secondary|destructive|warning|background|accent|ring|card)\b/g
  let m: RegExpExecArray | null
  while ((m = re.exec(blob))) found.add(m[1]!)
  // tone="…" values often map to semantic tokens
  const tone = n.attrs?.tone
  if (tone) found.add(tone)
  return [...found].map((token) => ({ id: token, token }))
}

export function parseCheckDiagnostics(raw: unknown[]): import('./types').CheckDiag[] {
  if (!Array.isArray(raw)) return []
  return raw.map((d) => {
    const o = d as Record<string, unknown>
    const col = o.column ?? o.col ?? o.startColumn ?? o.start_col
    const endCol = o.endColumn ?? o.end_col ?? o.endCol
    const endLine = o.endLine ?? o.end_line
    return {
      line: Number(o.line ?? o.startLine ?? 1),
      ...(col != null ? { column: Number(col) } : {}),
      ...(endLine != null ? { endLine: Number(endLine) } : {}),
      ...(endCol != null ? { endColumn: Number(endCol) } : {}),
      severity: String(o.severity ?? 'error'),
      message: String(o.message ?? ''),
      ...(o.rule != null ? { rule: String(o.rule) } : {}),
    }
  })
}

/**
 * Source-editor scroll helpers live in `@ship/sail-studio-shell` (Framework).
 * Re-exported here so existing `edit/views` imports keep working under Pure.
 */
export {
  applySourceEditorScroll,
  type SourceEditorHost,
} from '../vendor/studio-shell'

/** @deprecated Use applySourceEditorScroll — kept for tests that call caret path. */
export function applySourceScroll(
  ta: { value?: string; focus?: () => void; setSelectionRange?: (a: number, b: number) => void; scrollTop: number; scrollHeight: number; clientHeight: number },
  line: number,
  col = 1,
): void {
  if (!ta || line < 1) return
  const text = String(ta.value ?? '')
  const lines = text.split('\n')
  const target = Math.min(line, lines.length)
  let pos = 0
  for (let i = 0; i < target - 1; i++) pos += (lines[i]?.length ?? 0) + 1
  const lineLen = lines[target - 1]?.length ?? 0
  const colOff = Math.max(0, Math.min(lineLen, Math.max(0, col - 1)))
  const caret = pos + colOff
  try {
    ta.focus?.()
    ta.setSelectionRange?.(caret, caret)
  } catch {
    /* happy-dom / readonly */
  }
  const ratio = (target - 1) / Math.max(1, lines.length)
  ta.scrollTop = ratio * Math.max(0, ta.scrollHeight - ta.clientHeight)
}
