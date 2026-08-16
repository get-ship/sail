import { beforeEach } from 'vitest'
import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()

// Overlay hosts (dialog, drawer, popover, dropdown, tooltip, hover-card,
// context-menu): behavior tests locate the trigger or named target. The
// panel is not required to be in the a11y tree until opened. menu-item
// is mounted inside dropdown / context-menu. page / toggle / radio are
// parts of pagination / toggle-group / radio-group.

const openPopovers = new WeakSet<HTMLElement>()
let popoversWrapped = false
let observer: MutationObserver | null = null

wrapPopoverApi()

/** Happy-dom implements showPopover/hidePopover but does not hide a closed
 *  `[popover]` or reliably fire `toggle`. Product open/close still calls
 *  those methods; this only makes the a11y tree match the platform. */
function wrapPopoverApi(): void {
  if (popoversWrapped || typeof HTMLElement === 'undefined') return
  popoversWrapped = true

  const proto = HTMLElement.prototype as HTMLElement & {
    showPopover?: () => void
    hidePopover?: () => void
  }
  const origShow = proto.showPopover
  const origHide = proto.hidePopover

  proto.showPopover = function (this: HTMLElement) {
    if (openPopovers.has(this)) return
    this.hidden = false
    openPopovers.add(this)
    try {
      origShow?.call(this)
    } catch {
      /* already open / no native impl */
    }
    this.dispatchEvent(
      new ToggleEvent('toggle', { newState: 'open', oldState: 'closed', bubbles: false }),
    )
  }
  proto.hidePopover = function (this: HTMLElement) {
    if (!openPopovers.has(this)) return
    this.hidden = true
    openPopovers.delete(this)
    try {
      origHide?.call(this)
    } catch {
      /* already closed / no native impl */
    }
    this.dispatchEvent(
      new ToggleEvent('toggle', { newState: 'closed', oldState: 'open', bubbles: false }),
    )
  }
}

function hideClosedPopovers(root: ParentNode = document): void {
  for (const el of root.querySelectorAll('[popover]')) {
    if (el instanceof HTMLElement && !openPopovers.has(el)) el.hidden = true
  }
}

function onInvokerClick(e: Event): void {
  const t = e.target
  if (!(t instanceof Element)) return
  const inv = t.closest('[command][commandfor]')
  if (!inv) return
  const id = inv.getAttribute('commandfor')
  const cmd = inv.getAttribute('command')
  const el = id ? document.getElementById(id) : null
  if (!(el instanceof HTMLDialogElement)) return
  if (cmd === 'show-modal') el.showModal()
  else if (cmd === 'close') el.close()
}

beforeEach(() => {
  observer?.disconnect()
  document.removeEventListener('click', onInvokerClick)
  document.addEventListener('click', onInvokerClick)
  observer = new MutationObserver((muts) => {
    for (const m of muts) {
      for (const n of m.addedNodes) {
        if (!(n instanceof HTMLElement)) continue
        if (n.hasAttribute('popover') && !openPopovers.has(n)) n.hidden = true
        hideClosedPopovers(n)
      }
    }
    hideClosedPopovers(document)
  })
  observer.observe(document.documentElement, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['popover'],
  })
  hideClosedPopovers(document)
})
