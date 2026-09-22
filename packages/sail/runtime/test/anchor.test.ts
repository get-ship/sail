import { describe, expect, it } from 'vitest'
import { anchor } from '../src/index'

describe('use:anchor', () => {
  it('does not write a geometry mailbox', () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const h = anchor(el, 'bottom') as { destroy?: () => void }
    expect(el.style.getPropertyValue('--anchor-top')).toBe('')
    expect(el.style.getPropertyValue('--anchor-left')).toBe('')
    h.destroy?.()
    el.remove()
  })
}
