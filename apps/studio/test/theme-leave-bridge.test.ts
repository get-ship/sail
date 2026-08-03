/**
 * Theme leave bridge — shell rail ↔ dirty leave guard (Job A).
 */
import { afterEach, describe, expect, it } from 'vitest'
import {
  __resetDesignForTests,
  __setWorkingDocForTests,
  getDesignState,
  setDesignMode,
} from '../src/design'
import {
  requestThemeLeaveNavigate,
  setThemeLeaveNavigate,
} from '../src/theme-session/theme-leave-bridge'

afterEach(() => {
  __resetDesignForTests()
  setThemeLeaveNavigate(null)
})

describe('theme leave bridge', () => {
  it('blocks shell navigate when dirty and leaving an open theme', async () => {
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'leave-toy' } },
      modes: {
        light: {
          color: { primary: { default: { $value: 'oklch(0.5 0.1 40)', $type: 'color' } } },
        },
      },
    })
    setDesignMode('light')
    const { setBrandColor } = await import('../src/design')
    await setBrandColor('#112233')
    expect(getDesignState().dirty).toBe(true)

    let called: string | null = null
    setThemeLeaveNavigate((href) => {
      called = href
      return false
    })

    const orig = globalThis.location
    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: { ...orig, pathname: '/themes/leave-toy/color' },
    })

    const proceed = requestThemeLeaveNavigate('/edit')
    expect(proceed).toBe(false)
    expect(called).toBe('/edit')

    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: orig,
    })
  })

  it('allows navigate when destination stays on the same theme', async () => {
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'leave-toy' } },
      modes: {
        light: {
          color: { primary: { default: { $value: 'oklch(0.5 0.1 40)', $type: 'color' } } },
        },
      },
    })
    setDesignMode('light')
    const { setBrandColor } = await import('../src/design')
    await setBrandColor('#445566')
    expect(getDesignState().dirty).toBe(true)

    let called = false
    setThemeLeaveNavigate(() => {
      called = true
      return false
    })

    const orig = globalThis.location
    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: { ...orig, pathname: '/themes/leave-toy/overview' },
    })

    // Facet switch within same theme must not open leave dialog.
    expect(requestThemeLeaveNavigate('/themes/leave-toy/color')).toBe(true)
    expect(called).toBe(false)
    expect(requestThemeLeaveNavigate('/themes/leave-toy/shape')).toBe(true)
    expect(called).toBe(false)

    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: orig,
    })
  })

  it('allows navigate when not dirty even if leave handler is registered', async () => {
    __setWorkingDocForTests({
      $extensions: { ship: { themeId: 'leave-toy' } },
      modes: {
        light: {
          color: { primary: { default: { $value: 'oklch(0.5 0.1 40)', $type: 'color' } } },
        },
      },
    })
    expect(getDesignState().dirty).toBe(false)
    let called = false
    setThemeLeaveNavigate(() => {
      called = true
      return false
    })
    const orig = globalThis.location
    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: { ...orig, pathname: '/themes/leave-toy/overview' },
    })
    expect(requestThemeLeaveNavigate('/edit')).toBe(true)
    expect(called).toBe(false)
    Object.defineProperty(globalThis, 'location', {
      configurable: true,
      value: orig,
    })
  })
})
