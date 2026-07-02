import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'

import {
  installScrollRestoration,
  onNavigate,
  onPopState,
  applyScroll,
  __resetScrollRestoration,
} from '../src/scroll-restoration'

/** jsdom pins scrollX/scrollY to 0; override the getters so save() sees a real
 *  offset. Restored by vi.restoreAllMocks() in afterEach. */
function setScrollY(y: number): void {
  vi.spyOn(window, 'scrollY', 'get').mockReturnValue(y)
  vi.spyOn(window, 'scrollX', 'get').mockReturnValue(0)
}

function scrollKey(): number | undefined {
  return (window.history.state as { __shipScroll?: number } | null)?.__shipScroll
}

describe('scroll-restoration', () => {
  let scrollTo: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    __resetScrollRestoration()
    window.history.replaceState(null, '', '/')
    scrollTo = vi.spyOn(window, 'scrollTo').mockImplementation(() => {})
    // Make rAF synchronous so the applied scroll is observable inline.
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
      cb(0)
      return 0
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  it('installs manual scroll restoration and stamps the initial entry', () => {
    installScrollRestoration()
    expect(window.history.scrollRestoration).toBe('manual')
    expect(scrollKey()).toBeTypeOf('number')
  })

  it('scrolls to the top after a fresh push', () => {
    installScrollRestoration()
    const state = onNavigate(false)
    window.history.pushState(state, '', '/next')
    applyScroll()
    expect(scrollTo).toHaveBeenCalledWith(0, 0)
  })

  it('restores the saved offset on back/forward', () => {
    installScrollRestoration()
    const homeKey = scrollKey()!

    // Scroll the home page, then navigate to /next (saves home's offset).
    setScrollY(240)
    const nextState = onNavigate(false)
    window.history.pushState(nextState, '', '/next')
    applyScroll() // scrolls /next to the top
    scrollTo.mockClear()

    // Back to home (simulate the popstate state swap jsdom won't do for us).
    window.history.replaceState({ __shipScroll: homeKey }, '', '/')
    onPopState()
    applyScroll()
    expect(scrollTo).toHaveBeenCalledWith(0, 240)
  })

  it('keeps scroll (no scrollTo) on replace', () => {
    installScrollRestoration()
    onNavigate(true)
    applyScroll()
    expect(scrollTo).not.toHaveBeenCalled()
  })

  it('restores to 0,0 for an entry with no saved offset', () => {
    installScrollRestoration()
    window.history.replaceState({ __shipScroll: 4242 }, '', '/unknown')
    onPopState()
    applyScroll()
    expect(scrollTo).toHaveBeenCalledWith(0, 0)
  })

  it('is a no-op with no pending intent', () => {
    installScrollRestoration()
    applyScroll()
    expect(scrollTo).not.toHaveBeenCalled()
  })
})
