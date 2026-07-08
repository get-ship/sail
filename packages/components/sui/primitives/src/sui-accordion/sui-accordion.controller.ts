import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-accordion.sui'

export default controller<Scope>({
  mount(state, host) {

    const getItems = (): Element[] =>
      Array.from(host.querySelectorAll(':scope > sui-accordion-item'))

    const getEnabledItems = (): Element[] =>
      getItems().filter(i => !i.hasAttribute('disabled'))

    const getOpenValues = (): Set<string> => {
      const v = state.$signals.value()
      return new Set(v ? v.split(',').filter(Boolean) : [])
    }

    const syncItems = () => {
      const open = getOpenValues()
      for (const item of getItems()) {
        const val = item.getAttribute('value') || ''
        if (open.has(val)) {
          item.setAttribute('open', '')
        } else {
          item.removeAttribute('open')
        }
      }
    }

    const stopEffect = effect(() => {
      state.$signals.value()
      syncItems()
    })

    // Re-sync when items are added/removed AFTER connect. The value→open
    // effect only re-runs on `value` changes, so items inserted later (dynamic
    // children, or a framework that sets attributes then projects children)
    // would never pick up the initial open value without this.
    const mo = new MutationObserver(syncItems)
    mo.observe(host, { childList: true })

    const onToggle = (e: Event) => {
      const ev = e as CustomEvent
      ev.stopPropagation()
      const itemVal = ev.detail?.value as string
      const open = getOpenValues()

      if (state.$signals.type() === 'single') {
        state.$signals.value.set(open.has(itemVal) ? '' : itemVal)
      } else {
        if (open.has(itemVal)) {
          open.delete(itemVal)
        } else {
          open.add(itemVal)
        }
        state.$signals.value.set(Array.from(open).join(','))
      }

      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: state.$signals.value() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onNav = (e: Event) => {
      const ev = e as CustomEvent
      ev.stopPropagation()
      const enabled = getEnabledItems()
      if (!enabled.length) return

      const current = (ev.target as Element | null)?.closest('sui-accordion-item')
      const idx = current ? enabled.indexOf(current) : -1
      let target: Element | undefined

      switch (ev.detail?.direction) {
        case 'next':  target = enabled[(idx + 1) % enabled.length]; break
        case 'prev':  target = enabled[(idx - 1 + enabled.length) % enabled.length]; break
        case 'first': target = enabled[0]; break
        case 'last':  target = enabled[enabled.length - 1]; break
      }

      if (target) {
        const summary = target.shadowRoot?.querySelector('summary') as HTMLElement | null
        summary?.focus()
      }
    }

    host.addEventListener('sui-accordion-item-toggle', onToggle)
    host.addEventListener('sui-accordion-nav', onNav)

    return () => {
      stopEffect()
      mo.disconnect()
      host.removeEventListener('sui-accordion-item-toggle', onToggle)
      host.removeEventListener('sui-accordion-nav', onNav)
    }
  },
})
