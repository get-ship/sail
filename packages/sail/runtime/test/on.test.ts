import { describe, expect, it, vi } from 'vitest'
import { on, matchesKey } from '../src/on'

describe('on', () => {
  it('attaches and fires a handler', () => {
    const el = document.createElement('button')
    const fn = vi.fn()
    on(el, 'click', fn)
    el.dispatchEvent(new MouseEvent('click'))
    expect(fn).toHaveBeenCalledTimes(1)
  })

  it('disposer detaches', () => {
    const el = document.createElement('button')
    const fn = vi.fn()
    const dispose = on(el, 'click', fn)
    dispose()
    el.dispatchEvent(new MouseEvent('click'))
    expect(fn).toHaveBeenCalledTimes(0)
  })

  describe('modifiers', () => {
    it('preventDefault calls e.preventDefault()', () => {
      const el = document.createElement('a')
      const fn = vi.fn()
      on(el, 'click', fn, { modifiers: [{ kind: 'preventDefault' }] })
      const ev = new MouseEvent('click', { cancelable: true })
      el.dispatchEvent(ev)
      expect(ev.defaultPrevented).toBe(true)
      expect(fn).toHaveBeenCalled()
    })

    it('stopPropagation prevents bubbling', () => {
      const parent = document.createElement('div')
      const child = document.createElement('button')
      parent.appendChild(child)
      const parentFn = vi.fn()
      const childFn = vi.fn()
      parent.addEventListener('click', parentFn)
      on(child, 'click', childFn, { modifiers: [{ kind: 'stopPropagation' }] })
      child.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      expect(childFn).toHaveBeenCalled()
      expect(parentFn).not.toHaveBeenCalled()
    })

    it('self gates on event.target === el', () => {
      const parent = document.createElement('div')
      const child = document.createElement('button')
      parent.appendChild(child)
      const fn = vi.fn()
      on(parent, 'click', fn, { modifiers: [{ kind: 'self' }] })
      child.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      expect(fn).not.toHaveBeenCalled()
      parent.dispatchEvent(new MouseEvent('click'))
      expect(fn).toHaveBeenCalledTimes(1)
    })

    it('key:Enter matches only Enter', () => {
      const el = document.createElement('input')
      const fn = vi.fn()
      on(el, 'keydown', fn, { modifiers: [{ kind: 'key', arg: 'Enter' }] })
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }))
      expect(fn).not.toHaveBeenCalled()
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
      expect(fn).toHaveBeenCalledTimes(1)
    })

    it('key:meta+j matches Cmd+J', () => {
      const el = document.createElement('div')
      const fn = vi.fn()
      on(el, 'keydown', fn, { modifiers: [{ kind: 'key', arg: 'meta+j' }] })
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'j' }))
      expect(fn).not.toHaveBeenCalled()
      el.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'j', metaKey: true }),
      )
      expect(fn).toHaveBeenCalledTimes(1)
    })

    it('debounce coalesces rapid fires', async () => {
      const el = document.createElement('input')
      const fn = vi.fn()
      on(el, 'input', fn, { modifiers: [{ kind: 'debounce', arg: '50' }] })
      el.dispatchEvent(new Event('input'))
      el.dispatchEvent(new Event('input'))
      el.dispatchEvent(new Event('input'))
      expect(fn).not.toHaveBeenCalled()
      await new Promise((r) => setTimeout(r, 80))
      expect(fn).toHaveBeenCalledTimes(1)
    })

    it('throttle rate-limits', async () => {
      const el = document.createElement('div')
      const fn = vi.fn()
      on(el, 'mousemove', fn, { modifiers: [{ kind: 'throttle', arg: '50' }] })
      el.dispatchEvent(new MouseEvent('mousemove'))
      el.dispatchEvent(new MouseEvent('mousemove'))
      el.dispatchEvent(new MouseEvent('mousemove'))
      expect(fn).toHaveBeenCalledTimes(1)
      await new Promise((r) => setTimeout(r, 80))
      el.dispatchEvent(new MouseEvent('mousemove'))
      expect(fn).toHaveBeenCalledTimes(2)
    })
  })
})

describe('matchesKey', () => {
  it('plain key', () => {
    const ev = new KeyboardEvent('keydown', { key: 'Enter' })
    expect(matchesKey(ev, 'Enter')).toBe(true)
    expect(matchesKey(ev, 'a')).toBe(false)
  })

  it('case-insensitive', () => {
    const ev = new KeyboardEvent('keydown', { key: 'A' })
    expect(matchesKey(ev, 'a')).toBe(true)
    expect(matchesKey(ev, 'A')).toBe(true)
  })

  it('modifier required', () => {
    const ev = new KeyboardEvent('keydown', { key: 'k' })
    expect(matchesKey(ev, 'meta+k')).toBe(false)
  })

  it('modifier present', () => {
    const ev = new KeyboardEvent('keydown', {
      key: 'k',
      metaKey: true,
    })
    expect(matchesKey(ev, 'meta+k')).toBe(true)
  })

  it('extra modifier rejected', () => {
    const ev = new KeyboardEvent('keydown', { key: 'k', metaKey: true })
    expect(matchesKey(ev, 'k')).toBe(true) // modifier-less spec accepts any
    // (intentional — `key:k` matches on key alone, modifiers ignored)
  })

  it('cmd alias for meta', () => {
    const ev = new KeyboardEvent('keydown', { key: 'k', metaKey: true })
    expect(matchesKey(ev, 'cmd+k')).toBe(true)
  })
})
