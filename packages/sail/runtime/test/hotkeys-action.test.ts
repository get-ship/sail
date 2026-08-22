import { afterEach, describe, expect, it, vi } from 'vitest'
import { hotkeys } from '../src/actions/hotkeys'
import { runAction } from '../src/action'
import { createScope, withScope } from '../src/effect-scope'

function keydown(
  key: string,
  mods: { meta?: boolean; ctrl?: boolean; shift?: boolean } = {},
  target?: Element,
): void {
  const e = new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
    metaKey: !!mods.meta,
    ctrlKey: !!mods.ctrl,
    shiftKey: !!mods.shift,
  })
  ;(target ?? document).dispatchEvent(e)
}

describe('use:hotkeys', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('fires handler for matching key', () => {
    const save = vi.fn()
    const el = document.createElement('div')
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, hotkeys, () => ({ s: save }))
    })
    keydown('s', {}, el)
    expect(save).toHaveBeenCalledOnce()
    scope.dispose()
  })

  it('ignores keys while typing in INPUT by default', () => {
    const save = vi.fn()
    const el = document.createElement('div')
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, hotkeys, () => ({ s: save }))
    })
    keydown('s', {}, input)
    expect(save).not.toHaveBeenCalled()
    scope.dispose()
  })

  it('allowInInputs opts in for editable focus', () => {
    const save = vi.fn()
    const el = document.createElement('div')
    const input = document.createElement('input')
    el.appendChild(input)
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, hotkeys, () => ({
        s: { run: save, allowInInputs: true },
      }))
    })
    keydown('s', {}, input)
    expect(save).toHaveBeenCalledOnce()
    scope.dispose()
  })

  it('tears down listener on dispose', () => {
    const save = vi.fn()
    const el = document.createElement('div')
    document.body.appendChild(el)
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, hotkeys, () => ({ s: save }))
    })
    scope.dispose()
    keydown('s', {}, el)
    expect(save).not.toHaveBeenCalled()
  })
})
