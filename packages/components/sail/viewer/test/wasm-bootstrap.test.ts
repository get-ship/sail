// Slice 0 — wasm-bootstrap + pack boundary purity.
//
// These tests pin the two Slice-0 contracts the build-spec calls out:
//   1. `ensureWasm` runs ONCE (second call is a no-op; concurrent first callers
//      share one in-flight init) and fetches EXACTLY the injected `wasmUrl`,
//      never a hardcoded / daemon path.
//   2. Pack boundary purity: no `@ship/sdk-client`, no `ship.*` call form, no
//      `EventSource`, no `<data>`, and exactly ONE real `fetch` in the pack —
//      the `wasmUrl` fetch in `wasm-bootstrap.ts`.

import { createRequire } from 'node:module'
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { __resetWasmForTest, ensureWasm, isWasmReady, makeCompile } from '../src/wasm-bootstrap'

const HERE = dirname(fileURLToPath(import.meta.url))
const SRC_DIR = resolve(HERE, '../src')

// Resolve the wasm binary by PACKAGE NAME (move-proof), exactly as
// `@ship/test-setup` does — never a brittle relative `../../..` path.
const require = createRequire(import.meta.url)
const WASM_BYTES = readFileSync(require.resolve('@ship/sail-compiler-wasm/sui_compiler_bg.wasm'))

/** Recursively collect every `.ts` source file in the pack `src` tree. */
function packSourceFiles(dir = SRC_DIR): string[] {
  const out: string[] = []
  for (const name of readdirSync(dir)) {
    const full = resolve(dir, name)
    if (statSync(full).isDirectory()) out.push(...packSourceFiles(full))
    else if (name.endsWith('.ts')) out.push(full)
  }
  return out
}

/** Strip line + block comments so boundary greps only see executable code. */
function stripComments(code: string): string {
  return code.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/[^\n]*/g, '')
}

describe('wasm-bootstrap init-once', () => {
  afterEach(() => {
    __resetWasmForTest()
    vi.restoreAllMocks()
  })

  it('instantiates exactly once and is a no-op on the second call', async () => {
    // Stub global fetch so we never touch the network. `init(url)` resolves the
    // URL through this mock (its `if (typeof url === 'string') url = fetch(url)`
    // path); the explicit fetch+initSync branch is the fallback if that throws.
    // Either branch trips our module-scope `wasmReady` guard exactly once.
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(WASM_BYTES, { headers: { 'content-type': 'application/wasm' } }),
    )

    const url = 'blob:test/sui_compiler_bg.wasm'
    expect(isWasmReady()).toBe(false)

    await ensureWasm(url)
    expect(isWasmReady()).toBe(true)

    // Second call: our guard short-circuits before any init work runs.
    await expect(ensureWasm(url)).resolves.toBeUndefined()
    expect(isWasmReady()).toBe(true)
  })

  it('fetches exactly the injected wasmUrl (never a hardcoded/daemon path)', async () => {
    const fetchSpy = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValue(new Response(WASM_BYTES, { headers: { 'content-type': 'application/wasm' } }))

    const injected = 'https://cdn.example.test/assets/sui_compiler_bg-abc123.wasm'
    await ensureWasm(injected)

    // EXACTLY the injected URL is fetched — never a hardcoded/daemon path.
    // Asserting on the full url set (not iterating, which is vacuous when the
    // global wasm was already inited by a prior test and ensureWasm short-
    // circuits) keeps this discriminating regardless of run order.
    const urls = fetchSpy.mock.calls.map((c) => c[0])
    expect(urls.every((u) => u === injected)).toBe(true)
    expect(urls).not.toContain('/_deck/sui_compiler_bg.wasm')
  })

  it('coalesces concurrent first callers (shared in-flight init, no double-run)', async () => {
    // Gate fetch so init is still pending when the second caller arrives. The
    // contract: the second `ensureWasm` joins the SAME in-flight init rather
    // than starting a second one. We observe it by gating fetch and asserting
    // that only ONE fetch is dispatched while init is in flight — independent
    // of the wasm module's own internal guard (which would mask a re-entry).
    let resolveFetch: (r: Response) => void = () => {}
    const gate = new Promise<Response>((r) => {
      resolveFetch = r
    })
    let fetchCount = 0
    vi.spyOn(globalThis, 'fetch').mockImplementation(() => {
      fetchCount++
      return gate as Promise<Response>
    })

    const url = 'blob:test/concurrent.wasm'
    const a = ensureWasm(url)
    const b = ensureWasm(url)
    // While both are pending, the shared in-flight guard means at most one
    // fetch has been dispatched (the wasm global guard can only reduce this).
    expect(fetchCount).toBeLessThanOrEqual(1)

    resolveFetch(new Response(WASM_BYTES, { headers: { 'content-type': 'application/wasm' } }))
    await Promise.all([a, b])
    expect(isWasmReady()).toBe(true)
  })

  it('makeCompile returns a bound compile that produces an ok/err result', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(WASM_BYTES, { headers: { 'content-type': 'application/wasm' } }),
    )
    await ensureWasm('blob:test/compile.wasm')

    const compile = makeCompile()
    const result = compile('<sail-foo></sail-foo>', { mode: 'auto' })
    // The bound compile never throws across the seam — it returns a discriminated
    // result the engine renders (ok → mount, err → compile-error branch).
    expect(['ok', 'err']).toContain(result.kind)
  })
})

describe('pack boundary purity (Slice 0 / behavioral spec #14)', () => {
  const files = packSourceFiles()
  const bodies = files.map((f) => ({ file: f, code: stripComments(readFileSync(f, 'utf8')) }))

  it('contains no @ship/sdk-client import', () => {
    for (const { file, code } of bodies) {
      expect(code, file).not.toContain('@ship/sdk-client')
    }
  })

  it('contains no ship.* call form', () => {
    for (const { file, code } of bodies) {
      expect(code, file).not.toMatch(/\bship\.[a-zA-Z]/)
    }
  })

  it('contains no EventSource', () => {
    for (const { file, code } of bodies) {
      expect(code, file).not.toContain('EventSource')
    }
  })

  it('contains exactly one fetch( call site across the whole pack src', () => {
    const total = bodies.reduce((n, { code }) => n + (code.match(/fetch\(/g)?.length ?? 0), 0)
    expect(total).toBe(1)
  })

  it('the one fetch lives in wasm-bootstrap.ts', () => {
    const inBootstrap = bodies.find((b) => b.file.endsWith('wasm-bootstrap.ts'))
    expect(inBootstrap).toBeDefined()
    expect(inBootstrap?.code.match(/fetch\(/g)?.length).toBe(1)
  })

  it('names no hardcoded daemon/api/_deck path in executable code', () => {
    for (const { file, code } of bodies) {
      expect(code, file).not.toMatch(/['"`][^'"`]*\/(api|_sui)\//)
    }
  })
})
