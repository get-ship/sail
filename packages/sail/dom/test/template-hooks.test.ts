// End-to-end test for the `<use-kbd>` template hook through the
// production wasm-compiled path.
//
// The other v1 hooks (`<use-click-outside>`, `<use-focus-trap>`,
// `<use-clipboard>`, `<use-title>`, `<use-scroll-anchor>`,
// `<use-intersection>`) had wide compiler support in legacy hydrate
// but have no production callers yet.
// Their generic attr/bind/`as=` shape requires per-hook emit work
// in the Rust compiler that's out of scope for this gate-fail —
// hook coverage tracks production use, and only `<use-kbd>` lands
// here.

import { afterEach, describe, expect, it, vi } from 'vitest'
import { compileAndMount as hydrate } from '../src/test-utils/compile-sail'
import { controller } from '@ship/sail-runtime'

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})

const flush = () => new Promise((r) => setTimeout(r, 0))

describe('<use-kbd>', () => {
  it('routes document keydown to the action named on each <on>', async () => {
    const container = mkContainer()
    const onSearch = vi.fn()
    const onCreate = vi.fn()
    const def = controller(() => ({ search: onSearch, create: onCreate }))

    const template = `
      <use-kbd>
        <on key="/"      action="search" />
        <on key="cmd+n"  action="create" />
      </use-kbd>
      <p>x</p>
    `

    const handle = hydrate(template, def.setup, { container })
    await flush()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: '/' }))
    expect(onSearch).toHaveBeenCalledOnce()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', metaKey: true }))
    expect(onCreate).toHaveBeenCalledOnce()

    handle.dispose()
    // After dispose, document listener is removed.
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '/' }))
    expect(onSearch).toHaveBeenCalledOnce() // unchanged
  })

  it('ignores keys with no matching shortcut', async () => {
    const container = mkContainer()
    const onSearch = vi.fn()
    const def = controller(() => ({ search: onSearch }))
    const template = `<use-kbd><on key="/" action="search" /></use-kbd><p>x</p>`
    const handle = hydrate(template, def.setup, { container })
    await flush()

    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'a' }))
    expect(onSearch).not.toHaveBeenCalled()
    handle.dispose()
  })
})
