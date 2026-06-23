import { controller, effect, bindNativeEvent } from '@ship/sui-runtime'
import type { Scope } from './sui-dropdown-menu.sui'

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

export default controller<Scope>({
  mount(state, host) {
    const seq = ++_seq
    const panelId = `sui-dropdown-menu-panel-${seq}`
    const anchorName = `--sui-dd-${seq}`

    // Per-instance anchor so multiple dropdowns on a page don't collide.
    // CSS reads `--sui-dd-anchor` from the host; the @supports block in
    // the .sui forwards that to the slotted trigger's anchor-name and
    // the panel's position-anchor.
    host.style.setProperty('--sui-dd-anchor', anchorName)

    const triggerWrap = state.$signals.triggerWrap() as HTMLElement | null
    const triggerSlot = state.$signals.triggerSlot() as HTMLSlotElement | null
    const panelEl = state.$signals.panelEl() as PopoverEl | null

    if (!panelEl) return
    panelEl.id = panelId

    // ── Typeahead state ──────────────────────────────────────────────────
    let typeaheadBuf = ''
    let typeaheadTimer: ReturnType<typeof setTimeout> | null = null

    function pushTypeahead(key: string): number {
      if (typeaheadTimer) clearTimeout(typeaheadTimer)
      typeaheadBuf += key.toLowerCase()
      typeaheadTimer = setTimeout(() => { typeaheadBuf = '' }, 350)
      const items = getItems()
      const i = items.indexOf(document.activeElement as HTMLElement)
      for (let j = 0; j < items.length; j++) {
        const idx = (i + 1 + j) % items.length
        const label = (items[idx]!.textContent ?? '').trim().toLowerCase()
        if (label.startsWith(typeaheadBuf)) return idx
      }
      return -1
    }

    // ── Item management ──────────────────────────────────────────────────
    function getItems(): HTMLElement[] {
      return Array.from(
        host.querySelectorAll<HTMLElement>('[role="menuitem"], button[data-value], [data-value]'),
      ).filter(el => !el.hasAttribute('disabled'))
    }

    function focusItem(index: number) {
      const items = getItems()
      if (!items.length) return
      const i = ((index % items.length) + items.length) % items.length
      for (const it of items) it.setAttribute('tabindex', '-1')
      items[i]!.setAttribute('tabindex', '0')
      items[i]!.focus()
    }

    function currentIndex(): number {
      return getItems().indexOf(document.activeElement as HTMLElement)
    }

    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute('role')) el.setAttribute('role', 'menuitem')
        if (!el.hasAttribute('tabindex')) el.setAttribute('tabindex', idx === 0 ? '0' : '-1')
      })
    }

    function setupTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[] ?? []
      for (const el of els) {
        el.setAttribute('aria-haspopup', 'menu')
        el.setAttribute('aria-controls', panelId)
        el.setAttribute('aria-expanded', state.$signals.open() ? 'true' : 'false')
      }
    }

    // ── Open/close ───────────────────────────────────────────────────────
    function openPanel() {
      const trigger = (triggerSlot?.assignedElements({ flatten: true }) as HTMLElement[])?.[0]
      // CSS Anchor Positioning owns the placement when the browser
      // supports it (Chromium 125+, Safari 26+, Firefox 145+). Only
      // fall back to inline-style positioning on older engines.
      if (trigger && panelEl && !anchorSupported) {
        const r = trigger.getBoundingClientRect()
        panelEl.style.top = `${r.bottom + 4}px`
        panelEl.style.left = `${r.left}px`
      }
      panelEl?.showPopover?.()
    }

    function closePanel() {
      panelEl?.hidePopover?.()
    }

    // ── Events ───────────────────────────────────────────────────────────
    const onTriggerClick = () => {
      if (state.$signals.open()) closePanel()
      else openPanel()
    }

    const onToggle = (ev: Event) => {
      const te = ev as ToggleEvent
      const nowOpen = te.newState === 'open'
      if (nowOpen !== state.$signals.open()) state.$signals.open.set(nowOpen)
      host.setAttribute('data-state', nowOpen ? 'open' : 'closed')
      setupTrigger()
      if (nowOpen) queueMicrotask(() => { setupItems(); focusItem(0) })
    }

    const onItemClick = (e: Event) => {
      const target = (e.target as HTMLElement).closest('[data-value]') as HTMLElement | null
      if (!target) return
      host.emit!('sui-select', { value: target.dataset.value ?? '' })
      closePanel()
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (!state.$signals.open()) return
      const items = getItems()
      if (!items.length) return
      const i = currentIndex()
      if (e.key === 'ArrowDown') { e.preventDefault(); focusItem(i + 1) }
      else if (e.key === 'ArrowUp') { e.preventDefault(); focusItem(i - 1) }
      else if (e.key === 'Home') { e.preventDefault(); focusItem(0) }
      else if (e.key === 'End') { e.preventDefault(); focusItem(items.length - 1) }
      else if (e.key === 'Escape') { e.preventDefault(); closePanel() }
      else if (e.key === 'Tab') { closePanel() }
      else if ((e.key === 'Enter' || e.key === ' ') && i >= 0) { e.preventDefault(); items[i]!.click() }
      else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key)
        if (next >= 0) { e.preventDefault(); focusItem(next) }
      }
    }

    if (triggerWrap) bindNativeEvent(triggerWrap, 'click', onTriggerClick)
    bindNativeEvent(panelEl, 'toggle', onToggle as EventListener)
    bindNativeEvent(panelEl, 'click', onItemClick)
    bindNativeEvent(host, 'keydown', onKeydown as EventListener)
    if (triggerSlot) bindNativeEvent(triggerSlot, 'slotchange', setupTrigger)
    bindNativeEvent(panelEl, 'slotchange', setupItems)

    const stopSync = effect(() => {
      const wantOpen = state.$signals.open()
      host.setAttribute('data-state', wantOpen ? 'open' : 'closed')
      setupTrigger()
    })

    return () => {
      if (typeaheadTimer) clearTimeout(typeaheadTimer)
      stopSync()
    }
  },
})
