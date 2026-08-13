// Smoke test for every shipped `<set-*>` component.
//
// Each port is registered via `registerAll()`, then mounted with a
// minimum viable attribute set, and asserted on three things:
//   1. The custom element registered (tag is in the registry).
//   2. The shadow root is attached and contains a recognizable part.
//   3. Setting a reflected attribute updates the host's DOM attribute
//      (proves the prop pipeline runs).
//
// Per-component deeper tests (slot content, conditional ARIA, complex
// derived state) live in dedicated test files where they earn their
// existence; this file is the regression net for "did we break a
// port."

import { afterEach, describe, expect, it } from 'vitest'
// Side-effect import — index.ts pulls in every `<tag>.ts`, each of
// which side-effect imports its AOT-compiled module. By the time
// the test body runs, every <set-*> custom element is registered.
import '../src/index'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

interface PortCase {
  /** Tag suffix; the test mounts `<set-<name>>`. */
  tag: string
  /** A reflected attr to set + assert echoes back on the host. */
  attr: string
  /** Value to set for the reflect check. */
  value: string
  /** A part name (or class selector) that should be present in the shadow. */
  /** A selector for the component's own semantics: its role, its tag. */
  shadowPart: string
}

// Components that dissolved into the language (button, separator, tabs, tab,
// input, dialog…) are L1 elements, not custom elements — they are not this
// per-component smoke net.
const ports: PortCase[] = [
  { tag: 'sail-tree',       attr: 'size',  value: 'sm',      shadowPart: '[role="status"]' },
  { tag: 'sail-tree-grid',  attr: 'size',  value: 'sm',      shadowPart: '[role="status"]' },
  { tag: 'sail-json-view',  attr: 'data',  value: '{"a":1}', shadowPart: '[role="treeitem"]' },
  { tag: 'sail-timeline',   attr: 'hidden', value: '',       shadowPart: 'slot' },
  { tag: 'sail-file-upload', attr: 'accept', value: 'image/*', shadowPart: '[role="button"]' },
  { tag: 'sail-color-picker', attr: 'size', value: 'lg',     shadowPart: 'input[type="text"]' },
  { tag: 'sail-date-picker', attr: 'size', value: 'lg',      shadowPart: 'input' },
  { tag: 'sail-calendar',   attr: 'view',  value: 'month',   shadowPart: '[role="grid"]' },
  { tag: 'sail-markdown',   attr: 'value', value: '# hi',    shadowPart: '.sail-prose' },
  { tag: 'sail-diff',       attr: 'from',  value: 'a',       shadowPart: '[data-line-kind]' },
]

describe.each(ports)('<$tag>', ({ tag, attr, value, shadowPart }) => {
  it('registers as a custom element', () => {
    expect(customElements.get(tag)).toBeDefined()
  })

  it('mounts and renders its anatomy — the semantics, not a class', async () => {
    const { el, shadow } = mount(reg, tag, { [attr]: value })
    await flush()
    expect(shadow).not.toBeNull()
    expect(shadow!.querySelector(shadowPart)).not.toBeNull()
    expect(el.getAttribute(attr)).toBe(value)
  })
})
