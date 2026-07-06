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
import { createContainerRegistry, mount } from '../src/test-utils'

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
  shadowPart: string
}

const ports: PortCase[] = [
  { tag: 'sui-status-dot', attr: 'tone',  value: 'primary', shadowPart: '.dot' },
  { tag: 'sui-spinner',    attr: 'size',  value: 'lg',      shadowPart: '.ring' },
  { tag: 'sui-chip',       attr: 'tone',  value: 'primary', shadowPart: '.chip' },
  { tag: 'sui-card',       attr: 'padding', value: 'lg',    shadowPart: '.content' },
  { tag: 'sui-link',       attr: 'tone',  value: 'primary', shadowPart: 'a' },
  { tag: 'sui-separator',  attr: 'orientation', value: 'vertical', shadowPart: '.sep' },
  { tag: 'sui-skeleton',   attr: 'shape', value: 'circle',  shadowPart: '.sk' },
  { tag: 'sui-avatar',     attr: 'size',  value: 'lg',      shadowPart: '.root' },
  { tag: 'sui-button',     attr: 'tone',  value: 'primary', shadowPart: 'button' },
  { tag: 'sui-tabs',       attr: 'variant', value: 'pill',  shadowPart: '.strip' },
  { tag: 'sui-tab',        attr: 'name',  value: 'one',     shadowPart: 'button' },
  { tag: 'sui-empty-state', attr: 'tone', value: 'muted',   shadowPart: '.title' },
  { tag: 'sui-input',      attr: 'size',  value: 'lg',      shadowPart: 'input' },
]

describe.each(ports)('<$tag>', ({ tag, attr, value, shadowPart }) => {
  it('registers as a custom element', () => {
    expect(customElements.get(tag)).toBeDefined()
  })

  it('mounts with a shadow root containing the expected part', () => {
    const { el, shadow } = mount(reg, tag, { [attr]: value })
    expect(shadow).not.toBeNull()
    expect(shadow!.querySelector(shadowPart)).not.toBeNull()
    expect(el.getAttribute(attr)).toBe(value)
  })
})
