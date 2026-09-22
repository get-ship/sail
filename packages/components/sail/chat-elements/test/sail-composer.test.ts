// Behavioral suite for <sail-composer> — submit/keydown via onSubmit callback.
// AOT registration on import; the send action is a bare <button>.
// Product default is multiline grow (textarea + use:autosize).
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sail-composer/sail-composer.compiled.js'

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

const mount = async (markup = '<sail-composer></sail-composer>'): Promise<HTMLElement> => {
  const c = mk()
  c.innerHTML = markup
  const el = c.querySelector('sail-composer')!
  await flush()
  return el
}
// Default <textarea>; <input> when multiline={false}. Light DOM control.
const field = (el: HTMLElement): HTMLInputElement | HTMLTextAreaElement =>
  el.querySelector<HTMLInputElement | HTMLTextAreaElement>('input, textarea')!
const sendButton = (el: HTMLElement): HTMLElement =>
  el.querySelector<HTMLElement>('button[aria-label="Send"]')!

/** Capture host onSubmit (product callback; not a CustomEvent bus). */
function captureSubmit(el: HTMLElement): { detail: SubmitDetail | null } {
  const rec: { detail: SubmitDetail | null } = { detail: null }
  ;(el as HTMLElement & { onSubmit?: (d: SubmitDetail) => void }).onSubmit = (d) => {
    rec.detail = d
  }
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

describe('<sail-composer>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sail-composer')).toBeDefined()
  })

  it('defaults to a multiline textarea with use:autosize', async () => {
    const el = await mount()
    const ta = el.querySelector('textarea')
    expect(ta).not.toBeNull()
    expect(el.querySelector('input')).toBeNull()
    expect(ta!.hasAttribute('data-autosize')).toBe(true)
  })

  it('multiline={false} mounts a single-line input', async () => {
    const el = await mount('<sail-composer multiline="false"></sail-composer>')
    expect(el.querySelector('input')).not.toBeNull()
    expect(el.querySelector('textarea')).toBeNull()
  })

  it('invokes onSubmit with trimmed text on Enter, then clears the field', async () => {
    const el = await mount()
    const rec = captureSubmit(el)
    type(el, '  hello  ')
    await flush()
    pressEnter(el)
    expect(rec.detail).toEqual({ text: 'hello', attachments: [] })
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
    const el = await mount()
    const rec = captureSubmit(el)
    type(el, 'line one')
    await flush()
    pressEnter(el, { shift: true })
    expect(rec.detail).toBeNull()
    pressEnter(el)
    expect(rec.detail).toEqual({ text: 'line one', attachments: [] })
  })

  it('autosize publishes --autosize-h for multi-line content', async () => {
    const el = await mount()
    const ta = el.querySelector('textarea')!
    Object.defineProperty(ta, 'scrollHeight', {
      configurable: true,
      get: () => (ta.value.split('\n').length || 1) * 20,
    })
    type(el, 'line one\nline two\nline three')
    await flush()
    expect(ta.style.getPropertyValue('--autosize-h')).toBe('60px')
  })

  it('never submits while disabled', async () => {
    const el = await mount('<sail-composer disabled></sail-composer>')
    const rec = captureSubmit(el)
    type(el, 'blocked')
    await flush()
    pressEnter(el)
    expect(rec.detail).toBeNull()
  })

  it('echoes the consumer-owned attachments array back in onSubmit', async () => {
    const el = await mount()
    const attachments = [{ kind: 'file', name: 'a.png' }]
    ;(el as HTMLElement & { attachments: unknown[] }).attachments = attachments
    const rec = captureSubmit(el)
    type(el, 'with file')
    await flush()
    pressEnter(el)
    expect(rec.detail!.attachments).toEqual(attachments)
    expect(rec.detail!.attachments).not.toBe(attachments)
  })
})
