import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-hover-card.sui'

let _seq = 0

// Browsers without CSS Anchor Positioning fall back to JS rect math.
// Cached at module load — there's no platform path that toggles this
// at runtime, so re-checking per mount is wasted work.
const anchorSupported =
  typeof CSS !== 'undefined' &&
  typeof CSS.supports === 'function' &&
  CSS.supports('anchor-name', '--x')

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

type Placement = 'top' | 'bottom' | 'left' | 'right'

const GAP = 8

function positionPanel(trigger: HTMLElement, panel: HTMLElement, placement: Placement) {
  const r = trigger.getBoundingClientRect()
  panel.style.top = ''
  panel.style.left = ''
  panel.style.right = ''
  panel.style.bottom = ''
  switch (placement) {
    case 'top':
      panel.style.bottom = `${window.innerHeight - r.top + GAP}px`
      panel.style.left = `${r.left + r.width / 2}px`
      panel.style.transform = 'translateX(-50%)'
      break
    case 'left':
      panel.style.right = `${window.innerWidth - r.left + GAP}px`
      panel.style.top = `${r.top}px`
      break
    case 'right':
      panel.style.left = `${r.right + GAP}px`
      panel.style.top = `${r.top}px`
      break
    case 'bottom':
    default:
      panel.style.top = `${r.bottom + GAP}px`
      panel.style.left = `${r.left + r.width / 2}px`
      panel.style.transform = 'translateX(-50%)'
  }
}

export default controller<Scope>({
  mount(state, host) {
    const seq = ++_seq
    const cardId = `sui-hover-card-${seq}`
    const anchorName = `--sui-hc-${seq}`

    // Per-instance anchor name. CSS forwards it from the host to the
    // slotted trigger's anchor-name and the panel's position-anchor.
    host.style.setProperty('--sui-hc-anchor', anchorName)

    const triggerSlot = state.$signals.triggerSlot() as HTMLSlotElement | null
    const panelEl = state.$signals.panelEl() as PopoverEl | null

    if (!panelEl) return
    panelEl.id = cardId

    let showTimer: ReturnType<typeof setTimeout> | null = null
    let hideTimer: ReturnType<typeof setTimeout> | null = null

    function clearTimers() {
      if (showTimer) { clearTimeout(showTimer); showTimer = null }
      if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
    }

    function wireTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[] ?? []
      for (const el of els) {
        el.setAttribute('aria-describedby', cardId)
      }
    }

    function doShow() {
      clearTimers()
      const trigger = (triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[])?.[0]
      // CSS Anchor Positioning owns placement when the browser supports
      // it. JS fallback only fires on older engines.
      if (trigger && panelEl && !anchorSupported) {
        positionPanel(trigger, panelEl as HTMLElement, (state.$signals.placement() || 'bottom') as Placement)
      }
      panelEl?.showPopover?.()
    }

    function doHide() {
      clearTimers()
      panelEl?.hidePopover?.()
    }

    ;(host as HTMLElement & { show(): void; hide(): void }).show = doShow
    ;(host as HTMLElement & { hide(): void }).hide = doHide

    const onToggle = (ev: Event) => {
      const te = ev as ToggleEvent
      const nowOpen = te.newState === 'open'
      if (nowOpen !== state.$signals.open()) state.$signals.open.set(nowOpen)
      host.setAttribute('data-state', nowOpen ? 'open' : 'closed')
      host.dispatchEvent(new CustomEvent(nowOpen ? 'sui-open' : 'sui-close', {
        bubbles: true, composed: true,
      }))
    }

    const onPointerEnter = () => {
      clearTimers()
      showTimer = setTimeout(() => doShow(), state.$signals.showDelay() ?? 300)
    }

    const onPointerLeave = () => {
      clearTimers()
      hideTimer = setTimeout(() => doHide(), state.$signals.hideDelay() ?? 100)
    }

    const onFocusIn = () => { clearTimers(); doShow() }
    const onFocusOut = () => {
      clearTimers()
      hideTimer = setTimeout(() => doHide(), state.$signals.hideDelay() ?? 100)
    }

    panelEl.addEventListener('toggle', onToggle)
    triggerSlot?.addEventListener('slotchange', wireTrigger)
    wireTrigger()

    host.addEventListener('pointerenter', onPointerEnter)
    host.addEventListener('pointerleave', onPointerLeave)
    host.addEventListener('focusin', onFocusIn)
    host.addEventListener('focusout', onFocusOut)

    const stopSync = effect(() => {
      host.setAttribute('data-state', state.$signals.open() ? 'open' : 'closed')
    })

    return () => {
      clearTimers()
      panelEl.removeEventListener('toggle', onToggle)
      triggerSlot?.removeEventListener('slotchange', wireTrigger)
      host.removeEventListener('pointerenter', onPointerEnter)
      host.removeEventListener('pointerleave', onPointerLeave)
      host.removeEventListener('focusin', onFocusIn)
      host.removeEventListener('focusout', onFocusOut)
      stopSync()
    }
  },
})
