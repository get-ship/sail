import { describe, expect, it } from 'vitest'
import { renderRoute } from '@ship/test-setup'
import { click, getByRole } from '@ship/sail-test'

function group(attrs = '') {
  return `<toggle-group${attrs} value="week">
  <toggle value="day">Day</toggle>
  <toggle value="week">Week</toggle>
  <toggle value="month">Month</toggle>
</toggle-group>`
}

function key(el: Element, key: string) {
  el.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }))
  return el
}

describe('l1 toggle-group', () => {
  it('is a roving group with the value pressed and one tabstop', async () => {
    const v = renderRoute(group())
    await v.flush()
    const day = getByRole(v.root, 'button', { name: 'Day' }) as HTMLButtonElement
    const week = getByRole(v.root, 'button', { name: 'Week' }) as HTMLButtonElement
    const month = getByRole(v.root, 'button', { name: 'Month' }) as HTMLButtonElement
    expect(day.getAttribute('aria-pressed')).toBe('false')
    expect(week.getAttribute('aria-pressed')).toBe('true')
    expect(month.getAttribute('aria-pressed')).toBe('false')
    expect(day.getAttribute('tabindex')).toBe('-1')
    expect(week.getAttribute('tabindex')).toBe('0')
    expect(month.getAttribute('tabindex')).toBe('-1')
    expect(document.activeElement).not.toBe(week)
    v.unmount()
  })

  it('single mode selects on click and deselects on the pressed one', async () => {
    const v = renderRoute(group())
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Month' }))
    await v.flush()
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('aria-pressed')).toBe('true')
    expect((getByRole(v.root, 'button', { name: 'Week' }) as HTMLElement).getAttribute('aria-pressed')).toBe('false')
    expect((getByRole(v.root, 'button', { name: 'Week' }) as HTMLElement).getAttribute('tabindex')).toBe('-1')
    click(getByRole(v.root, 'button', { name: 'Month' }))
    await v.flush()
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('aria-pressed')).toBe('false')
    v.unmount()
  })

  it('multiple keeps a space-separated set', async () => {
    const v = renderRoute(group(` type="multiple"`))
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Day' }))
    await v.flush()
    click(getByRole(v.root, 'button', { name: 'Month' }))
    await v.flush()
    expect((getByRole(v.root, 'button', { name: 'Day' }) as HTMLElement).getAttribute('aria-pressed')).toBe('true')
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('aria-pressed')).toBe('true')
    click(getByRole(v.root, 'button', { name: 'Day' }))
    await v.flush()
    expect((getByRole(v.root, 'button', { name: 'Day' }) as HTMLElement).getAttribute('aria-pressed')).toBe('false')
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('aria-pressed')).toBe('true')
    v.unmount()
  })

  it('arrows rove and select in single mode; Home and End work', async () => {
    const v = renderRoute(group())
    await v.flush()
    const day = getByRole(v.root, 'button', { name: 'Day' }) as HTMLButtonElement
    day.focus()
    key(day, 'ArrowRight')
    await v.flush()
    const week = getByRole(v.root, 'button', { name: 'Week' }) as HTMLButtonElement
    expect(document.activeElement).toBe(week)
    expect(week.getAttribute('aria-pressed')).toBe('true')
    expect(day.getAttribute('aria-pressed')).toBe('false')
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('tabindex')).toBe('-1')
    key(week, 'End')
    await v.flush()
    const month = getByRole(v.root, 'button', { name: 'Month' }) as HTMLButtonElement
    expect(document.activeElement).toBe(month)
    key(month, 'Home')
    await v.flush()
    expect(document.activeElement).toBe(day)
    expect(day.getAttribute('aria-pressed')).toBe('true')
    expect(month.getAttribute('aria-pressed')).toBe('false')
    v.unmount()
  })

  it('vertical roves with Up/Down', async () => {
    const v = renderRoute(group(` orientation="vertical"`))
    await v.flush()
    const month = getByRole(v.root, 'button', { name: 'Month' }) as HTMLButtonElement
    month.focus()
    key(month, 'ArrowUp')
    await v.flush()
    const week = getByRole(v.root, 'button', { name: 'Week' }) as HTMLButtonElement
    expect(document.activeElement).toBe(week)
    expect((getByRole(v.root, 'button', { name: 'Month' }) as HTMLElement).getAttribute('tabindex')).toBe('-1')
    v.unmount()
  })

  it('multiple arrows rove without selecting', async () => {
    const v = renderRoute(group(` type="multiple"`))
    await v.flush()
    const day = getByRole(v.root, 'button', { name: 'Day' }) as HTMLButtonElement
    day.focus()
    key(day, 'ArrowRight')
    await v.flush()
    const week = getByRole(v.root, 'button', { name: 'Week' }) as HTMLButtonElement
    expect(document.activeElement).toBe(week)
    expect(week.getAttribute('aria-pressed')).toBe('true')
    v.unmount()
  })

  it('disabled toggles are unpressed, unclickable and outside the rove', async () => {
    const v = renderRoute(`<toggle-group value="day">
  <toggle value="day">Day</toggle>
  <toggle value="week" disabled>Week</toggle>
  <toggle value="month">Month</toggle>
</toggle-group>`)
    await v.flush()
    const week = getByRole(v.root, 'button', { name: 'Week' }) as HTMLButtonElement
    expect(week.disabled).toBe(true)
    expect(week.getAttribute('tabindex')).toBe('-1')
    click(week)
    await v.flush()
    expect(week.getAttribute('aria-pressed')).toBe('false')
    expect((getByRole(v.root, 'button', { name: 'Day' }) as HTMLElement).getAttribute('aria-pressed')).toBe('true')
    v.unmount()
  })
})