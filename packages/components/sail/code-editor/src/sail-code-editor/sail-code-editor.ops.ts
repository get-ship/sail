/**
 * Thin SailAuthoring HTTP client for <sail-code-editor> when `ops-url` is set.
 *
 * Contract (SPEC-LSP-1-studio-bridge):
 *   POST {base}/sail.ops.v1.SailAuthoring/Hover    { source?, path?, line, column } → { contents?, range? }
 *   POST {base}/sail.ops.v1.SailAuthoring/Complete { source?, path?, line, column } → { items: [...] }
 *   Diagnostics: Diagnose (preferred) or Check — poll and merge in the editor.
 *
 * Feature-detects missing methods (404 / unknown method) and degrades to
 * structure-only hover/completions/diags. No hard dependency on studio-client.
 */

export interface OpsHoverResult {
  contents: string
  range?: {
    line?: number
    column?: number
    endLine?: number
    endColumn?: number
  }
}

export interface OpsCompletionItem {
  label: string
  detail?: string
  info?: string
  insert?: string
  kind?: string
}

export interface OpsDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

export interface OpsPosArgs {
  source?: string
  path?: string
  line: number
  column: number
  signal?: any
}

export interface OpsDiagArgs {
  source?: string
  path?: string
  signal?: any
}

export interface OpsAuthoringClient {
  hover(args: OpsPosArgs): Promise<OpsHoverResult | null>
  complete(args: OpsPosArgs): Promise<OpsCompletionItem[]>
  diagnostics(args: OpsDiagArgs): Promise<OpsDiagnostic[]>
  /** True after a successful call, false after feature-detect failure. */
  supportsHover(): boolean | null
  supportsComplete(): boolean | null
  supportsDiagnostics(): boolean | null
}

type MethodKey = 'Hover' | 'Complete' | 'Diagnose' | 'Check'

/**
 * Normalize ops base URL.
 * - falsy / whitespace → null (ops disabled)
 * - trailing slash stripped
 * - bare `/` → `''` (same-origin relative, matches createOpsClient default)
 */
export function normalizeOpsBase(url: string | null | undefined): string | null {
  if (url == null) return null
  const t = String(url).trim()
  if (!t) return null
  const noTrail = t.replace(/\/+$/, '')
  // `/` alone becomes empty after strip → same-origin
  return noTrail
}

/** Convert absolute UTF-16 offset → 1-based line/column (Sail/LSP convention). */
export function offsetToLineCol(
  source: string,
  offset: number,
): { line: number; column: number } {
  const safe = Math.max(0, Math.min(offset, source.length))
  let line = 1
  let column = 1
  for (let i = 0; i < safe; i++) {
    if (source.charCodeAt(i) === 10 /* \n */) {
      line++
      column = 1
    } else {
      column++
    }
  }
  return { line, column }
}

function methodPath(base: string, method: MethodKey): string {
  return `${base}/sail.ops.v1.SailAuthoring/${method}`
}

function isUnavailableStatus(status: number): boolean {
  return status === 404 || status === 405 || status === 501
}

function looksLikeUnknownMethod(body: unknown, status: number): boolean {
  if (isUnavailableStatus(status)) return true
  if (!body || typeof body !== 'object') return false
  const o = body as { code?: string; message?: string }
  const msg = typeof o.message === 'string' ? o.message.toLowerCase() : ''
  const code = typeof o.code === 'string' ? o.code.toLowerCase() : ''
  return (
    code.includes('unimplemented') ||
    code.includes('not_found') ||
    code.includes('notfound') ||
    msg.includes('unknown method') ||
    msg.includes('not found') ||
    msg.includes('unimplemented') ||
    msg.includes('no such method')
  )
}

function asSeverity(v: unknown): 'error' | 'warning' | 'info' {
  const s = String(v ?? 'error').toLowerCase()
  if (s === 'warning' || s === 'warn') return 'warning'
  if (s === 'info' || s === 'hint') return 'info'
  return 'error'
}

/** Parse Check/Diagnose diagnostics array into editor shape. */
export function parseOpsDiagnostics(raw: unknown): OpsDiagnostic[] {
  if (!Array.isArray(raw)) return []
  const out: OpsDiagnostic[] = []
  for (const d of raw) {
    if (!d || typeof d !== 'object') continue
    const o = d as Record<string, unknown>
    const line = Number(o.line ?? o.startLine ?? o.start_line ?? 1) || 1
    const colRaw = o.column ?? o.col ?? o.startColumn ?? o.start_col
    const endLineRaw = o.endLine ?? o.end_line
    const endColRaw = o.endColumn ?? o.end_col ?? o.endCol
    const message = String(o.message ?? o.msg ?? '')
    if (!message) continue
    const item: OpsDiagnostic = {
      line,
      severity: asSeverity(o.severity),
      message,
      source: o.source != null ? String(o.source) : 'ops',
    }
    if (colRaw != null && colRaw !== '') item.column = Number(colRaw) || 1
    if (endLineRaw != null && endLineRaw !== '') item.endLine = Number(endLineRaw) || line
    if (endColRaw != null && endColRaw !== '') {
      const n = Number(endColRaw)
      if (n) item.endColumn = n
    }
    out.push(item)
  }
  return out
}

function parseHoverBody(json: unknown): OpsHoverResult | null {
  if (json == null) return null
  if (typeof json === 'string') {
    const t = json.trim()
    return t ? { contents: t } : null
  }
  if (typeof json !== 'object') return null
  const o = json as Record<string, unknown>
  // Null-ish empty body from server
  if (o.contents == null && o.markdown == null && o.value == null) {
    // Some servers wrap: { hover: { contents } }
    if (o.hover && typeof o.hover === 'object') {
      return parseHoverBody(o.hover)
    }
    return null
  }
  const contents = String(o.contents ?? o.markdown ?? o.value ?? '').trim()
  if (!contents) return null
  const range = o.range
  const result: OpsHoverResult = { contents }
  if (range && typeof range === 'object') {
    const r = range as Record<string, unknown>
    const next: NonNullable<OpsHoverResult['range']> = {}
    if (r.line != null) next.line = Number(r.line)
    if (r.column != null) next.column = Number(r.column)
    else if (r.col != null) next.column = Number(r.col)
    if (r.endLine != null) next.endLine = Number(r.endLine)
    else if (r.end_line != null) next.endLine = Number(r.end_line)
    if (r.endColumn != null) next.endColumn = Number(r.endColumn)
    else if (r.end_col != null) next.endColumn = Number(r.end_col)
    result.range = next
  }
  return result
}

function parseCompleteItems(json: unknown): OpsCompletionItem[] {
  if (!json || typeof json !== 'object') return []
  const o = json as Record<string, unknown>
  const raw = Array.isArray(o.items) ? o.items : Array.isArray(json) ? (json as unknown[]) : []
  const out: OpsCompletionItem[] = []
  for (const it of raw) {
    if (!it || typeof it !== 'object') continue
    const x = it as Record<string, unknown>
    const label = String(x.label ?? x.name ?? '').trim()
    if (!label) continue
    const item: OpsCompletionItem = { label }
    if (x.detail != null) item.detail = String(x.detail)
    if (x.info != null) item.info = String(x.info)
    else if (x.documentation != null) item.info = String(x.documentation)
    if (x.insert != null) item.insert = String(x.insert)
    else if (x.insertText != null) item.insert = String(x.insertText)
    else if (x.textEdit && typeof x.textEdit === 'object') {
      const te = x.textEdit as { newText?: string }
      if (te.newText != null) item.insert = String(te.newText)
    }
    if (x.kind != null) item.kind = String(x.kind)
    out.push(item)
  }
  return out
}

export function createOpsAuthoringClient(opts: {
  baseUrl: string
  fetch?: (...a: any[]) => Promise<any>
}): OpsAuthoringClient {
  const base = normalizeOpsBase(opts.baseUrl) ?? ''
  const g = globalThis as { fetch?: (...a: any[]) => Promise<any> }
  const fetchFn = opts.fetch ?? (g.fetch ? g.fetch.bind(g) : (() => { throw new Error("fetch unavailable") }))

  // null = unknown, true = works, false = unavailable
  let hoverOk: boolean | null = null
  let completeOk: boolean | null = null
  let diagnoseOk: boolean | null = null
  let checkOk: boolean | null = null

  async function post(
    method: MethodKey,
    body: object,
    signal?: any,
  ): Promise<{ ok: true; json: unknown } | { ok: false; unavailable: boolean; status: number }> {
    let res: any
    try {
      res = await fetchFn(methodPath(base, method), {
        method: 'POST',
        headers: { 'content-type': 'application/json', accept: 'application/json' },
        body: JSON.stringify(body),
        cache: 'no-store',
        signal,
      })
    } catch (err) {
      if (signal?.aborted) throw err
      // Network error — not a feature-detect fail; treat as soft miss
      return { ok: false, unavailable: false, status: 0 }
    }
    const json = await res.json().catch(() => ({}))
    if (!res.ok || (json && typeof json === 'object' && (json as { code?: string }).code)) {
      const unavailable = looksLikeUnknownMethod(json, res.status)
      return { ok: false, unavailable, status: res.status }
    }
    return { ok: true, json }
  }

  return {
    supportsHover: () => hoverOk,
    supportsComplete: () => completeOk,
    supportsDiagnostics: () =>
      diagnoseOk === true || checkOk === true
        ? true
        : diagnoseOk === false && checkOk === false
          ? false
          : null,

    async hover(args) {
      if (hoverOk === false) return null
      const body: Record<string, unknown> = {
        line: args.line,
        column: args.column,
      }
      if (args.source != null) body.source = args.source
      if (args.path) body.path = args.path
      const res = await post('Hover', body, args.signal)
      if (!res.ok) {
        if (res.unavailable) hoverOk = false
        return null
      }
      hoverOk = true
      return parseHoverBody(res.json)
    },

    async complete(args) {
      if (completeOk === false) return []
      const body: Record<string, unknown> = {
        line: args.line,
        column: args.column,
      }
      if (args.source != null) body.source = args.source
      if (args.path) body.path = args.path
      const res = await post('Complete', body, args.signal)
      if (!res.ok) {
        if (res.unavailable) completeOk = false
        return []
      }
      completeOk = true
      return parseCompleteItems(res.json)
    },

    async diagnostics(args) {
      const body: Record<string, unknown> = {}
      if (args.source != null) body.source = args.source
      if (args.path) body.path = args.path
      // Prefer Diagnose (structure+tsgo merge) when present; fall back to Check.
      if (diagnoseOk !== false) {
        const res = await post('Diagnose', { ...body, includeTs: true }, args.signal)
        if (res.ok) {
          diagnoseOk = true
          const j = res.json as Record<string, unknown>
          return parseOpsDiagnostics(j.diagnostics ?? j)
        }
        if (res.unavailable) diagnoseOk = false
        // non-unavailable error → try Check
      }
      if (checkOk === false) return []
      const res = await post('Check', { ...body, includeTs: true }, args.signal)
      if (!res.ok) {
        if (res.unavailable) checkOk = false
        return []
      }
      checkOk = true
      const j = res.json as Record<string, unknown>
      return parseOpsDiagnostics(j.diagnostics ?? j)
    },
  }
}
