// chromeOffset — autohide state only. Height collapse is Style (`data-hidden`).
// No `--chrome-offset` mailbox. No `.style`.

import type { Action } from '../action'

const HIDE_AFTER_Y = 48

export const chromeOffset: Action<void> = (node) => {
  const el = node as HTMLElement

  let hidden = false
  let lastY = 0
  let enabled = false

  const readEnabled = (): boolean => {
    if (!el.hasAttribute('autohide')) return false
    const v = el.getAttribute('autohide')
    return v !== 'false' && v !== '0'
  }

  const applyHidden = (next: boolean): void => {
    if (!enabled) {
      hidden = false
      el.removeAttribute('data-hidden')
      return
    }
    hidden = next
    if (next) el.setAttribute('data-hidden', 'true')
    else el.removeAttribute('data-hidden')
  }

  const syncMode = (): void => {
    enabled = readEnabled()
    if (!enabled) applyHidden(false)
  }

  const onScroll = (): void => {
    if (!enabled) return
    const y = el.scrollTop || 0
    if (y < HIDE_AFTER_Y) applyHidden(false)
    else if (y > lastY + 4) applyHidden(true)
    else if (y < lastY - 4) applyHidden(false)
    lastY = y
  }

  syncMode()
  el.addEventListener('scroll', onScroll, { passive: true })
  const mo =
    typeof MutationObserver !== 'undefined'
      ? new MutationObserver(() => syncMode())
      : null
  mo?.observe(el, { attributes: true, attributeFilter: ['autohide'] })

  return {
    destroy: () => {
      mo?.disconnect()
      el.removeEventListener('scroll', onScroll)
      el.removeAttribute('data-hidden')
    },
  }
}
