// Behavioral suite for <sui-composer> — submit/keydown event contracts.
// AOT registration on import; the send action is a bare <button>.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-composer/sui-composer.compiled.js'

const containers: HTMLElement[] = []
const mk = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})
const flush = () => new Promise((r) => setTimeout(r, 0))

type SubmitDetail = { text: string; attachments: unknown[] }

const mount = async (markup = '<sui-composer></sui-composer>'): Promise<HTMLElement> => {
  const c = mk()
  c.innerHTML = markup
  const el = c.querySelector('sui-composer')!
  await flush()
  return el
}
// The field is the composer's text control — an <input>, or a <textarea> in
// multiline mode. Light DOM: query the real control off the host.
const field = (el: HTMLElement): HTMLInputElement =>
  el.querySelector<HTMLInputElement>('input, textarea')!
const sendButton = (el: HTMLElement): HTMLElement =>
  el.querySelector<HTMLElement>('button[aria-label="Send"]')!

/** Drive the field as a user would: set the value + fire input so the
 *  controller's text signal tracks it, then return the captured submit
 *  detail (or null if no submit fired). */
function captureSubmit(el: HTMLElement): { detail: SubmitDetail | null } {
  const rec: { detail: SubmitDetail | null } = { detail: null }
  el.addEventListener('sui-composer-submit', (e) => {
    rec.detail = (e as CustomEvent).detail
  })
  return rec
}
function type(el: HTMLElement, value: string): void {
  const f = field(el)
  f.value = value
  f.dispatchEvent(new Event('input', { bubbles: true }))
}
function pressEnter(el: HTMLElement, opts: { shift?: boolean } = {}): void {
  field(el).dispatchEvent(
    new KeyboardEvent('keydown', { key: 'Enter', shiftKey: !!opts.shift, bubbles: true }),
  )
}

describe('<sui-composer>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-composer')).toBeDefined()
  })

  it('emits sui-composer-submit with trimmed text on Enter, then clears the field', async () => {
    const el = await mount()
    const rec = captureSubmit(el)
    type(el, '  hello  ')
    await flush()
    pressEnter(el)
    expect(rec.detail).toEqual({ text: 'hello', attachments: [] })
    // Field resets after a successful submit so the next turn starts clean.
    expect(field(el).value).toBe('')
  })

  it('does not submit empty / whitespace-only input', async () => {
    const el = await mount()
    const rec = captureSubmit(el)
    type(el, '   ')
    await flush()
    pressEnter(el)
    expect(rec.detail).toBeNull()
  })

  it('submits when the Send control is clicked', async () => {
    const el = await mount()
    const rec = captureSubmit(el)
    type(el, 'via button')
    await flush()
    sendButton(el).click()
    expect(rec.detail).toEqual({ text: 'via button', attachments: [] })
  })

  it('Shift+Enter in multiline mode inserts a newline instead of submitting', async () => {
    const el = await mount('<sui-composer multiline></sui-composer>')
    const rec = captureSubmit(el)
    type(el, 'line one')
    await flush()
    pressEnter(el, { shift: true })
    expect(rec.detail).toBeNull()
    // Plain Enter still submits in multiline mode.
    pressEnter(el)
    expect(rec.detail).toEqual({ text: 'line one', attachments: [] })
  })

  it('never submits while disabled', async () => {
    const el = await mount('<sui-composer disabled></sui-composer>')
    const rec = captureSubmit(el)
    type(el, 'blocked')
    await flush()
    pressEnter(el)
    expect(rec.detail).toBeNull()
  })

  it('echoes the consumer-owned attachments array back in the submit event', async () => {
    const el = await mount()
    const attachments = [{ kind: 'file', name: 'a.png' }]
    ;(el as HTMLElement & { attachments: unknown[] }).attachments = attachments
    const rec = captureSubmit(el)
    type(el, 'with file')
    await flush()
    pressEnter(el)
    expect(rec.detail!.attachments).toEqual(attachments)
    // The composer copies rather than aliasing the consumer's array.
    expect(rec.detail!.attachments).not.toBe(attachments)
  })

  it('the submit event reaches the document (bubbles + composed)', async () => {
    const el = await mount()
    let seenAtDocument = false
    document.addEventListener('sui-composer-submit', () => { seenAtDocument = true }, {
      once: true,
    })
    type(el, 'x')
    await flush()
    pressEnter(el)
    expect(seenAtDocument).toBe(true)
  })
})
