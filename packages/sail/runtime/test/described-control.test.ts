// describedControl — form control ↔ hint/error ARIA association.
//
// Capability mix-in the L1 <field> stamps as `use:describedControl`. Attached
// directly here (runAction lifecycle is covered elsewhere). MutationObserver
// callbacks are async in jsdom — attribute-flip assertions flush microtasks.
import { describe, expect, it, afterEach } from 'vitest'
import { describedControl } from '../src/actions/described-control'

function makeHost(opts: { invalid?: boolean; withControl?: boolean } = {}): {
  el: HTMLElement
  hint: HTMLElement
  error: HTMLElement
  control: HTMLElement
  input: HTMLInputElement | null
} {
  const el = document.createElement('div')
  el.setAttribute('data-sail-host', 'field')
  if (opts.invalid) el.setAttribute('invalid', '')

  // Structure mirrors field.sail: label, control, hint, error-gate → [data-field-error]
  // (role=alert + aria-live stamped by the action, not free author markup).
  const label = document.createElement('div')
  label.textContent = 'Email'

  const control = document.createElement('div')
  let input: HTMLInputElement | null = null
  if (opts.withControl !== false) {
    input = document.createElement('input')
    input.type = 'email'
    control.append(input)
  }

  const hint = document.createElement('div')
  hint.textContent = 'We will never share your email.'

  const errorGate = document.createElement('div')
  const error = document.createElement('div')
  error.setAttribute('data-field-error', '')
  error.textContent = 'Enter a valid email address.'
  errorGate.append(error)

  el.append(label, control, hint, errorGate)
  document.body.append(el)
  return { el, hint, error, control, input }
}

describe('describedControl (use: action)', () => {
  afterEach(() => {
    document.body.replaceChildren()
  })

  it('assigns stable ids to hint and error regions', () => {
    const { hint, error } = makeHost()
    describedControl(document.querySelector('[data-sail-host="field"]')!)
    expect(hint.id).toMatch(/^sail-field-hint-\d+$/)
    expect(error.id).toMatch(/^sail-field-error-\d+$/)
    expect(hint.id).not.toBe(error.id)
  })

  it('stamps role=alert + aria-live=polite on the error part (closed anatomy)', () => {
    const { el, error } = makeHost()
    describedControl(el)
    expect(error.getAttribute('role')).toBe('alert')
    expect(error.getAttribute('aria-live')).toBe('polite')
  })

  it('valid → aria-describedby points at hint; aria-invalid false', () => {
    const { el, hint, input } = makeHost()
    describedControl(el)
    expect(input).not.toBeNull()
    expect(input!.getAttribute('aria-describedby')).toBe(hint.id)
    expect(input!.getAttribute('aria-invalid')).toBe('false')
  })

  it('invalid → aria-describedby points at error; aria-invalid true', () => {
    const { el, error, input } = makeHost({ invalid: true })
    describedControl(el)
    expect(input).not.toBeNull()
    expect(input!.getAttribute('aria-describedby')).toBe(error.id)
    expect(input!.getAttribute('aria-invalid')).toBe('true')
  })

  it('toggle invalid rewires describedby and aria-invalid', async () => {
    const { el, hint, error, input } = makeHost()
    describedControl(el)
    expect(input!.getAttribute('aria-describedby')).toBe(hint.id)
    expect(input!.getAttribute('aria-invalid')).toBe('false')

    el.setAttribute('invalid', '')
    await Promise.resolve()
    expect(input!.getAttribute('aria-describedby')).toBe(error.id)
    expect(input!.getAttribute('aria-invalid')).toBe('true')

    el.removeAttribute('invalid')
    await Promise.resolve()
    expect(input!.getAttribute('aria-describedby')).toBe(hint.id)
    expect(input!.getAttribute('aria-invalid')).toBe('false')
  })

  // Pair to field fill=true: the control must be described by the live hint
  // (valid) / error (invalid) node. Dropping setAttribute('aria-describedby')
  // leaves getElementById(null) and this fails.
  it('describedby idref resolves to the live hint, then the error', async () => {
    const { el, hint, error, input } = makeHost()
    el.setAttribute('fill', '')
    describedControl(el)
    expect(input).not.toBeNull()

    const hintRef = input!.getAttribute('aria-describedby')
    expect(hintRef).toBeTruthy()
    expect(document.getElementById(hintRef!)).toBe(hint)
    expect(document.getElementById(hintRef!)!.textContent).toContain('never share')
    expect(input!.getAttribute('aria-invalid')).toBe('false')

    el.setAttribute('invalid', '')
    await Promise.resolve()
    const errorRef = input!.getAttribute('aria-describedby')
    expect(errorRef).toBeTruthy()
    expect(document.getElementById(errorRef!)).toBe(error)
    expect(document.getElementById(errorRef!)!.textContent).toContain('valid email')
    expect(input!.getAttribute('aria-invalid')).toBe('true')
  })

  it('late-mounted control still gets aria-describedby at the hint', async () => {
    const { el, hint, control } = makeHost({ withControl: false })
    describedControl(el)
    const input = document.createElement('input')
    input.type = 'email'
    control.append(input)
    await Promise.resolve()
    expect(input.getAttribute('aria-describedby')).toBe(hint.id)
    expect(document.getElementById(input.getAttribute('aria-describedby')!)).toBe(hint)
  })
})
