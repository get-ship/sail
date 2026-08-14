
import type { Finding, FindingRow, FileFilter, RuleFilter } from './types'
import { APPLY_FIX_READY } from './types'

const LAYOUT_RULE = 'sail-layout'

/** Extract first line + optional "suggest:" trailer from compiler diagnostic text. */
export function splitDiagnosticMessage(raw: string): {
  message: string
  why: string
  suggest: string | null
} {
  const why = raw
  const suggestMatch = raw.match(/\n\s*suggest:\s*([\s\S]+)$/i)
  let body = raw
  let suggest: string | null = null
  if (suggestMatch) {
    suggest = suggestMatch[1]!.trim()
    body = raw.slice(0, suggestMatch.index).trimEnd()
  }
  const message = body.split('\n')[0]?.trim() || body
  return { message, why, suggest }
}

export function fileBase(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || path
}

export function formatFileSpan(path: string, line: number): string {
  return `${fileBase(path)}:${line}`
}

/**
 * True only when LayoutCheck (or equivalent) reports ≥2 viewport passes
 * that actually ran (no per-viewport error).
 */
export function bothViewportsRan(layoutCheck: {
  viewports?: unknown[]
} | null | undefined): boolean {
  const vps = layoutCheck?.viewports
  if (!Array.isArray(vps) || vps.length < 2) return false
  return vps.every((v) => {
    if (!v || typeof v !== 'object') return false
    const o = v as Record<string, unknown>
    return o.error == null || o.error === ''
  })
}

export function parseRawDiagnostic(
  raw: unknown,
  opts: {
    defaultPath?: string | undefined
    bothViewports?: boolean | undefined
    nodeId?: string | null | undefined
  } = {},
): Finding | null {
  if (!raw || typeof raw !== 'object') return null
  const o = raw as Record<string, unknown>
  const severity = String(o.severity ?? 'error')
  const rule = String(o.rule ?? 'unknown')
  const fullMsg = String(o.message ?? '')
  const { message, why, suggest } = splitDiagnosticMessage(fullMsg)
  const path = String(o.path ?? o.file ?? opts.defaultPath ?? '')
  const line = Number(o.line ?? o.startLine ?? 1) || 1
  const col = Number(o.col ?? o.column ?? o.startCol ?? 1) || 1
  const endLine = Number(o.end_line ?? o.endLine ?? line) || line
  const endCol = Number(o.end_col ?? o.endCol ?? col) || col
  const isLayout = rule === LAYOUT_RULE || rule.startsWith('sail-layout')
  const nodeId =
    opts.nodeId ??
    (o.node_id != null
      ? String(o.node_id)
      : o.nodeId != null
        ? String(o.nodeId)
        : null)
  const hasFix = suggest != null && suggest.length > 0
  const id =
    o.id != null
      ? String(o.id)
      : `${path}|${rule}|${line}|${col}|${message.slice(0, 48)}`

  return {
    id,
    severity,
    rule,
    message,
    why,
    suggest,
    hasFix,
    path,
    line,
    col,
    endLine,
    endCol,
    fileSpan: formatFileSpan(path, line),
    bothViewports: isLayout ? !!opts.bothViewports : false,
    isLayout,
    nodeId,
  }
}

/** Parse SailAuthoring/Check diagnostics array. */
export function parseCheckFindings(
  diagnostics: unknown[],
  opts: {
    defaultPath?: string | undefined
    bothViewports?: boolean | undefined
    /** Optional node ids keyed by "rule|line|message-prefix" or Model diags. */
    nodeByKey?: Map<string, string> | undefined
  } = {},
): Finding[] {
  if (!Array.isArray(diagnostics)) return []
  const out: Finding[] = []
  for (const d of diagnostics) {
    const o = d as Record<string, unknown>
    const key = `${o.rule ?? ''}|${o.line ?? ''}|${String(o.message ?? '').split('\n')[0]}`
    const nodeId = opts.nodeByKey?.get(key) ?? null
    const f = parseRawDiagnostic(d, {
      defaultPath: opts.defaultPath,
      bothViewports: opts.bothViewports,
      nodeId,
    })
    if (f) out.push(f)
  }
  return out
}

/** Build node lookup from SailInspect/Model diagnostics. */
export function nodeMapFromModelDiagnostics(
  diagnostics: unknown[],
): Map<string, string> {
  const m = new Map<string, string>()
  if (!Array.isArray(diagnostics)) return m
  for (const d of diagnostics) {
    if (!d || typeof d !== 'object') continue
    const o = d as Record<string, unknown>
    const nodeId = o.node_id ?? o.nodeId
    if (nodeId == null) continue
    const msg = String(o.message ?? '').split('\n')[0]
    const key = `${o.rule ?? ''}|${o.span && typeof o.span === 'object' ? (o.span as { line?: number }).line : o.line}|${msg}`
    m.set(key, String(nodeId))
    // also line-only fallback
    const line =
      o.span && typeof o.span === 'object'
        ? (o.span as { line?: number }).line
        : o.line
    if (line != null) {
      m.set(`${o.rule ?? ''}|${line}|`, String(nodeId))
    }
  }
  return m
}

export function enrichNodeIds(
  findings: Finding[],
  nodeByKey: Map<string, string>,
): Finding[] {
  return findings.map((f) => {
    if (f.nodeId) return f
    const k1 = `${f.rule}|${f.line}|${f.message}`
    const k2 = `${f.rule}|${f.line}|`
    const nodeId = nodeByKey.get(k1) ?? nodeByKey.get(k2) ?? null
    return nodeId ? { ...f, nodeId } : f
  })
}

export function countSeverities(findings: Finding[]): {
  errorCount: number
  warningCount: number
} {
  let errorCount = 0
  let warningCount = 0
  for (const f of findings) {
    if (f.severity === 'error') errorCount++
    else if (f.severity === 'warning') warningCount++
  }
  return { errorCount, warningCount }
}

export function statusNoteFor(errorCount: number, warningCount: number): string {
  if (errorCount > 0) {
    return `${errorCount} error${errorCount === 1 ? '' : 's'} block the build`
  }
  if (warningCount > 0) {
    return `${warningCount} warning${warningCount === 1 ? '' : 's'}`
  }
  return 'All clear'
}

export function filterFindings(
  findings: Finding[],
  rule: RuleFilter,
  file: FileFilter,
): Finding[] {
  return findings.filter((f) => {
    if (rule && f.rule !== rule) return false
    if (file) {
      const base = fileBase(f.path)
      if (f.path !== file && base !== file && !f.path.endsWith(file)) return false
    }
    return true
  })
}

/** Distinct rule ids for filter dropdown. */
export function ruleOptions(findings: Finding[]): string[] {
  return [...new Set(findings.map((f) => f.rule))].sort()
}

/** Distinct file paths for filter dropdown. */
export function fileOptions(findings: Finding[]): string[] {
  return [...new Set(findings.map((f) => f.path).filter(Boolean))].sort()
}

export function toFindingRows(
  findings: Finding[],
  selectedId: string | null,
): FindingRow[] {
  return findings.map((f) => ({
    ...f,
    selected: f.id === selectedId,
    showBothViewports: f.isLayout && f.bothViewports,
    showFixChip: f.hasFix,
    showApply: f.hasFix && APPLY_FIX_READY,
  }))
}


/**
 * Source snippet around a finding span (1-based line/col).
 * Highlights the span line; returns lines with 1-based numbers.
 */
export function sourceSnippet(
  source: string,
  line: number,
  col = 1,
  context = 1,
): { lines: { n: number; text: string; highlight: boolean }[]; highlight: string } {
  const lines = source.split('\n')
  const start = Math.max(1, line - context)
  const end = Math.min(lines.length, line + context)
  const out: { n: number; text: string; highlight: boolean }[] = []
  for (let n = start; n <= end; n++) {
    out.push({ n, text: lines[n - 1] ?? '', highlight: n === line })
  }
  const hl = lines[line - 1] ?? ''
  const caret = col > 1 ? col - 1 : 0
  const highlight =
    hl.length > 0
      ? `${hl.slice(0, caret)}⟨${hl.slice(caret, caret + 1) || ' '}⟩${hl.slice(caret + 1)}`
      : ''
  return { lines: out, highlight }
}

/**
 * Auto-fix on save: when mode is `off`, leave source untouched.
 * `suggest` never writes. `apply` rewrites participation-style grow/shrink
 * chains client-side (mirrors compiler fix payloads).
 */
export function autoFixOnSave(
  source: string,
  findings: Finding[],
  mode: 'off' | 'suggest' | 'apply',
): { text: string; changed: boolean; applied: number } {
  if (mode === 'off' || mode === 'suggest') {
    return { text: source, changed: false, applied: 0 }
  }
  if (!APPLY_FIX_READY) {
    return { text: source, changed: false, applied: 0 }
  }
  let text = source
  let applied = 0
  for (const f of findings) {
    if (
      f.rule === 'sail-participation-style' &&
      (/\sgrow(?:[=\s>]|$)/.test(text) ||
        /\sshrink(?:[=\s>]|$)/.test(text) ||
        text.includes('.grow(') ||
        text.includes('.shrink('))
    ) {
      const next = applyParticipationStyleFix(text)
      if (next !== text) {
        text = next
        applied++
      }
    }
  }
  return { text, changed: applied > 0, applied }
}

/** Client-side participation-style rewrite: bare `grow` / `grow="N"` attrs →
 *  block `style({ box: grow(N) })` (product is the box channel, not attrs). */
export function applyParticipationStyleFix(source: string): string {
  return source
    .replace(
      /<([a-zA-Z][\w-]*)([^>]*?)\s+grow(?:="(\d+)"|=\{(\d+)\})?([^>]*)>/g,
      (_m, tag, pre, nQuoted, nBrace, post) => {
        const n = nQuoted ?? nBrace ?? '1'
        const indent = '  '
        return `<${tag}${pre}${post}>\n${indent}style({ box: grow(${n}) })\n`
      },
    )
    .replace(
      /<([a-zA-Z][\w-]*)([^>]*?)\s+shrink(?:="(\d+)"|=\{(\d+)\})?([^>]*)>/g,
      (_m, tag, pre, nQuoted, nBrace, post) => {
        const n = nQuoted ?? nBrace ?? '1'
        const indent = '  '
        return `<${tag}${pre}${post}>\n${indent}style({ box: shrink(${n}) })\n`
      },
    )
}

/** Apply one finding's known rewrite when possible. */
export function applyFixToSource(
  source: string,
  finding: Finding,
): { ok: boolean; text: string; message: string } {
  if (finding.rule === 'sail-participation-style') {
    const next = applyParticipationStyleFix(source)
    if (next !== source) {
      return { ok: true, text: next, message: 'applied participation-style fix' }
    }
  }
  if (finding.suggest) {
    return {
      ok: false,
      text: source,
      message: `manual suggest: ${finding.suggest}`,
    }
  }
  return { ok: false, text: source, message: 'no machine fix for this finding' }
}
