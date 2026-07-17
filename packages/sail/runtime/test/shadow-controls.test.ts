// A shadow component that nests a framework-styled control (a <button ghost>
// dismiss, an <input>) must carry the CONTROL SKINS in its shadow root — the
// light-DOM document contracts don't cross the boundary. Guards the fix for
// the sail-alert pilot's LANG-GAP (bare controls in every shadow L2).
//
// The *effect* (a nested button paints) is browser-only: jsdom doesn't
// support constructable adoptedStyleSheets, so we guard the invariant at the
// string level — the sheet adoptSuiSheets builds carries the control skins.

import { describe, expect, it } from 'vitest'
import { CONTROL_SKINS_CSS } from '../src/styles/index'

describe('shadow control skins', () => {
  it('the shadow-adopted control sheet carries the button/input/toggle skins', () => {
    expect(CONTROL_SKINS_CSS).toContain("button[variant='solid']")
    expect(CONTROL_SKINS_CSS).toContain("button[variant='ghost']")
    expect(CONTROL_SKINS_CSS).toContain("input[type='checkbox']")
    // Element-keyed tone channel so a nested <button tone=primary> resolves
    // --tone on itself, not only :host-keyed on the component.
    expect(CONTROL_SKINS_CSS).toMatch(/\[tone='primary'\]/)
  })

  it('omits the light-DOM host rules (those are the component element’s job)', () => {
    expect(CONTROL_SKINS_CSS).not.toContain('[data-component] { display: block')
    expect(CONTROL_SKINS_CSS).not.toContain('slot { display: contents')
  })
})
