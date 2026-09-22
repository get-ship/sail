import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { clipboard } from '../src/actions/clipboard'
import { runAction } from '../src/action'
import { createScope, withScope } from '../src/effect-scope'

describe('use:clipboard', () => {
  let writeText: ReturnType<typeof vi.fn>

  beforeEach(() => {
    writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('copies param text and fires onCopy on success', async () => {
    const onCopy = vi.fn()
    const el = document.createElement('button')
    el.textContent = 'fallback'
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, clipboard, () => ({ text: 'hello', onCopy }))
    })
    el.click()
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith('hello'))
    await vi.waitFor(() => expect(onCopy).toHaveBeenCalledOnce())
    scope.dispose()
    el.remove()
  })

  it('falls back to host text content when param is empty', async () => {
    const el = document.createElement('button')
    el.textContent = '  copy me  '
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, clipboard, () => undefined)
    })
    el.click()
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith('copy me'))
    scope.dispose()
    el.remove()
  })
})
