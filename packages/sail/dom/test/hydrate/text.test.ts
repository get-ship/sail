// @vitest-environment jsdom

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { hydrateText } from '../../src/hydrate/text'

describe('hydrate/text', () => {
  it('collapses SSR text and reflects signal changes', () => {
    const root = document.createElement('p')
    root.innerHTML = 'count: <!--sail:text-->0<!--/sail-->'
    const open = root.childNodes[1] as Comment
    const sig = signal(0)
    const dispose = hydrateText(open, () => sig())
    expect(root.textContent).toBe('count: 0')
    sig.set(7)
    expect(root.textContent).toBe('count: 7')
    dispose()
    sig.set(99)
    expect(root.textContent).toBe('count: 7')
  })

  it('renders nullish as empty string', () => {
    const root = document.createElement('p')
    root.innerHTML = '<!--sail:text-->old<!--/sail-->'
    const open = root.firstChild as Comment
    const sig = signal<unknown>(null)
    hydrateText(open, () => sig())
    expect(root.textContent).toBe('')
    sig.set('hi')
    expect(root.textContent).toBe('hi')
  })
})
