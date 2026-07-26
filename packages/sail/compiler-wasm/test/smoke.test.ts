import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { beforeAll, describe, expect, test } from 'vitest'
// The public entry is the wasm-pack `--target web` bundle in pkg/. It ships as
// @ship/sail-compiler-wasm and powers the docs playground + LSP worker in the
// browser. This smoke test drives it headlessly: initSync from the raw wasm
// bytes (no fetch), then exercise the real compile/analyze surface. pkg/ is
// built by `bun run wasm` (wired as `pretest`), so a fresh checkout is covered.
import {
  analyze,
  compile,
  completions_at,
  completion_vocabulary,
  hover_at,
  initSync,
} from '../pkg/sail_compiler.js'

beforeAll(() => {
  const wasm = fileURLToPath(new URL('../pkg/sail_compiler_bg.wasm', import.meta.url))
  initSync({ module: readFileSync(wasm) })
})

interface WasmSymbol {
  name: string
  kind: string
  source: string
  line: number
  col: number
}

interface WasmHover {
  name: string
  kind: string
  source: string
  line: number
  col: number
}

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

  test('completions_at returns scope symbols from the same table the native LSP uses', () => {
    const src = '<controller>\nexport default controller((ctx) => ({ count: ctx.signal(0) }))\n</controller>\n<p>{count()}</p>'
    const items = completions_at(src) as WasmSymbol[]
    expect(Array.isArray(items)).toBe(true)
    const count = items.find((s) => s.name === 'count')
    expect(count).toBeDefined()
    expect(count!.kind).toBe('value')
    expect(count!.source).toBe('setup')
  })

  test('hover_at resolves the symbol under a 1-based cursor', () => {
    const src = '<controller>\nexport default controller((ctx) => ({ count: ctx.signal(0) }))\n</controller>\n<p>{count()}</p>'
    // `count` starts at column 5 on line 4 (`<p>{count()}</p>`).
    const h = hover_at(src, 4, 5) as WasmHover | undefined
    expect(h).toBeDefined()
    expect(h!.name).toBe('count')
    expect(h!.kind).toBe('value')
  })

  test('hover_at returns undefined when the cursor is not on a symbol', () => {
    const h = hover_at('<column></column>', 1, 3) as WasmHover | undefined
    expect(h).toBeUndefined()
  })

  test('completion_vocabulary exposes compiler-owned directive + block tables', () => {
    const vocab = completion_vocabulary() as {
      events: string[]
      modifiers: Array<{ canonical: string; alias?: string; arg?: string; summary: string }>
      bind_targets: Array<{ target: string; summary: string }>
      blocks: Array<{ open: string; clauses: string[]; summary: string }>
      directives: Array<{ syntax: string; summary: string }>
      retired_directives: Array<{ syntax: string; instead: string }>
      motion_value_forms: Array<{ form: string; summary: string }>
    }
    expect(vocab.events).toContain('click')
    expect(vocab.events).toContain('focusin')
    expect(vocab.modifiers.some((m) => m.canonical === 'preventDefault')).toBe(true)
    expect(vocab.modifiers.some((m) => m.canonical === 'preventDefault' && m.alias === 'prevent')).toBe(true)
    expect(vocab.bind_targets.map((b) => b.target)).toContain('value')
    expect(vocab.blocks.map((b) => b.open)).toContain('<match on={…}>')
    expect(vocab.directives.some((d) => d.syntax.includes('on:'))).toBe(true)
    expect(vocab.retired_directives.some((d) => d.syntax.includes('style:'))).toBe(true)
    expect(vocab.motion_value_forms.some((f) => f.form.includes('quoted'))).toBe(true)
  })
})
