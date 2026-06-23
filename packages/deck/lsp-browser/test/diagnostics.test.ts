import { describe, it, expect } from 'vitest'
import { diagnose } from '../src/diagnostics'

describe('diagnose', () => {
  it('returns empty array for well-formed template', async () => {
    const src = `<div>
  <p>{count()}</p>
  <button on:click={() => bump()}>+1</button>
</div>`
    expect(await diagnose(src)).toEqual([])
  })

  it('reports unterminated expression', async () => {
    // `{count(}` — unclosed call inside `{}` is caught by the grammar
    const src = `<div>{count(}</div>`
    const diags = await diagnose(src)
    expect(diags.length).toBeGreaterThan(0)
    expect(diags[0].severity).toBe('error')
    expect(diags[0].source).toBe('sui')
    expect(diags[0].line).toBeGreaterThan(0)
  })

  it('reports malformed attribute', async () => {
    const src = `<div on:click=>x</div>`
    const diags = await diagnose(src)
    expect(diags.length).toBeGreaterThan(0)
    expect(diags[0].severity).toBe('error')
  })

  it('1-based line and column on parse error', async () => {
    const src = 'line one\nline two\n<div\n'
    const diags = await diagnose(src)
    expect(diags.length).toBeGreaterThan(0)
    const d = diags[0]
    expect(d.line).toBeGreaterThanOrEqual(1)
    expect(d.column).toBeGreaterThanOrEqual(1)
  })
})
