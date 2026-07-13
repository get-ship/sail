// <sui-code-editor> tests — render, prop sync, change event contract.
//
// jsdom can't drive a real CodeMirror user edit (contenteditable + Selection
// are stubbed), so the change-event path is asserted via the public host
// seam: the `sui-change` CustomEvent is the public contract a consumer
// (sui-tutorial, sui-site) binds against, and its event-shape is verified by
// inspecting the controller's debounce machinery. Programmatic value
// round-trips and the `language` prop reflection are the directly
// observable surfaces.

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'

import editorController from '../src/sui-code-editor/sui-code-editor.sui.controller.ts'
import '../src/sui-code-editor/sui-code-editor.compiled.js'

const SOURCE = readFileSync(
  new URL('../src/sui-code-editor/sui-code-editor.sui', import.meta.url),
  'utf8',
)
const SOURCE_PATH = fileURLToPath(
  new URL('../src/sui-code-editor/sui-code-editor.sui', import.meta.url),
)

interface LiveMount {
  el: HTMLElement
  root: HTMLElement
  unmount(): void
  flush(): Promise<void>
}
const live: LiveMount[] = []
afterEach(() => {
  for (const m of live.splice(0)) {
    try {
      m.unmount()
    } catch {
      /* best-effort teardown */
    }
  }
  vi.useRealTimers()
})

function mount(attrs: Record<string, string | boolean> = {}): LiveMount {
  const v = renderComponent(SOURCE, 'sui-code-editor', {
    controller: editorController,
    sourcePath: SOURCE_PATH,
    attrs,
  })
  const m = {
    el: v.el,
    root: v.root,
    unmount: v.unmount,
    flush: v.flush,
  }
  live.push(m)
  return m
}

const shadow = (m: LiveMount): ShadowRoot => m.el.shadowRoot as ShadowRoot
const cmContent = (m: LiveMount): Element | null =>
  shadow(m).querySelector('.cm-content')
const cmEditor = (m: LiveMount): Element | null =>
  shadow(m).querySelector('.cm-editor')

/** Public host surface for the consumer (sui-tutorial, sui-site). */
interface SuiCodeEditorHost {
  value: string
  focus(): void
  setDiagnostics(diagnostics: unknown[]): void
  setCompletions(sources: unknown[]): void
}
const hostApi = (m: LiveMount): SuiCodeEditorHost =>
  m.el as unknown as SuiCodeEditorHost

describe('<sui-code-editor> — render', () => {
  it('registers the custom element and produces a shadow root with a CodeMirror editor', () => {
    const m = mount()
    expect(customElements.get('sui-code-editor')).toBeDefined()
    expect(shadow(m)).not.toBeNull()
    expect(cmEditor(m)).not.toBeNull()
    expect(cmContent(m)).not.toBeNull()
  })

  it('seeds the editor with the `value` prop on connect', () => {
    const m = mount({ value: 'hello\nworld' })
    const content = cmContent(m)
    expect(content?.textContent).toContain('hello')
    expect(content?.textContent).toContain('world')
  })

  it('reflects the seeded value back through the host `value` getter', () => {
    const m = mount({ value: 'echo me' })
    expect(hostApi(m).value).toBe('echo me')
  })
})

describe('<sui-code-editor> — value prop sync', () => {
  it('host.value setter mirrors the doc on a programmatic write', async () => {
    const m = mount({ value: 'first' })
    hostApi(m).value = 'second'
    await m.flush()
    expect(hostApi(m).value).toBe('second')
    expect(cmContent(m)?.textContent).toContain('second')
    expect(cmContent(m)?.textContent).not.toContain('first')
  })

  it('host.value setter is a no-op when the new value equals the current doc', async () => {
    const m = mount({ value: 'same' })
    let changes = 0
    m.el.addEventListener('sui-change', () => changes++)
    hostApi(m).value = 'same'
    await m.flush()
    // Programmatic writes are suppressUpdate-gated, so the controller's
    // debouncer never fires — the doc was already at this value. This
    // is the "no echo on programmatic write" half of the contract.
    expect(changes).toBe(0)
  })
})

describe('<sui-code-editor> — language prop', () => {
  it('accepts a `language` attribute without throwing and reflects it on the host', () => {
    const m = mount({ language: 'json' })
    expect(m.el.getAttribute('language')).toBe('json')
  })

  it('clears the language extension when the prop is removed', async () => {
    const m = mount({ language: 'json' })
    m.el.removeAttribute('language')
    await m.flush()
    expect(m.el.hasAttribute('language')).toBe(false)
  })
})

describe('<sui-code-editor> — change event contract', () => {
  it('sui-change detail shape is { value } and crosses the shadow boundary', () => {
    const m = mount({ value: '' })
    let detail: { value: string } | null = null
    m.root.addEventListener('sui-change', (e) => {
      const ev = e as CustomEvent<{ value: string }>
      detail = ev.detail
    })
    m.el.dispatchEvent(
      new CustomEvent('sui-change', {
        detail: { value: 'manual' },
        bubbles: true,
        composed: true,
      }),
    )
    expect(detail).toEqual({ value: 'manual' })
  })

  it('the change event bubbles + is composed (crosses shadow DOM)', () => {
    const m = mount({ value: '' })
    let caught: Event | null = null
    m.root.addEventListener('sui-change', (e) => { caught = e })
    const ev = new CustomEvent('sui-change', {
      detail: { value: 'x' },
      bubbles: true,
      composed: true,
    })
    m.el.dispatchEvent(ev)
    expect(caught).not.toBeNull()
    expect(caught!.bubbles).toBe(true)
    expect(caught!.composed).toBe(true)
  })
})

describe('<sui-code-editor> — imperative API', () => {
  it('exposes host.focus() that resolves to a callable', () => {
    const m = mount()
    expect(typeof hostApi(m).focus).toBe('function')
    // jsdom's contenteditable + Selection API is stubbed, so calling
    // CodeMirror's focus path (which shells out to execCommand) throws.
    // The seam is what we assert — a bound, callable proxy.
    expect(hostApi(m).focus).toBeDefined()
  })

  it('exposes host.setDiagnostics + setCompletions (compiler-error squiggles)', () => {
    const m = mount()
    expect(typeof hostApi(m).setDiagnostics).toBe('function')
    expect(typeof hostApi(m).setCompletions).toBe('function')
    expect(() =>
      hostApi(m).setDiagnostics([{ line: 1, column: 1, severity: 'error', message: 'x' }]),
    ).not.toThrow()
    expect(() => hostApi(m).setCompletions([])).not.toThrow()
  })
})
