import { controller, effect, bindNativeEvent, slotted } from '@ship/sui-runtime'
import type { Scope } from './sui-popover.sui'

// Per-instance anchor-name so multiple popovers on a page each address
// their own trigger via CSS Anchor Positioning.
let anchorSeq = 0

const anchorSupported =
  typeof CSS !== 'undefined' &&
  typeof CSS.supports === 'function' &&
  CSS.supports('anchor-name', '--x')

type Placement =
  | 'top' | 'top-end'
  | 'bottom' | 'bottom-end'
  | 'left' | 'left-end'
  | 'right' | 'right-end'

// Popover method surface (showPopover / hidePopover / togglePopover) is
// available on HTMLElement in the Popover-API-aware DOM. Cast at the
// call sites (`as PopoverApi`) where the controller invokes them,
// since the existence is feature-detected via `typeof el.showPopover`.
interface PopoverApi {
  showPopover?: () => void
  hidePopover?: () => void
  togglePopover?: () => boolean
}

export default controller<Scope>({
  mount(state, host) {
    const id = ++anchorSeq
    const anchorName = `--sui-popover-${id}`
    const panelId = `sui-popover-panel-${id}`

    host.style.setProperty('--sui-popover-anchor', anchorName)

    const triggerSlot = state.$signals.triggerSlot()
    const panelEl = state.$signals.panelEl() as (HTMLElement & PopoverApi) | null
    if (!panelEl) return

    panelEl.id = panelId

    // ── Trigger wiring ──────────────────────────────────────────────────
    // The platform handles open/close: setting popovertarget on the
    // slotted button wires its click + keyboard activation directly to
    // the popover. We do NOT add a manual click handler — that would
    // double-toggle and steal focus management from the browser.
    // We DO set the ARIA attributes the popover attribute doesn't infer
    // (haspopup) and reflect aria-expanded from the toggle event.
    slotted(state.$signals.triggerSlot as () => HTMLSlotElement | null, (els) => {
      for (const el of els as HTMLElement[]) {
        el.setAttribute('popovertarget', panelId)
        el.setAttribute('aria-haspopup', 'dialog')
        el.setAttribute('aria-controls', panelId)
        el.setAttribute('aria-expanded', state.$signals.open() ? 'true' : 'false')
      }
    })

    // Track open state locally. `:popover-open` is the authoritative source in
    // browsers, but some environments (and selector engines without that
    // pseudo) throw on `matches(':popover-open')` — fall back to this mirror.
    let popoverOpen = false
    const isPopoverOpen = (): boolean => {
      try {
        return panelEl.matches?.(':popover-open') ?? popoverOpen
      } catch {
        return popoverOpen
      }
    }

    // ── Native toggle event → reflect into state.$signals.open + emit sui-toggle
    function onToggle(ev: Event) {
      const newState = (ev as ToggleEvent).newState
      const nowOpen = newState === 'open'
      popoverOpen = nowOpen
      // Reflect attribute so consumer-observable state.$signals.open() flips.
      // Guard against feedback: we only act if the attribute disagrees.
      const hasAttr = host.hasAttribute('open')
      if (nowOpen && !hasAttr) host.setAttribute('open', '')
      else if (!nowOpen && hasAttr) host.removeAttribute('open')

      // Update aria-expanded on each trigger.
      const slot = triggerSlot as HTMLSlotElement | null
      const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
      for (const el of els) {
        el.setAttribute('aria-expanded', nowOpen ? 'true' : 'false')
      }

      host.emit!('sui-toggle', { open: nowOpen })

      if (nowOpen && !anchorSupported) positionFallback()
    }
    bindNativeEvent(panelEl, 'toggle', onToggle as EventListener)

    // ── Public methods (delegate to native) ────────────────────────────
    host.expose!({
      show: () => panelEl.showPopover?.(),
      hide: () => panelEl.hidePopover?.(),
      toggle: () =>
        typeof panelEl.togglePopover === 'function'
          ? void panelEl.togglePopover()
          : (isPopoverOpen()
              ? panelEl.hidePopover?.()
              : panelEl.showPopover?.()),
    })

    // ── Drive native popover from `open` prop ──────────────────────────
    // Consumer flips the attribute → effect calls show/hide. Native
    // light-dismiss flips the attribute via onToggle → effect sees the
    // values agree and no-ops. No feedback loop.
    const stopOpenSync = effect(() => {
      const wantOpen = state.$signals.open()
      const isOpen = isPopoverOpen()
      if (wantOpen && !isOpen) panelEl.showPopover?.()
      else if (!wantOpen && isOpen) panelEl.hidePopover?.()
    })

    // ── Position fallback for browsers without anchor-positioning ──────
    function positionFallback() {
      const slot = triggerSlot as HTMLSlotElement | null
      const els = (slot?.assignedElements({ flatten: true }) ?? []) as HTMLElement[]
      const trigger = els[0]
      if (!trigger || !panelEl) return
      const r = trigger.getBoundingClientRect()
      const placement = state.$signals.placement() as Placement
      panelEl.style.top = ''
      panelEl.style.left = ''
      panelEl.style.right = ''
      panelEl.style.bottom = ''
      const gap = 6
      switch (placement) {
        case 'top':
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`
          panelEl.style.left = `${r.left}px`
          break
        case 'top-end':
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`
          panelEl.style.right = `${window.innerWidth - r.right}px`
          break
        case 'bottom-end':
          panelEl.style.top = `${r.bottom + gap}px`
          panelEl.style.right = `${window.innerWidth - r.right}px`
          break
        case 'left':
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`
          panelEl.style.top = `${r.top}px`
          break
        case 'left-end':
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`
          break
        case 'right':
          panelEl.style.left = `${r.right + gap}px`
          panelEl.style.top = `${r.top}px`
          break
        case 'right-end':
          panelEl.style.left = `${r.right + gap}px`
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`
          break
        case 'bottom':
        default:
          panelEl.style.top = `${r.bottom + gap}px`
          panelEl.style.left = `${r.left}px`
      }
    }

    function onWindowChange() {
      if (!state.$signals.open()) return
      positionFallback()
    }
    if (!anchorSupported) {
      bindNativeEvent(window, 'resize', onWindowChange)
      bindNativeEvent(window, 'scroll', onWindowChange, { capture: true, passive: true })
    }

    return () => {
      stopOpenSync()
    }
  },
})
