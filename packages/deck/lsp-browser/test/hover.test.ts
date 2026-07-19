import { describe, it, expect, beforeAll } from 'vitest'
import { hoverAt } from '../src/hover'

beforeAll(async () => {
  // The module initializes WASM at the top level; waiting here ensures
  // vocabulary is loaded before any assertion runs.
  await new Promise((resolve) => setTimeout(resolve, 0))
})

describe('hoverAt', () => {
  it('returns null when the cursor is outside any token', () => {
    expect(hoverAt('<div>x</div>', 0)).toBeNull()
  })

  it('hovers the on: directive from compiler vocabulary', () => {
    const src = '<button on:click={ok}>x</button>'
    const info = hoverAt(src, 10)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('on:')
    expect(info!.contents).toContain('event')
  })

  it('hovers the bind: directive from compiler vocabulary', () => {
    const src = '<input bind:value={x} />'
    const info = hoverAt(src, 12)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('bind:')
    expect(info!.contents).toContain('two-way')
  })

  it('hovers a modifier after the pipe from compiler vocabulary', () => {
    const src = '<form on:submit|preventDefault={save}>x</form>'
    const info = hoverAt(src, 22)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('|preventDefault')
    expect(info!.contents).toContain('preventDefault')
  })

  it('hovers a key:<Name> modifier via its base', () => {
    const src = '<input on:keydown|key:Enter={ok} />'
    const info = hoverAt(src, 22)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('|key:')
  })

  it('hovers a block keyword after {# from compiler vocabulary', () => {
    const src = '{#if x()}<p>hi</p>{/if}'
    const info = hoverAt(src, 3)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('{#if}')
    expect(info!.contents).toContain('conditional')
  })

  it('hovers while-loading block from compiler vocabulary', () => {
    const src = '{#while-loading of items}…{/while-loading}'
    const info = hoverAt(src, 6)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('{#while-loading}')
    expect(info!.contents).toContain('async')
  })

  it('returns null on a plain word that has no entry', () => {
    expect(hoverAt('<div>hello</div>', 7)).toBeNull()
  })
})
