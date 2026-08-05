import { describe, expect, it, afterEach } from 'vitest'
import {
  getByRole,
  queryByRole,
  getAllByRole,
  getByText,
  accessibleName,
  getRole,
  click,
} from '../src/queries'

afterEach(() => {
  document.body.innerHTML = ''
})

function mount(html: string): HTMLElement {
  const root = document.createElement('div')
  root.innerHTML = html
  document.body.appendChild(root)
  return root
}

describe('getByRole — ARIA-1 stamps', () => {
  it('finds native button by role + name', () => {
    const root = mount(`<button type="button">Save draft</button>`)
    const btn = getByRole(root, 'button', { name: 'Save draft' })
    expect(btn.tagName).toBe('BUTTON')
    expect(accessibleName(btn)).toBe('Save draft')
  })

  it('finds aria-label name (compiler label prop stamp)', () => {
    const root = mount(`<button type="button" aria-label="Close dialog"></button>`)
    expect(getByRole(root, 'button', { name: /close dialog/i })).toBeTruthy()
  })

  it('matches pressed / expanded / selected state', () => {
    const root = mount(`
      <button type="button" aria-pressed="true">Bold</button>
      <button type="button" aria-expanded="true">Menu</button>
      <div role="option" aria-selected="true">A</div>
    `)
    expect(getByRole(root, 'button', { name: 'Bold', pressed: true }).textContent).toBe('Bold')
    expect(getByRole(root, 'button', { name: 'Menu', expanded: true }).textContent).toBe('Menu')
    expect(getByRole(root, 'option', { selected: true }).textContent).toBe('A')
  })

  it('matches host anatomy roles (navigation, status, tablist)', () => {
    const root = mount(`
      <nav aria-label="Primary">nav</nav>
      <div role="status" aria-live="polite">Saved</div>
      <div role="tablist"><button role="tab">One</button></div>
    `)
    expect(getByRole(root, 'navigation', { name: 'Primary' }).tagName).toBe('NAV')
    expect(getByRole(root, 'status').textContent).toContain('Saved')
    expect(getByRole(root, 'tablist')).toBeTruthy()
    expect(getByRole(root, 'tab', { name: 'One' })).toBeTruthy()
  })

  it('heading level via native tag + aria-level', () => {
    const root = mount(`
      <h1>Title</h1>
      <div role="heading" aria-level="2">Sub</div>
    `)
    expect(getByRole(root, 'heading', { name: 'Title', level: 1 }).tagName).toBe('H1')
    expect(getByRole(root, 'heading', { name: 'Sub', level: 2 }).getAttribute('aria-level')).toBe(
      '2',
    )
  })

  it('queryByRole returns null; getAllByRole lists', () => {
    const root = mount(`
      <button type="button">A</button>
      <button type="button">B</button>
    `)
    expect(queryByRole(root, 'link')).toBeNull()
    expect(getAllByRole(root, 'button')).toHaveLength(2)
  })

  it('click helper fires on role target', () => {
    const root = mount(`<button type="button">Go</button>`)
    let n = 0
    getByRole(root, 'button', { name: 'Go' }).addEventListener('click', () => {
      n++
    })
    click(getByRole(root, 'button', { name: 'Go' }))
    expect(n).toBe(1)
  })

  it('name exact defaults true; exact:false allows substring', () => {
    const root = mount(`<button type="button">Save draft now</button>`)
    expect(queryByRole(root, 'button', { name: 'Save draft' })).toBeNull()
    expect(getByRole(root, 'button', { name: 'Save draft', exact: false }).textContent).toBe(
      'Save draft now',
    )
    expect(getByRole(root, 'button', { name: 'Save draft now' }).textContent).toBe(
      'Save draft now',
    )
    expect(getByRole(root, 'button', { name: /Save draft/ }).textContent).toBe('Save draft now')
  })
})

describe('getByText', () => {
  it('finds visible text without structure selectors', () => {
    const root = mount(`<p><span>Hello fleet</span></p>`)
    expect(getByText(root, 'Hello fleet').textContent).toContain('Hello fleet')
  })
})

describe('getRole', () => {
  it('implicit roles for common tags', () => {
    const root = mount(`
      <a href="/x">link</a>
      <input type="checkbox" />
      <input type="text" />
    `)
    expect(getRole(root.querySelector('a')!)).toBe('link')
    expect(getRole(root.querySelector('input[type=checkbox]')!)).toBe('checkbox')
    expect(getRole(root.querySelector('input[type=text]')!)).toBe('textbox')
  })
})
