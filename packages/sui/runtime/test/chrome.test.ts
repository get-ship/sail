import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { chrome, notifyViewUnmount } from '../src/chrome'
import { setRouterPort } from '../src/route'

interface CapturedEvent {
  type: string
  detail: unknown
}

function captureAll(types: string[]): { events: CapturedEvent[]; restore: () => void } {
  const events: CapturedEvent[] = []
  const handlers = new Map<string, (e: Event) => void>()
  for (const t of types) {
    const h = (e: Event): void => {
      events.push({ type: e.type, detail: (e as CustomEvent).detail })
    }
    handlers.set(t, h)
    document.addEventListener(t, h)
  }
  return {
    events,
    restore() {
      for (const [t, h] of handlers) document.removeEventListener(t, h)
    },
  }
}

describe('chrome wrapper', () => {
  let restore: () => void

  beforeEach(() => {
    delete (window as { __suiCurrentViewId?: unknown }).__suiCurrentViewId
    document.body.removeAttribute('style')
  })

  afterEach(() => {
    restore?.()
  })

  describe('notify', () => {
    it('dispatches sui:notify with the opts as detail', () => {
      const cap = captureAll(['sui:notify'])
      restore = cap.restore
      chrome.notify({ tone: 'success', title: 'Saved', duration: 1000 })
      expect(cap.events).toHaveLength(1)
      expect(cap.events[0]!.detail).toMatchObject({
        tone: 'success',
        title: 'Saved',
        duration: 1000,
      })
    })

    it('normalizes the `error` tone alias to `destructive`', () => {
      const cap = captureAll(['sui:notify'])
      restore = cap.restore
      chrome.notify({ tone: 'error', title: 'Boom' })
      const detail = cap.events[0]!.detail as { tone: string }
      expect(detail.tone).toBe('destructive')
    })
  })

  describe('navigate', () => {
    // chrome.navigate delegates to the unified router via the runtime
    // navigate() → host RouterPort (ADR unified-deck-router). It no longer
    // pushes history or emits a `sui:navigate` *trigger* itself — the host
    // router owns history + mounting and emits the route-change notification.
    it('delegates to the registered RouterPort', () => {
      const port = { navigate: vi.fn(), back: vi.fn() }
      setRouterPort(port)
      restore = () => { (globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined }
      chrome.navigate('/agents/foo')
      expect(port.navigate).toHaveBeenCalledWith('/agents/foo', {})
    })

    it('forwards {replace: true} to the port', () => {
      const port = { navigate: vi.fn(), back: vi.fn() }
      setRouterPort(port)
      restore = () => { (globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined }
      chrome.navigate('/skills', { replace: true })
      expect(port.navigate).toHaveBeenCalledWith('/skills', { replace: true })
    })

    it('falls back to history when no router is registered', () => {
      ;(globalThis as { __shipRouterPort?: unknown }).__shipRouterPort = undefined
      const pushSpy = vi.spyOn(history, 'pushState')
      restore = () => pushSpy.mockRestore()
      chrome.navigate('/agents/foo')
      expect(pushSpy).toHaveBeenCalledTimes(1)
    })
  })

  describe('dock', () => {
    it('emits the right event per method', () => {
      const cap = captureAll([
        'sui:dock:toggle', 'sui:dock:show', 'sui:dock:hide',
        'sui:dock:activate', 'sui:dock:clear', 'sui:dock:popout',
        'sui:dock:close', 'sui:dock:resize',
      ])
      restore = cap.restore
      chrome.dock.toggle()
      chrome.dock.show()
      chrome.dock.hide()
      chrome.dock.activate('terminal')
      chrome.dock.clear('terminal')
      chrome.dock.popout('terminal')
      chrome.dock.close()
      chrome.dock.resize(320)
      expect(cap.events.map((e) => e.type)).toEqual([
        'sui:dock:toggle', 'sui:dock:show', 'sui:dock:hide',
        'sui:dock:activate', 'sui:dock:clear', 'sui:dock:popout',
        'sui:dock:close', 'sui:dock:resize',
      ])
      expect(cap.events[3]!.detail).toMatchObject({ tab: 'terminal' })
      expect(cap.events[7]!.detail).toMatchObject({ height: 320 })
    })
  })

  describe('bar', () => {
    it('forwards mode + prefill on ship:bar:open', () => {
      const cap = captureAll(['ship:bar:open', 'ship:bar:close'])
      restore = cap.restore
      chrome.bar.open('find', '>err')
      chrome.bar.close()
      expect(cap.events[0]!.detail).toMatchObject({ mode: 'find', prefill: '>err' })
      expect(cap.events[1]!.type).toBe('ship:bar:close')
    })
  })

  describe('state-bearing slots stamp the viewId', () => {
    it('reads window.__suiCurrentViewId by default', () => {
      ;(window as { __suiCurrentViewId?: string }).__suiCurrentViewId = 'view-42'
      const cap = captureAll(['sui:breadcrumb:set', 'sui:status:set', 'sui:theme:set', 'sui:focus:enter'])
      restore = cap.restore
      chrome.breadcrumb.set([{ label: 'X' }])
      chrome.status.set('k', { label: 'v' })
      chrome.theme.set('--x', '1')
      chrome.focus.enter()
      for (const ev of cap.events) {
        expect((ev.detail as { viewId?: string }).viewId).toBe('view-42')
      }
    })

    it('lets the caller override via { viewId }', () => {
      ;(window as { __suiCurrentViewId?: string }).__suiCurrentViewId = 'shell'
      const cap = captureAll(['sui:breadcrumb:set'])
      restore = cap.restore
      chrome.breadcrumb.set([{ label: 'Y' }], { viewId: 'override' })
      expect((cap.events[0]!.detail as { viewId?: string }).viewId).toBe('override')
    })

    it('omits viewId when no current and no override', () => {
      const cap = captureAll(['sui:status:set'])
      restore = cap.restore
      chrome.status.set('k', { label: 'v' })
      const detail = cap.events[0]!.detail as { viewId?: string }
      expect(detail.viewId).toBeUndefined()
    })
  })

  describe('breadcrumb.clear', () => {
    it('emits sui:breadcrumb:set with an empty items array', () => {
      const cap = captureAll(['sui:breadcrumb:set'])
      restore = cap.restore
      chrome.breadcrumb.clear()
      const detail = cap.events[0]!.detail as { items: unknown[] }
      expect(detail.items).toEqual([])
    })
  })

  describe('modal.request', () => {
    it('resolves with the action label dispatched via sui:modal:respond', async () => {
      const cap = captureAll(['sui:modal:request'])
      restore = cap.restore
      const pending = chrome.modal.request({
        title: 'Confirm?',
        actions: ['Cancel', 'Confirm'],
      })
      expect(cap.events).toHaveLength(1)
      const { id } = cap.events[0]!.detail as { id: number }
      document.dispatchEvent(
        new CustomEvent('sui:modal:respond', { detail: { id, action: 'Confirm' } }),
      )
      await expect(pending).resolves.toBe('Confirm')
    })

    it('queues distinct ids per request', async () => {
      const cap = captureAll(['sui:modal:request'])
      restore = cap.restore
      const a = chrome.modal.request({ title: 'A', actions: ['ok'] })
      const b = chrome.modal.request({ title: 'B', actions: ['ok'] })
      const ids = cap.events.map((e) => (e.detail as { id: number }).id)
      expect(new Set(ids).size).toBe(2)
      // Respond out of order — each promise resolves to its own action.
      document.dispatchEvent(new CustomEvent('sui:modal:respond', { detail: { id: ids[1], action: 'B-out' } }))
      document.dispatchEvent(new CustomEvent('sui:modal:respond', { detail: { id: ids[0], action: 'A-out' } }))
      await expect(a).resolves.toBe('A-out')
      await expect(b).resolves.toBe('B-out')
    })
  })

  describe('notifyViewUnmount', () => {
    it('dispatches sui:view:unmount with the viewId', () => {
      const cap = captureAll(['sui:view:unmount'])
      restore = cap.restore
      notifyViewUnmount('view-99')
      expect(cap.events[0]!.detail).toMatchObject({ viewId: 'view-99' })
    })
  })
})
