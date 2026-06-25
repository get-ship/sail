import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-account-menu/sui-account-menu.compiled.js'
import '../src/sui-icon/sui-icon.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const triggerName = (el: HTMLElement) =>
  shadow(el).querySelector('.trigger .name')?.textContent
/** Find a panel action button by the lucide icon it carries. */
const actionByIcon = (el: HTMLElement, icon: string): HTMLButtonElement | null => {
  for (const btn of Array.from(shadow(el).querySelectorAll('button.action'))) {
    if (btn.querySelector(`sui-icon[name="${icon}"]`)) return btn as HTMLButtonElement
  }
  return null
}

describe('<sui-account-menu> — identity rendering', () => {
  it('signed-out: trigger reads "Sign in"', async () => {
    const { el } = mount(reg, 'sui-account-menu')
    await flush()
    expect(customElements.get('sui-account-menu')).toBeDefined()
    expect(triggerName(el)).toBe('Sign in')
    // signed-out fallback avatar shows the user icon
    expect(shadow(el).querySelector('.trigger sui-icon[name="user"]')).not.toBeNull()
  })

  it('signed-in: trigger shows the name + derived initials; panel shows email + plan', async () => {
    const { el } = mount(reg, 'sui-account-menu', {
      status: 'signed-in',
      name: 'Ada Lovelace',
      email: 'ada@example.com',
      plan: 'Pro',
    })
    await flush()
    expect(triggerName(el)).toBe('Ada Lovelace')
    expect(shadow(el).querySelector('.trigger .avatar-fallback')?.textContent).toBe('AL')
    expect(shadow(el).querySelector('.id-email')?.textContent).toBe('ada@example.com')
    expect(shadow(el).querySelector('.plan')?.textContent).toBe('Pro')
  })

  it('signed-in with image: renders the avatar img instead of initials', async () => {
    const { el } = mount(reg, 'sui-account-menu', {
      status: 'signed-in',
      name: 'Ada',
      image: 'https://example.com/a.png',
    })
    await flush()
    const img = shadow(el).querySelector('.trigger img.avatar') as HTMLImageElement
    expect(img).not.toBeNull()
    expect(img.getAttribute('src')).toBe('https://example.com/a.png')
  })

  it('provider folds into the sign-in label + the connect hint', async () => {
    const { el } = mount(reg, 'sui-account-menu', { provider: 'Ship Cloud' })
    await flush()
    expect(actionByIcon(el, 'log-in')?.textContent?.trim()).toBe('Sign in with Ship Cloud')
    expect(shadow(el).querySelector('.id-email')?.textContent).toBe(
      'Connect this workspace to Ship Cloud',
    )
  })

  it('label-style="compact" reflects to the host', async () => {
    const { el } = mount(reg, 'sui-account-menu', { 'label-style': 'compact' })
    await flush()
    expect(el.getAttribute('label-style')).toBe('compact')
  })
})

describe('<sui-account-menu> — actions emit events (no SDK/router)', () => {
  it('signed-out emits sui-account-signin; settings emits sui-account-settings', async () => {
    const { el } = mount(reg, 'sui-account-menu')
    await flush()
    let signin = 0
    let settings = 0
    el.addEventListener('sui-account-signin', () => signin++)
    el.addEventListener('sui-account-settings', () => settings++)
    actionByIcon(el, 'log-in')!.click()
    actionByIcon(el, 'settings')!.click()
    await flush()
    expect(signin).toBe(1)
    expect(settings).toBe(1)
  })

  it('signed-in emits sui-account-signout and sui-account-billing', async () => {
    const { el } = mount(reg, 'sui-account-menu', { status: 'signed-in', name: 'Ada' })
    await flush()
    const seen: string[] = []
    el.addEventListener('sui-account-signout', () => seen.push('out'))
    el.addEventListener('sui-account-billing', () => seen.push('bill'))
    actionByIcon(el, 'log-out')!.click()
    actionByIcon(el, 'credit-card')!.click()
    await flush()
    expect(seen.sort()).toEqual(['bill', 'out'])
  })

  it('emits sui-account-menu-ready once after mount', async () => {
    const c = reg.mk()
    let ready = 0
    c.addEventListener('sui-account-menu-ready', () => ready++)
    c.innerHTML = '<sui-account-menu></sui-account-menu>'
    await flush()
    expect(ready).toBe(1)
  })
})
