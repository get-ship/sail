import { describe, expect, it, vi, beforeEach } from 'vitest'
import { useClipboardHook } from '../../src/template-hooks/clipboard'
import type { TemplateHookContext } from '../../src/template-hook'

function makeCtx(
  el: HTMLElement,
  attrs: Record<string, unknown>,
): { ctx: TemplateHookContext<Record<string, unknown>>; cleanups: (() => void)[] } {
  const cleanups: (() => void)[] = []
  const ctx: TemplateHookContext<Record<string, unknown>> = {
    attrs,
    el,
    cleanup: (fn) => cleanups.push(fn),
  }
  return { ctx, cleanups }
}

describe('useClipboardHook', () => {
  let writeText: ReturnType<typeof vi.fn>

  beforeEach(() => {
    writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      writable: true,
      configurable: true,
    })
  })

  it('registers under name "clipboard"', () => {
    expect(useClipboardHook.name).toBe('clipboard')
  })

  it('does nothing when text is not provided', () => {
    const el = document.createElement('button')
    const { ctx, cleanups } = makeCtx(el, { text: undefined })
    expect(() => useClipboardHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)
  })

  it('copies plain string text on click', async () => {
    const el = document.createElement('button')
    const { ctx } = makeCtx(el, { text: 'hello world' })
    useClipboardHook.factory(ctx as never)

    el.dispatchEvent(new MouseEvent('click'))
    await Promise.resolve()

    expect(writeText).toHaveBeenCalledWith('hello world')
  })

  it('copies signal accessor result on click', async () => {
    const el = document.createElement('button')
    const getText = vi.fn().mockReturnValue('dynamic text')
    const { ctx } = makeCtx(el, { text: getText })
    useClipboardHook.factory(ctx as never)

    el.dispatchEvent(new MouseEvent('click'))
    await Promise.resolve()

    expect(getText).toHaveBeenCalled()
    expect(writeText).toHaveBeenCalledWith('dynamic text')
  })

  it('fires onCopy after successful write', async () => {
    const el = document.createElement('button')
    const onCopy = vi.fn()
    const { ctx } = makeCtx(el, { text: 'copied!', onCopy })
    useClipboardHook.factory(ctx as never)

    el.dispatchEvent(new MouseEvent('click'))
    await Promise.resolve()

    expect(onCopy).toHaveBeenCalledTimes(1)
  })

  it('removes click listener on cleanup', async () => {
    const el = document.createElement('button')
    const { ctx, cleanups } = makeCtx(el, { text: 'test' })
    useClipboardHook.factory(ctx as never)

    for (const fn of cleanups) fn()

    el.dispatchEvent(new MouseEvent('click'))
    await Promise.resolve()

    expect(writeText).not.toHaveBeenCalled()
  })
})
