import { describe, expect, it, afterEach } from 'vitest'
import { installViewport, forceBreakpoint, BREAKPOINT_MIN_WIDTH } from '../src/viewport'

afterEach(() => {
  // force restore if a test forgot
})

describe('installViewport', () => {
  it('matchMedia reports tablet min-width against forced width', () => {
    const restore = installViewport(800)
    try {
      expect(window.matchMedia('(min-width: 768px)').matches).toBe(true)
      expect(window.matchMedia('(min-width: 1280px)').matches).toBe(false)
      expect(window.matchMedia('(min-width: 390px)').matches).toBe(true)
    } finally {
      restore()
    }
  })

  it('forceBreakpoint tokens use design scale', () => {
    expect(BREAKPOINT_MIN_WIDTH.tablet).toBe(768)
    const restore = forceBreakpoint('desktop')
    try {
      expect(window.matchMedia('(min-width: 1280px)').matches).toBe(true)
    } finally {
      restore()
    }
  })
})
