import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  attachStudioBridge,
  setFabVisible,
  STUDIO_HOST_ID,
} from '../src/index'

const LS_FAB = 'sail-studio.fab'

function setDevMeta() {
  document.head.innerHTML = '<meta name="deck-mode" content="development" />'
}

describe('studio bridge', () => {
  let handle: ReturnType<typeof attachStudioBridge>
  let prevPath: string

  beforeEach(() => {
    document.documentElement.innerHTML = ''
    setDevMeta()
    document.body.innerHTML = '<main id="app"><button id="t">t</button></main>'
    try {
      localStorage.removeItem(LS_FAB)
    } catch {
      /* */
    }
    prevPath = location.pathname
    // Host-app default path for happy-dom.
    history.replaceState(null, '', '/')
  })

  afterEach(() => {
    handle?.close()
    try {
      localStorage.removeItem(LS_FAB)
    } catch {
      /* */
    }
    history.replaceState(null, '', prevPath || '/')
  })

  it('no-ops outside development', () => {
    document.head.innerHTML = '<meta name="deck-mode" content="production" />'
    handle = attachStudioBridge()
    expect(document.getElementById(STUDIO_HOST_ID)).toBeNull()
    expect(document.getElementById('sail-studio-fab')).toBeNull()
  })

  it('mounts host outside #root and FAB on documentElement', () => {
    handle = attachStudioBridge()
    const host = document.getElementById(STUDIO_HOST_ID)
    const fab = document.getElementById('sail-studio-fab')
    expect(host).toBeTruthy()
    expect(fab).toBeTruthy()
    expect(host!.parentElement).toBe(document.documentElement)
    expect(document.getElementById('app')?.contains(host!)).toBe(false)
  })

  it('⌘⇧S toggles open (API + hotkey)', () => {
    handle = attachStudioBridge()
    handle.setOpen(true)
    expect(handle.isOpen()).toBe(true)
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 's', metaKey: true, shiftKey: true, bubbles: true }),
    )
    expect(handle.isOpen()).toBe(false)
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 's', metaKey: true, shiftKey: true, bubbles: true }),
    )
    expect(handle.isOpen()).toBe(true)
  })

  it('⌘⇧A enters annotate mode; esc returns to inspect', () => {
    handle = attachStudioBridge()
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'a', metaKey: true, shiftKey: true, bubbles: true }),
    )
    expect(handle.mode()).toBe('annotate')
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(handle.mode()).toBe('inspect')
  })

  it('⌘⇧X enters screenshot mode', () => {
    handle = attachStudioBridge()
    document.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'x', metaKey: true, shiftKey: true, bubbles: true }),
    )
    expect(handle.mode()).toBe('screenshot')
  })

  it('host uses shadow root containment', () => {
    handle = attachStudioBridge()
    const host = document.getElementById(STUDIO_HOST_ID)!
    expect(host.shadowRoot).toBeTruthy()
    expect(host.style.contain).toContain('strict')
  })

  it('setFabVisible hides and shows FAB + localStorage', () => {
    handle = attachStudioBridge()
    expect(document.getElementById('sail-studio-fab')!.style.display).not.toBe('none')
    setFabVisible(false)
    expect(document.getElementById('sail-studio-fab')!.style.display).toBe('none')
    expect(localStorage.getItem(LS_FAB)).toBe('0')
    setFabVisible(true)
    expect(document.getElementById('sail-studio-fab')!.style.display).toBe('')
    expect(localStorage.getItem(LS_FAB)).toBe('1')
  })

  it('FAB unmounts while studio is open and remounts on close', () => {
    handle = attachStudioBridge()
    expect(document.getElementById('sail-studio-fab')).toBeTruthy()
    handle.setOpen(true)
    expect(document.getElementById('sail-studio-fab')).toBeNull()
    handle.setOpen(false)
    expect(document.getElementById('sail-studio-fab')).toBeTruthy()
  })

  it('FAB does not mount when studio is already the app document (meta)', () => {
    document.head.innerHTML =
      '<meta name="deck-mode" content="development" /><meta name="sail-project" content="apps/studio" />'
    handle = attachStudioBridge()
    expect(document.getElementById('sail-studio-fab')).toBeNull()
  })

  it('FAB does not mount on /_studio path', () => {
    history.replaceState(null, '', '/_studio')
    handle = attachStudioBridge()
    expect(document.getElementById('sail-studio-fab')).toBeNull()
  })

  it('postMessage close from Studio closes the host overlay', () => {
    handle = attachStudioBridge()
    handle.setOpen(true)
    expect(handle.isOpen()).toBe(true)
    window.dispatchEvent(
      new MessageEvent('message', {
        data: { channel: 'sail-studio', type: 'close' },
      }),
    )
    expect(handle.isOpen()).toBe(false)
  })

  it('FAB does not mount when light-DOM shell is already present', () => {
    document.body.innerHTML =
      '<div data-sail-studio-shell data-studio-rail><main id="app">studio</main></div>'
    handle = attachStudioBridge()
    expect(document.getElementById('sail-studio-fab')).toBeNull()
  })

  it('open injects /_studio #root HTML into shadow frame', async () => {
    const shell =
      '<div id="root"><div data-sail-studio-shell data-studio-rail><button data-activity="edit">EDIT</button></div></div>'
    const orig = globalThis.fetch
    globalThis.fetch = (async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.includes('/_studio')) {
        return new Response(`<!doctype html><html><body>${shell}</body></html>`, {
          status: 200,
          headers: { 'content-type': 'text/html' },
        })
      }
      return orig(input as RequestInfo)
    }) as typeof fetch

    handle = attachStudioBridge()
    handle.setOpen(true)
    // fill is async
    await new Promise((r) => setTimeout(r, 20))
    const host = document.getElementById(STUDIO_HOST_ID)!
    const frame = host.shadowRoot!.getElementById('f')!
    expect(frame.innerHTML).toMatch(/data-sail-studio-shell/)
    expect(frame.innerHTML).toMatch(/data-activity="edit"/)
    expect(frame.textContent).toMatch(/EDIT/)
    globalThis.fetch = orig
  })
})
