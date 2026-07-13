// Behavioral suite for <sui-tool-call>. AOT registration on import; the
// disclosure composes <sui-collapsible>, so import that too.
import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-tool-call/sui-tool-call.compiled.js'

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

const mount = (markup: string): HTMLElement => {
  const c = mk()
  c.innerHTML = markup
  return c.querySelector('sui-tool-call')!
}
// Light DOM, no class hooks. The disclosure is a native <details>/<summary>;
// the header's two announced texts are the tool name then the status (the dot
// and the chevron are aria-hidden decoration).
const headerText = (el: HTMLElement): HTMLElement[] =>
  Array.from(
    el.querySelectorAll<HTMLElement>('summary span:not([aria-hidden="true"])'),
  )
const nameEl = (el: HTMLElement): HTMLElement => headerText(el)[0]!
const statusEl = (el: HTMLElement): HTMLElement => headerText(el)[1]!

describe('<sui-tool-call>', () => {
  it('registers with the global custom-element registry', () => {
    expect(customElements.get('sui-tool-call')).toBeDefined()
  })

  it('renders the tool name + status text in the header', () => {
    const el = mount(
      '<sui-tool-call name="read_file" status="running">{"path":"x"}</sui-tool-call>',
    )
    expect(nameEl(el).textContent).toBe('read_file')
    expect(statusEl(el).textContent).toBe('running')
  })

  it('tracks a mid-call resolution running→ok through host attr + header text', async () => {
    const el = mount('<sui-tool-call name="grep" status="running"></sui-tool-call>')
    expect(el.getAttribute('status')).toBe('running')
    expect(statusEl(el).textContent).toBe('running')
    // Consumer flips the attribute as the call resolves; the header follows.
    el.setAttribute('status', 'ok')
    await flush()
    expect(el.getAttribute('status')).toBe('ok')
    expect(statusEl(el).textContent).toBe('ok')
  })

  it('labels + projects args (default slot) and result (result slot) into the body', () => {
    const el = mount(
      '<sui-tool-call name="run" open>' +
        '<span class="a">--flag</span>' +
        '<span slot="result">done</span>' +
        '</sui-tool-call>',
    )
    const body = el.querySelector<HTMLElement>('details')!
    expect(body.textContent).toContain('Arguments')
    expect(body.textContent).toContain('Result')
    // Light-DOM nodes slot into the disclosure body by slot contract.
    const argSlot = el.querySelector<HTMLSlotElement>('details slot:not([name])')!
    const resultSlot = el.querySelector<HTMLSlotElement>('details slot[name="result"]')!
    expect(argSlot).not.toBeNull()
    expect(resultSlot).not.toBeNull()
    expect(el.querySelector('[slot="result"]')!.textContent).toBe('done')
  })

  it('reflects open so the disclosure can start expanded', () => {
    const el = mount('<sui-tool-call name="x" open></sui-tool-call>')
    expect(el.hasAttribute('open')).toBe(true)
  })
})
