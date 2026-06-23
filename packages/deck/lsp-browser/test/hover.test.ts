import { describe, it, expect } from 'vitest'
import { hoverAt } from '../src/hover'

describe('hoverAt', () => {
  it('returns null when the cursor is outside any token', () => {
    expect(hoverAt('<div>x</div>', 0)).toBeNull()
  })

  it('hovers the on: directive', () => {
    const src = '<button on:click={ok}>x</button>'
    // Cursor inside "on:"
    const info = hoverAt(src, 10)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('on:<event>')
    expect(info!.contents).toContain('DOM event listener')
  })

  it('hovers the bind: directive', () => {
    const src = '<input bind:value={x} />'
    const info = hoverAt(src, 12)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('bind:<target>')
  })

  it('hovers a modifier after the pipe', () => {
    const src = '<form on:submit|preventDefault={save}>x</form>'
    // Cursor inside "preventDefault"
    const info = hoverAt(src, 22)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('|preventDefault')
    expect(info!.contents).toContain('e.preventDefault()')
  })

  it('hovers a key:<Name> modifier via its base', () => {
    const src = '<input on:keydown|key:Enter={ok} />'
    // Cursor inside "key:Enter"
    const info = hoverAt(src, 22)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('|key:<KeyName>')
  })

  it('hovers a block keyword after {#', () => {
    const src = '{#if x()}<p>hi</p>{/if}'
    // Cursor inside "if"
    const info = hoverAt(src, 3)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('{#if expr}')
  })

  it('hovers while-loading block', () => {
    const src = '{#while-loading of items}…{/while-loading}'
    const info = hoverAt(src, 6)
    expect(info).not.toBeNull()
    expect(info!.contents).toContain('while-loading')
    expect(info!.contents).toContain('live signal')
  })

  it('returns null on a plain word that has no entry', () => {
    expect(hoverAt('<div>hello</div>', 7)).toBeNull()
  })
})
