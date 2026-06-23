import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-portal/sui-portal.compiled.js'
import '../src/sui-toast/sui-toast.compiled.js'
import '../src/sui-toast/sui-toast-container.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

type SuiToast = HTMLElement & { dismiss(r?: string): void; close(r?: string): void }
type SuiToastContainer = HTMLElement & { show(opts?: Record<string, unknown>): SuiToast }

const shadow = (el: HTMLElement) => el.shadowRoot!
const closeBtn = (el: HTMLElement) => shadow(el).querySelector<HTMLButtonElement>('.close')
const progressEl = (el: HTMLElement) => shadow(el).querySelector<HTMLElement>('.progress')
const autoIcon = (el: HTMLElement) => {
  const wrap = shadow(el).querySelector<HTMLElement>('.icon')
  return wrap ? wrap.querySelector<SVGSVGElement>('svg') : null
}
const toastDiv = (el: HTMLElement) => shadow(el).querySelector<HTMLDivElement>('.toast')

describe('<sui-toast>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-toast')).toBeDefined()
  })

  // ── Rendering ─────────────────────────────────────────────────────────────

  it('renders with tone=info — auto-icon SVG present in shadow', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'info', duration: '0' })
    const svg = autoIcon(el)
    expect(svg).not.toBeNull()
    expect(svg!.getAttribute('aria-hidden')).toBe('true')
  })

  it('renders with tone=success — auto-icon SVG present', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'success', duration: '0' })
    expect(autoIcon(el)).not.toBeNull()
  })

  it('renders with tone=warning — auto-icon SVG present', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'warning', duration: '0' })
    expect(autoIcon(el)).not.toBeNull()
  })

  it('renders with tone=destructive — auto-icon SVG present', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'destructive', duration: '0' })
    expect(autoIcon(el)).not.toBeNull()
  })

  it('renders with tone=accent — auto-icon SVG present', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'accent', duration: '0' })
    expect(autoIcon(el)).not.toBeNull()
  })

  it('renders with tone=default — no auto-icon, data-no-icon set', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'default', duration: '0' })
    const svg = autoIcon(el)
    expect(!svg || svg.hidden).toBe(true)
    expect(el.hasAttribute('data-no-icon')).toBe(true)
  })

  it('consumer-supplied icon suppresses auto-icon', async () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'info', duration: '0' },
      '<svg slot="icon" data-custom></svg>')
    await flush()
    expect(el.querySelector('svg[data-custom]')).not.toBeNull()
    const svg = autoIcon(el)
    expect(!svg || svg.hidden).toBe(true)
  })

  it('size sm — reflects attribute', () => {
    const { el } = mount(reg, 'sui-toast', { size: 'sm', duration: '0' })
    expect(el.getAttribute('size')).toBe('sm')
  })

  it('size lg — reflects attribute', () => {
    const { el } = mount(reg, 'sui-toast', { size: 'lg', duration: '0' })
    expect(el.getAttribute('size')).toBe('lg')
  })

  it('slotted title projects into shadow', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' },
      '<span slot="title">My Title</span>')
    expect(el.textContent).toContain('My Title')
  })

  it('slotted body projects into shadow', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' }, 'Body text here')
    expect(el.textContent).toContain('Body text here')
  })

  it('slotted actions project into shadow', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' },
      '<button slot="actions">Undo</button>')
    expect(el.querySelector('[slot="actions"]')).not.toBeNull()
  })

  it('renders close button when dismissible (default)', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    expect(closeBtn(el)).not.toBeNull()
  })

  it('dismissible=false — no close button rendered', async () => {
    const { el } = mount(reg, 'sui-toast', { dismissible: true, duration: '0' })
    el.removeAttribute('dismissible')
    await flush()
    expect(closeBtn(el)).toBeNull()
  })

  // ── Auto-dismiss ───────────────────────────────────────────────────────────

  it('duration=0 — host stays connected, no timer fires', async () => {
    vi.useFakeTimers()
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    vi.advanceTimersByTime(10_000)
    expect(spy).not.toHaveBeenCalled()
    expect(el.isConnected).toBe(true)
    vi.useRealTimers()
  })

  it('duration=500 — timer fires, host removed, sui-dismiss with reason=timeout (bubbles + composed)', () => {
    vi.useFakeTimers()
    const { el } = mount(reg, 'sui-toast', { duration: '500' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    vi.advanceTimersByTime(500)
    expect(spy).toHaveBeenCalledTimes(1)
    const evt = spy.mock.calls[0]![0] as CustomEvent
    expect(evt.detail).toEqual({ reason: 'timeout' })
    expect(evt.bubbles).toBe(true)
    expect(evt.composed).toBe(true)
    expect(el.isConnected).toBe(false)
    vi.useRealTimers()
  })

  it('hover pauses timer; mouseleave resumes', () => {
    vi.useFakeTimers()
    const { el } = mount(reg, 'sui-toast', { duration: '1000' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    // Advance 400ms, pause
    vi.advanceTimersByTime(400)
    el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
    // Advance another 700ms while paused — should NOT dismiss
    vi.advanceTimersByTime(700)
    expect(spy).not.toHaveBeenCalled()
    // Resume
    el.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }))
    // Remaining ≈ 600ms; advance 600ms → should dismiss
    vi.advanceTimersByTime(600)
    expect(spy).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })

  it('focus pauses timer; focusout resumes', () => {
    vi.useFakeTimers()
    const { el } = mount(reg, 'sui-toast', { duration: '1000' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    vi.advanceTimersByTime(300)
    el.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
    vi.advanceTimersByTime(800)
    expect(spy).not.toHaveBeenCalled()
    el.dispatchEvent(new FocusEvent('focusout', { bubbles: true }))
    vi.advanceTimersByTime(700)
    expect(spy).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })

  // ── Dismissal ─────────────────────────────────────────────────────────────

  it('close button click → sui-dismiss with reason=manual, host removed', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    closeBtn(el)!.click()
    expect(spy).toHaveBeenCalledTimes(1)
    expect((spy.mock.calls[0]![0] as CustomEvent).detail.reason).toBe('manual')
    expect(el.isConnected).toBe(false)
  })

  it('Escape key → sui-dismiss reason=manual (when dismissible)', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(spy).toHaveBeenCalledTimes(1)
    expect((spy.mock.calls[0]![0] as CustomEvent).detail.reason).toBe('manual')
    expect(el.isConnected).toBe(false)
  })

  it('Escape key ignored when dismissible=false', async () => {
    const { el } = mount(reg, 'sui-toast', { dismissible: true, duration: '0' })
    el.removeAttribute('dismissible')
    await flush()
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(spy).not.toHaveBeenCalled()
    expect(el.isConnected).toBe(true)
  })

  it('programmatic dismiss(programmatic) → event with that reason + host removed', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    const t = el as SuiToast
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    t.dismiss('programmatic')
    expect(spy).toHaveBeenCalledTimes(1)
    expect((spy.mock.calls[0]![0] as CustomEvent).detail.reason).toBe('programmatic')
    expect(el.isConnected).toBe(false)
  })

  it('close() under prefers-reduced-motion falls back to sync dismiss', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    const t = el as SuiToast
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    // matchMedia is stubbed to return matches=false in test setup; override for this test
    const origMatch = window.matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: (_q: string) => ({
        matches: true, media: _q, onchange: null,
        addListener: () => {}, removeListener: () => {},
        addEventListener: () => {}, removeEventListener: () => {},
        dispatchEvent: () => false,
      }),
    })
    void t.close()
    expect(spy).toHaveBeenCalledTimes(1)
    expect(el.isConnected).toBe(false)
    Object.defineProperty(window, 'matchMedia', { writable: true, value: origMatch })
  })

  // ── ARIA ──────────────────────────────────────────────────────────────────

  it('tone=destructive — inner div role=alert, aria-live=assertive', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'destructive', duration: '0' })
    const div = toastDiv(el)
    expect(div?.getAttribute('role')).toBe('alert')
    expect(div?.getAttribute('aria-live')).toBe('assertive')
  })

  it('tone=warning — inner div role=alert, aria-live=assertive', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'warning', duration: '0' })
    const div = toastDiv(el)
    expect(div?.getAttribute('role')).toBe('alert')
    expect(div?.getAttribute('aria-live')).toBe('assertive')
  })

  it('tone=info — inner div role=status, aria-live=polite', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'info', duration: '0' })
    const div = toastDiv(el)
    expect(div?.getAttribute('role')).toBe('status')
    expect(div?.getAttribute('aria-live')).toBe('polite')
  })

  it('tone=default — inner div role=status, aria-live=polite', () => {
    const { el } = mount(reg, 'sui-toast', { tone: 'default', duration: '0' })
    const div = toastDiv(el)
    expect(div?.getAttribute('role')).toBe('status')
    expect(div?.getAttribute('aria-live')).toBe('polite')
  })

  it('host gets tabindex=0 if not already set', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '0' })
    expect(el.getAttribute('tabindex')).toBe('0')
  })

  // ── Progress bar ──────────────────────────────────────────────────────────

  it('show-progress=true with duration>0 — progress bar rendered', () => {
    const { el } = mount(reg, 'sui-toast', { 'show-progress': true, duration: '5000' })
    expect(progressEl(el)).not.toBeNull()
  })

  it('show-progress=false — no progress bar', () => {
    const { el } = mount(reg, 'sui-toast', { duration: '5000' })
    expect(progressEl(el)).toBeNull()
  })

  it('show-progress=true with duration=0 — no progress bar', () => {
    const { el } = mount(reg, 'sui-toast', { 'show-progress': true, duration: '0' })
    expect(progressEl(el)).toBeNull()
  })

  // ── Cleanup ───────────────────────────────────────────────────────────────

  it('disconnect clears timer and removes listeners (no leaks)', () => {
    vi.useFakeTimers()
    const { el, container } = mount(reg, 'sui-toast', { duration: '500' })
    const spy = vi.fn()
    el.addEventListener('sui-dismiss', spy)
    container.remove()
    vi.advanceTimersByTime(1000)
    expect(spy).not.toHaveBeenCalled()
    vi.useRealTimers()
  })
})

// ── Container ────────────────────────────────────────────────────────────────

describe('<sui-toast-container>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('sui-toast-container')).toBeDefined()
  })

  it('sets role=region, aria-label, aria-live=polite on connect', () => {
    const { el } = mount(reg, 'sui-toast-container')
    expect(el.getAttribute('role')).toBe('region')
    expect(el.getAttribute('aria-label')).toBe('Notifications')
    expect(el.getAttribute('aria-live')).toBe('polite')
  })

  it('position reflects to host attribute', () => {
    const { el } = mount(reg, 'sui-toast-container', { position: 'top-left' })
    expect(el.getAttribute('position')).toBe('top-left')
  })

  it('data-slide-from propagates to sui-toast children on slotchange', async () => {
    const { el } = mount(reg, 'sui-toast-container', { position: 'top-center' })
    const toast = document.createElement('sui-toast')
    toast.setAttribute('duration', '0')
    el.appendChild(toast)
    await flush()
    expect(toast.getAttribute('data-slide-from')).toBe('top')
  })

  it('position change re-applies data-slide-from to existing children', async () => {
    const { el } = mount(reg, 'sui-toast-container', { position: 'bottom-left' })
    const toast = document.createElement('sui-toast')
    toast.setAttribute('duration', '0')
    el.appendChild(toast)
    await flush()
    expect(toast.getAttribute('data-slide-from')).toBe('bottom')
    el.setAttribute('position', 'top-right')
    await flush()
    expect(toast.getAttribute('data-slide-from')).toBe('top')
  })

  it('show({ title, description, tone }) creates sui-toast in the portal stack', async () => {
    const { el } = mount(reg, 'sui-toast-container')
    const container = el as SuiToastContainer
    const toast = container.show({ title: 'Hello', description: 'World', tone: 'info', duration: 0 })
    expect(toast.tagName.toLowerCase()).toBe('sui-toast')
    expect(toast.getAttribute('tone')).toBe('info')
    expect(toast.querySelector('[slot="title"]')?.textContent).toBe('Hello')
    expect(toast.textContent).toContain('World')
    // sui-toast-container now portals toasts to a body-level stack — the
    // toast lives in document.body, not inside the container's light DOM.
    const stack = document.body.querySelector('[data-sui-toast-stack]')
    expect(stack?.contains(toast)).toBe(true)
  })

  it('max-visible=2 with 3 toasts — oldest gets close called', async () => {
    const { el } = mount(reg, 'sui-toast-container', { 'max-visible': '2' })
    const container = el as SuiToastContainer
    const t1 = container.show({ duration: 0 })
    const t2 = container.show({ duration: 0 })
    const closeSpy = vi.fn()
    ;(t1 as any).close = closeSpy
    container.show({ duration: 0 })
    await flush()
    expect(closeSpy).toHaveBeenCalledWith('programmatic')
    void t2
  })

  it('max-visible=0 — no overflow enforcement', async () => {
    const { el } = mount(reg, 'sui-toast-container', { 'max-visible': '0' })
    const container = el as SuiToastContainer
    const toasts = [
      container.show({ duration: 0 }),
      container.show({ duration: 0 }),
      container.show({ duration: 0 }),
      container.show({ duration: 0 }),
      container.show({ duration: 0 }),
      container.show({ duration: 0 }),
    ]
    await flush()
    const still = toasts.filter((t) => t.isConnected)
    expect(still.length).toBe(6)
  })
})
