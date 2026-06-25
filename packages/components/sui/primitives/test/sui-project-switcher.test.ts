import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-project-switcher/sui-project-switcher.compiled.js'
import '../src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'
import type { SwitcherProject } from '../src/sui-project-switcher/sui-project-switcher.controller'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

type Switcher = HTMLElement & { projects: SwitcherProject[] }

const shadow = (el: HTMLElement) => el.shadowRoot!
const rows = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('button.row')) as HTMLButtonElement[]
const rowOpenBtns = (el: HTMLElement) =>
  Array.from(shadow(el).querySelectorAll('button.row-open')) as HTMLButtonElement[]
const triggerName = (el: HTMLElement) => shadow(el).querySelector('.trigger .name')?.textContent

const PROJECTS: SwitcherProject[] = [
  { id: 'alpha', name: 'Alpha', path: '/work/alpha', hue: 'blue' },
  { id: 'beta', name: 'Beta', path: '/work/beta' }, // no hue → derived
]

async function mountWith(active = 'alpha'): Promise<Switcher> {
  const { el } = mount(reg, 'sui-project-switcher', { 'active-id': active })
  await flush()
  ;(el as Switcher).projects = PROJECTS
  await flush()
  return el as Switcher
}

describe('<sui-project-switcher> — rendering', () => {
  it('registers; empty list shows the placeholder trigger + empty menu', async () => {
    const { el } = mount(reg, 'sui-project-switcher')
    await flush()
    expect(customElements.get('sui-project-switcher')).toBeDefined()
    expect(triggerName(el)).toBe('Project')
    expect(shadow(el).querySelector('.menu-empty')).not.toBeNull()
  })

  it('renders one row per project with name + path; trigger shows the active name', async () => {
    const el = await mountWith('alpha')
    expect(rows(el).map((r) => r.querySelector('.row-name')?.textContent)).toEqual(['Alpha', 'Beta'])
    expect(shadow(el).querySelector('.row[data-active] .row-name')?.textContent).toBe('Alpha')
    expect(triggerName(el)).toBe('Alpha')
  })

  it('marks the active row aria-checked; non-active rows are unchecked', async () => {
    const el = await mountWith('beta')
    const checked = rows(el).map((r) => r.getAttribute('aria-checked'))
    expect(checked).toEqual(['false', 'true'])
  })

  it('uses an explicit hue, deriving a stable one when omitted', async () => {
    const el = await mountWith('alpha')
    const dots = rows(el).map((r) => r.querySelector('.dot')?.getAttribute('data-hue'))
    expect(dots[0]).toBe('blue') // explicit
    expect(dots[1]).toBeTruthy() // derived from id — present and stable
    expect(dots[1]).not.toBe('') // a real palette token, not empty
  })

  it('parses the projects="…JSON…" attribute (SSR fallback)', async () => {
    const json = JSON.stringify([{ id: 'x', name: 'Xeon', path: '/x' }])
    const { el } = mount(reg, 'sui-project-switcher', { 'active-id': 'x', projects: json })
    await flush()
    expect(rows(el).map((r) => r.querySelector('.row-name')?.textContent)).toEqual(['Xeon'])
  })
})

describe('<sui-project-switcher> — selection events (no SDK/router)', () => {
  it('picking a non-active project emits sui-project-select {id}', async () => {
    const el = await mountWith('alpha')
    let detail: { id: string } | null = null
    el.addEventListener('sui-project-select', (e) => {
      detail = (e as CustomEvent).detail
    })
    rows(el)[1]!.click() // Beta
    await flush()
    expect(detail).toEqual({ id: 'beta' })
  })

  it('picking the already-active project does NOT emit', async () => {
    const el = await mountWith('alpha')
    let fired = 0
    el.addEventListener('sui-project-select', () => fired++)
    rows(el)[0]!.click() // Alpha — already active
    await flush()
    expect(fired).toBe(0)
  })

  it('the open-in-window affordance emits sui-project-open {id}', async () => {
    const el = await mountWith('alpha')
    let detail: { id: string } | null = null
    el.addEventListener('sui-project-open', (e) => {
      detail = (e as CustomEvent).detail
    })
    rowOpenBtns(el)[1]!.click() // open Beta in a window
    await flush()
    expect(detail).toEqual({ id: 'beta' })
  })

  it('emits sui-project-switcher-ready once after mount', async () => {
    const c = reg.mk()
    let ready = 0
    c.addEventListener('sui-project-switcher-ready', () => ready++)
    c.innerHTML = '<sui-project-switcher></sui-project-switcher>'
    await flush()
    expect(ready).toBe(1)
  })
})
