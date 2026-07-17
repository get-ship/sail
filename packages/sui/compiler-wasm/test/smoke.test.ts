import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { beforeAll, describe, expect, test } from 'vitest'
// The public entry is the wasm-pack `--target web` bundle in pkg/. It ships as
// @ship/sail-compiler-wasm and powers the docs playground + LSP worker in the
// browser. This smoke test drives it headlessly: initSync from the raw wasm
// bytes (no fetch), then exercise the real compile/analyze surface. pkg/ is
// built by `bun run wasm` (wired as `pretest`), so a fresh checkout is covered.
import { analyze, compile, initSync } from '../pkg/sui_compiler.js'

beforeAll(() => {
  const wasm = fileURLToPath(new URL('../pkg/sui_compiler_bg.wasm', import.meta.url))
  initSync({ module: readFileSync(wasm) })
})

describe('@ship/sail-compiler-wasm', () => {
  test('compiles a trivial source to runtime-bound component JS', () => {
    const out = compile('<column><p>hi</p></column>', '{}')
    expect(out.kind).toBe('ok')
    expect(typeof out.js).toBe('string')
    expect(out.js.length).toBeGreaterThan(0)
    // The emitted module hydrates against the runtime, not a re-parse at load.
    expect(out.js).toContain('@ship/sail-runtime')
    expect(out.js).toContain('component(')
    // css/dts are part of the result shape; they're populated only when the
    // source carries a style block / typed controller, so assert the contract
    // (fields present, string-or-undefined) rather than non-empty here.
    expect(out).toHaveProperty('css')
    expect(out).toHaveProperty('dts')
  })

  test('reports a structured error rather than throwing', () => {
    const out = compile('<column><p>hi', '{}')
    expect(out.kind).toBe('err')
    expect(out.message).toContain('unterminated')
    expect(typeof out.line).toBe('number')
    expect(typeof out.col).toBe('number')
  })

  test('analyze surfaces the LSP stream (diagnostics + scope) for one source', () => {
    const a = analyze('<column><p>hi</p></column>')
    expect(Array.isArray(a.diagnostics)).toBe(true)
    expect(a).toHaveProperty('symbols')
    expect(a).toHaveProperty('tags')
    expect(a).toHaveProperty('events')
  })
})
