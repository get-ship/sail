import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { getByRole } from '@ship/sail-test'

const SOURCE = `<label>
  Bio
  <textarea placeholder="Tell us about yourself"></textarea>
</label>`

describe('l1 textarea rest', () => {
  it('mounts and is locatable by field name', () => {
    const v = renderRoute(SOURCE)
    expect(getByRole(v.root, 'textbox', { name: 'Bio' })).toBeTruthy()
    v.unmount()
  })

  it('stamps native rows from the host prop', () => {
    const v = renderRoute(`<label>Notes<textarea rows="5"></textarea></label>`)
    const el = getByRole(v.root, 'textbox', { name: 'Notes' }) as HTMLTextAreaElement
    expect(el.tagName).toBe('TEXTAREA')
    expect(el.getAttribute('rows')).toBe('5')
    expect(Number(el.rows)).toBe(5)
    v.unmount()
  })

  it('defaults to 3 native rows when omitted', () => {
    const v = renderRoute(`<label>Notes<textarea></textarea></label>`)
    const el = getByRole(v.root, 'textbox', { name: 'Notes' }) as HTMLTextAreaElement
    expect(el.getAttribute('rows')).toBe('3')
    expect(Number(el.rows)).toBe(3)
    v.unmount()
  })
})
