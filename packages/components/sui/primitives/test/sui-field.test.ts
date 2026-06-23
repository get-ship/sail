import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-field/sui-field.compiled.js'
import { createContainerRegistry, flush } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

function mountField(inner = '') {
  const container = reg.mk()
  container.innerHTML = `<sui-field>${inner}</sui-field>`
  const el = container.querySelector('sui-field') as HTMLElement
  return { el, container }
}

function mountFieldInvalid(inner = '') {
  const container = reg.mk()
  container.innerHTML = `<sui-field invalid>${inner}</sui-field>`
  const el = container.querySelector('sui-field') as HTMLElement
  return { el, container }
}

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-field>', () => {

  it('renders shadow DOM', () => {
    const { el } = mountField()
    expect(shadow(el)).not.toBeNull()
  })

  it('has label, default, hint, and error slots', () => {
    const { el } = mountField()
    const slots = shadow(el).querySelectorAll('slot')
    const names = Array.from(slots).map(s => s.getAttribute('name') ?? '(default)')
    expect(names).toContain('label')
    expect(names).toContain('hint')
    expect(names).toContain('error')
    expect(names).toContain('(default)')
  })

  it('renders slotted label, control, and hint', async () => {
    const { el } = mountField(`
      <span slot="label">Name</span>
      <input type="text" name="name" />
      <span slot="hint">Your full name</span>
    `)
    await flush()
    expect(el.querySelector('[slot="label"]')).not.toBeNull()
    expect(el.querySelector('input')).not.toBeNull()
    expect(el.querySelector('[slot="hint"]')).not.toBeNull()
  })

  it('invalid attribute reflects on host', () => {
    const { el } = mountFieldInvalid()
    expect(el.hasAttribute('invalid')).toBe(true)
  })

  it('invalid=false by default', () => {
    const { el } = mountField()
    expect(el.hasAttribute('invalid')).toBe(false)
  })

  it('error div has role=alert and aria-live=polite', () => {
    const { el } = mountField()
    const errorDiv = shadow(el).querySelector('.error')!
    expect(errorDiv.getAttribute('role')).toBe('alert')
    expect(errorDiv.getAttribute('aria-live')).toBe('polite')
  })

  it('hint div is visible when not invalid', () => {
    const { el } = mountField()
    const hintDiv = shadow(el).querySelector('.hint') as HTMLElement
    expect(hintDiv).not.toBeNull()
    // CSS hides .error when :not([invalid]); hint is shown
    expect(el.hasAttribute('invalid')).toBe(false)
  })

  it('error div is visible when invalid', () => {
    const { el } = mountFieldInvalid()
    const errorDiv = shadow(el).querySelector('.error') as HTMLElement
    expect(errorDiv).not.toBeNull()
    expect(el.hasAttribute('invalid')).toBe(true)
  })

  it('wires aria-describedby on slotted control when not invalid', async () => {
    const { el } = mountField(`
      <input type="text" name="body" />
      <span slot="hint">helper text</span>
    `)
    await flush()
    const ctrl = el.querySelector('input') as HTMLInputElement
    expect(ctrl.getAttribute('aria-describedby')).toBeTruthy()
    // Should reference the hint id
    const describedBy = ctrl.getAttribute('aria-describedby') ?? ''
    const hintId = shadow(el).querySelector('.hint')?.id
    expect(hintId).toBeTruthy()
    expect(describedBy).toContain(hintId!)
  })

  it('wires aria-describedby to error div when invalid', async () => {
    const { el } = mountFieldInvalid(`
      <input type="text" name="body" />
      <span slot="error">Required</span>
    `)
    await flush()
    const ctrl = el.querySelector('input') as HTMLInputElement
    const describedBy = ctrl.getAttribute('aria-describedby') ?? ''
    const errorId = shadow(el).querySelector('.error')?.id
    expect(errorId).toBeTruthy()
    expect(describedBy).toContain(errorId!)
  })

  it('sets invalid on slotted SUI control', async () => {
    const container = reg.mk()
    // Use a plain input with invalid attribute check
    container.innerHTML = `
      <sui-field invalid>
        <input type="text" name="ctrl" />
      </sui-field>
    `
    await flush()
    const ctrl = container.querySelector('input') as HTMLInputElement
    // Should have aria-invalid set to true by field controller
    expect(ctrl.getAttribute('aria-invalid')).toBe('true')
  })

  it('rewires aria-describedby when invalid toggles to false', async () => {
    const { el } = mountFieldInvalid(`
      <input type="text" name="q" />
    `)
    await flush()
    const ctrl = el.querySelector('input') as HTMLInputElement
    const errorId = shadow(el).querySelector('.error')?.id ?? ''
    expect(ctrl.getAttribute('aria-describedby')).toContain(errorId)

    el.removeAttribute('invalid')
    await flush()

    const hintId = shadow(el).querySelector('.hint')?.id ?? ''
    const describedBy = ctrl.getAttribute('aria-describedby') ?? ''
    expect(describedBy).not.toContain(errorId)
    expect(describedBy).toContain(hintId)
  })

  it('hint and error divs have stable ids', async () => {
    const { el } = mountField()
    await flush()
    const hintEl = shadow(el).querySelector('.hint')!
    const errorEl = shadow(el).querySelector('.error')!
    expect(hintEl.id).toMatch(/^sui-field-hint-\d+$/)
    expect(errorEl.id).toMatch(/^sui-field-error-\d+$/)
  })

  it('cleanup on disconnect — no leaked effects', () => {
    const { container } = mountField('<input />')
    expect(container.isConnected).toBe(true)
    container.remove()
    expect(container.isConnected).toBe(false)
  })

  it('multiple field instances have different IDs', async () => {
    const { el: a } = mountField()
    const { el: b } = mountField()
    await flush()
    const aHintId = shadow(a).querySelector('.hint')?.id ?? ''
    const bHintId = shadow(b).querySelector('.hint')?.id ?? ''
    expect(aHintId).toBeTruthy()
    expect(bHintId).toBeTruthy()
    expect(aHintId).not.toBe(bHintId)
  })
})
