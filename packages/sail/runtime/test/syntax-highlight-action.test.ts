import { describe, expect, it } from 'vitest'
import { runAction } from '../src/action'
import { syntaxHighlight } from '../src/actions/syntax-highlight'
import { createScope, withScope } from '../src/effect-scope'
import { signal } from '../src/signal'

describe('syntaxHighlight action', () => {
  it('highlights from use:syntaxHighlight={code} param', () => {
    const el = document.createElement('div')
    const pre = document.createElement('pre')
    el.appendChild(pre)
    const code = signal('const x = 1')
    const scope = createScope()
    withScope(scope, () => {
      runAction(el, syntaxHighlight, () => code())
    })
    expect(pre.innerHTML).toContain('tok-')
    expect(pre.textContent).toContain('const')
    expect(pre.textContent).toContain('x')
    code.set('let y = 2')
    expect(pre.textContent).toContain('y')
    scope.dispose()
  })

  it('resolves code from host CE when action is on an inner node', () => {
    const host = document.createElement('div')
    host.setAttribute('data-sail-host', 'code-block')
    host.className = 'sail-code-block'
    ;(host as HTMLElement & { code?: string }).code = 'signal(0)'
    host.setAttribute('code', 'signal(0)')

    const inner = document.createElement('div')
    const pre = document.createElement('pre')
    inner.appendChild(pre)
    host.appendChild(inner)

    const scope = createScope()
    withScope(scope, () => {
      runAction(inner, syntaxHighlight)
    })
    expect(pre.innerHTML).toContain('tok-')
    expect(pre.textContent).toContain('signal')
    scope.dispose()
  })

  it('does not dump empty when host code is present and param is empty', () => {
    const host = document.createElement('div')
    host.className = 'sail-code-block'
    host.setAttribute('code', 'export default 1')
    const inner = document.createElement('div')
    const pre = document.createElement('pre')
    inner.appendChild(pre)
    host.appendChild(inner)
    document.body.appendChild(host)

    const scope = createScope()
    withScope(scope, () => {
      runAction(inner, syntaxHighlight, () => '')
    })
    expect(pre.textContent).toContain('export')
    scope.dispose()
    host.remove()
  })

  it('re-highlights when bindProp sets host.code after the action mounts', async () => {
    const host = document.createElement('div')
    host.className = 'sail-code-block'
    host.setAttribute('data-sail-host', 'code-block')
    const inner = document.createElement('div')
    const pre = document.createElement('pre')
    inner.appendChild(pre)
    host.appendChild(inner)
    document.body.appendChild(host)

    const scope = createScope()
    withScope(scope, () => {
      runAction(inner, syntaxHighlight)
    })
    // Empty first paint — then bindProp-style identity write (no attribute).
    expect(pre.textContent ?? '').toBe('')
    ;(host as HTMLElement & { code?: string }).code = 'const later = true'
    // Property interceptor + microtask/rAF
    await new Promise((r) => queueMicrotask(() => r(undefined)))
    expect(pre.innerHTML).toContain('tok-')
    expect(pre.textContent).toContain('later')
    scope.dispose()
    host.remove()
  })
})
