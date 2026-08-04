// Unit tests for SailAuthoring ops client used by <sail-code-editor ops-url>.
// Mocks fetch; no CodeMirror mount required for the Hover RPC contract path.

import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createOpsAuthoringClient,
  normalizeOpsBase,
  offsetToLineCol,
  parseOpsDiagnostics,
} from '../src/sail-code-editor/sail-code-editor.ops'

afterEach(() => {
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe('normalizeOpsBase', () => {
  it('disables on empty / whitespace', () => {
    expect(normalizeOpsBase('')).toBeNull()
    expect(normalizeOpsBase('   ')).toBeNull()
    expect(normalizeOpsBase(null)).toBeNull()
    expect(normalizeOpsBase(undefined)).toBeNull()
  })

  it('maps bare "/" to same-origin empty base', () => {
    expect(normalizeOpsBase('/')).toBe('')
    expect(normalizeOpsBase('https://ops.example/')).toBe('https://ops.example')
  })
})

describe('offsetToLineCol', () => {
  it('returns 1-based line/column', () => {
    expect(offsetToLineCol('ab\ncd', 0)).toEqual({ line: 1, column: 1 })
    expect(offsetToLineCol('ab\ncd', 2)).toEqual({ line: 1, column: 3 })
    expect(offsetToLineCol('ab\ncd', 3)).toEqual({ line: 2, column: 1 })
    expect(offsetToLineCol('ab\ncd', 5)).toEqual({ line: 2, column: 3 })
  })
})

describe('parseOpsDiagnostics', () => {
  it('maps Check/Diagnose diagnostic shapes', () => {
    const out = parseOpsDiagnostics([
      { line: 2, col: 4, end_line: 2, end_col: 10, severity: 'warning', message: 'x', rule: 'r' },
      { line: 1, column: 1, severity: 'error', message: 'y', source: 'tsgo' },
    ])
    expect(out).toHaveLength(2)
    expect(out[0]).toMatchObject({
      line: 2,
      column: 4,
      endLine: 2,
      endColumn: 10,
      severity: 'warning',
      message: 'x',
    })
    expect(out[1]).toMatchObject({ line: 1, column: 1, severity: 'error', source: 'tsgo' })
  })
})

describe('createOpsAuthoringClient — Hover', () => {
  it('POSTs SailAuthoring/Hover with 1-based line/col and returns contents', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      expect(String(input)).toBe('/sail.ops.v1.SailAuthoring/Hover')
      expect(init?.method).toBe('POST')
      const body = JSON.parse(String(init?.body ?? '{}')) as Record<string, unknown>
      expect(body).toMatchObject({
        source: 'const n = signal(0)',
        path: 'apps/studio/routes/page.sail',
        line: 1,
        column: 11,
      })
      return new Response(JSON.stringify({ contents: '```ts\nfunction signal<T>(v: T): signal<T>\n```' }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    })
    const client = createOpsAuthoringClient({ baseUrl: '/', fetch: fetchMock as unknown as typeof fetch })
    const hover = await client.hover({
      source: 'const n = signal(0)',
      path: 'apps/studio/routes/page.sail',
      line: 1,
      column: 11,
    })
    expect(hover?.contents).toContain('signal<T>')
    expect(client.supportsHover()).toBe(true)
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('feature-detects 404 Hover and stops calling', async () => {
    const fetchMock = vi.fn(async () => new Response('not found', { status: 404 }))
    const client = createOpsAuthoringClient({ baseUrl: '/', fetch: fetchMock as unknown as typeof fetch })
    expect(await client.hover({ line: 1, column: 1, source: 'x' })).toBeNull()
    expect(client.supportsHover()).toBe(false)
    expect(await client.hover({ line: 1, column: 1, source: 'x' })).toBeNull()
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })
})

describe('createOpsAuthoringClient — Complete', () => {
  it('POSTs Complete and maps items', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      expect(String(input)).toBe('/sail.ops.v1.SailAuthoring/Complete')
      expect(init?.method).toBe('POST')
      const body = JSON.parse(String(init?.body ?? '{}')) as Record<string, unknown>
      expect(body).toMatchObject({ source: 'sig', line: 3, column: 2 })
      return new Response(
        JSON.stringify({
          items: [
            { label: 'signal', detail: 'function', insertText: 'signal' },
            { label: 'computed', info: 'derived' },
          ],
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    })
    const client = createOpsAuthoringClient({
      baseUrl: '/',
      fetch: fetchMock as unknown as typeof fetch,
    })
    const items = await client.complete({ source: 'sig', line: 3, column: 2 })
    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(items).toEqual([
      { label: 'signal', detail: 'function', insert: 'signal' },
      { label: 'computed', info: 'derived' },
    ])
    expect(client.supportsComplete()).toBe(true)
  })
})

describe('createOpsAuthoringClient — diagnostics', () => {
  it('falls back from Diagnose 404 to Check', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.endsWith('/Diagnose')) {
        return new Response(JSON.stringify({ code: 'not_found', message: 'unknown method' }), {
          status: 404,
        })
      }
      if (url.endsWith('/Check')) {
        return new Response(
          JSON.stringify({
            ok: true,
            count: 1,
            diagnostics: [{ line: 4, col: 2, severity: 'error', message: 'TS2322' }],
          }),
          { status: 200, headers: { 'content-type': 'application/json' } },
        )
      }
      return new Response('no', { status: 500 })
    })
    const client = createOpsAuthoringClient({ baseUrl: '/', fetch: fetchMock as unknown as typeof fetch })
    const diags = await client.diagnostics({ source: 'x', path: 'a.sail' })
    expect(diags).toHaveLength(1)
    expect(diags[0]).toMatchObject({ line: 4, column: 2, message: 'TS2322' })
    expect(fetchMock).toHaveBeenCalledTimes(2)
  })

  it('cancels in-flight via AbortSignal', async () => {
    const ac = new AbortController()
    const fetchMock = vi.fn((_input: RequestInfo | URL, init?: RequestInit) => {
      return new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener('abort', () => {
          reject(new DOMException('aborted', 'AbortError'))
        })
      })
    })
    const client = createOpsAuthoringClient({ baseUrl: '/', fetch: fetchMock as unknown as typeof fetch })
    const p = client.hover({ line: 1, column: 1, source: 'x', signal: ac.signal })
    ac.abort()
    await expect(p).rejects.toBeTruthy()
  })
})
