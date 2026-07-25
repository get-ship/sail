// Residual: opt-in shadow containment wrappers that nest a framework-styled
// control (a <button ghost> dismiss) need CONTROL_SKINS in the adopted sheet —
// document contracts do not cross a shadow boundary. L1 paint does not use this
// path. Guarded at the string level (jsdom lacks constructable adoptedStyleSheets).

import { describe, expect, it } from 'vitest'
import { CONTROL_SKINS_CSS } from '../src/styles/index'

describe('control skins (residual shadow adoption + light contracts share form)', () => {
  it('carries button/input/toggle skins without tone attr channels', () => {
    expect(CONTROL_SKINS_CSS).toContain("button[variant='solid']")
    expect(CONTROL_SKINS_CSS).toContain("button[variant='ghost']")
    expect(CONTROL_SKINS_CSS).toContain("input[type='checkbox']")
    expect(CONTROL_SKINS_CSS).not.toMatch(/\[tone='primary'\]/)
    expect(CONTROL_SKINS_CSS).not.toContain(':host')
  })

  it('omits the light-DOM host rules (those are the component element’s job)', () => {
    expect(CONTROL_SKINS_CSS).not.toContain('[data-component] { display: block')
    expect(CONTROL_SKINS_CSS).not.toContain('slot { display: contents')
  })
})
