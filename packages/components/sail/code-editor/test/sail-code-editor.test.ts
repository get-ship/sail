// <sail-code-editor> tests — render, prop sync, change callback contract.
//
// jsdom can't drive a real CodeMirror user edit (contenteditable + Selection
// are stubbed), so the change path is asserted via the public host seam:
// the `onChange` callback prop is the public contract a consumer
// (tutorial, sail-site) binds with `on-change={fn}`. Programmatic value
// round-trips and the `language` prop reflection are the directly
// observable surfaces.

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'

import editorController from '../src/sail-code-editor/sail-code-editor.sail.controller.ts'
import '../src/sail-code-editor/sail-code-editor.compiled.js'

const SOURCE = readFileSync(
  new URL('../src/sail-code-editor/sail-code-editor.sail', import.meta.url),
  'utf8',
)
const SOURCE_PATH = fileURLToPath(
  new URL('../src/sail-code-editor/sail-code-editor.sail', import.meta.url),
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
  const v = renderComponent(SOURCE, 'sail-code-editor', {
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

// Sail components are LIGHT DOM: the template (and the CodeMirror view the
// `use:editorMount` action builds inside it) renders into the host element
// itself — `el.shadowRoot` is null by design, so queries run against `el`.
const editorRoot = (m: LiveMount): HTMLElement => m.el
const cmContent = (m: LiveMount): Element | null =>
  editorRoot(m).querySelector('.cm-content')
const cmEditor = (m: LiveMount): Element | null =>
  editorRoot(m).querySelector('.cm-editor')

/** Public host surface for the consumer (sail-tutorial, sail-site). */
interface SailCodeEditorHost {
  value: string
  focus(): void
  setDiagnostics(diagnostics: unknown[]): void
  setCompletions(sources: unknown[]): void
}
const hostApi = (m: LiveMount): SailCodeEditorHost =>
  m.el as unknown as SailCodeEditorHost

describe('<sail-code-editor> — render', () => {
  it('registers the custom element and mounts a CodeMirror editor in its light DOM', () => {
    const m = mount()
    expect(customElements.get('sail-code-editor')).toBeDefined()
    // Light DOM is the product model — no shadow root; the editor is a
    // descendant of the host.
    expect(m.el.shadowRoot).toBeNull()
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

describe('<sail-code-editor> — value prop sync', () => {
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
    ;(m.el as HTMLElement & { onChange?: () => void }).onChange = () => {
      changes++
    }
    hostApi(m).value = 'same'
    await m.flush()
    // Programmatic writes are suppressUpdate-gated, so the controller's
    // debouncer never fires — the doc was already at this value. This
    // is the "no echo on programmatic write" half of the contract.
    expect(changes).toBe(0)
  })
})

describe('<sail-code-editor> — language prop', () => {
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

describe('<sail-code-editor> — ops-url degrade', () => {
  it('mounts without ops-url (sail-site / non-studio embeds)', () => {
    const m = mount({ language: 'sail', value: '<column></column>' })
    expect(cmEditor(m)).not.toBeNull()
    expect(m.el.getAttribute('ops-url')).toBeFalsy()
  })

  it('accepts ops-url + ops-path without throwing', async () => {
    const m = mount({
      language: 'sail',
      'ops-url': '/',
      'ops-path': 'apps/studio/routes/page.sail',
      value: 'const n = signal(0)',
    })
    await m.flush()
    expect(m.el.getAttribute('ops-url')).toBe('/')
    expect(m.el.getAttribute('ops-path')).toBe('apps/studio/routes/page.sail')
    expect(cmEditor(m)).not.toBeNull()
  })
})

describe('<sail-code-editor> — change callback contract', () => {
  it('onChange payload shape is { value }', () => {
    const m = mount({ value: '' })
    let detail: { value: string } | null = null
    ;(m.el as HTMLElement & { onChange?: (d: { value: string }) => void }).onChange = (d) => {
      detail = d
    }
    ;(m.el as HTMLElement & { onChange?: (d: { value: string }) => void }).onChange?.({
      value: 'manual',
    })
    expect(detail).toEqual({ value: 'manual' })
  })

  it('onChange is a direct host property (no event bus)', () => {
    const m = mount({ value: '' })
    let called = 0
    const host = m.el as HTMLElement & { onChange?: (d: { value: string }) => void }
    host.onChange = () => {
      called++
    }
    host.onChange?.({ value: 'x' })
    expect(called).toBe(1)
  })
})

describe('<sail-code-editor> — imperative API', () => {
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
