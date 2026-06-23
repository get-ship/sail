import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-dialog.sui'
import { FocusReturn, acquireScrollLock, releaseScrollLock, focusableWithin } from '../overlay-focus'

let _seq = 0

export default controller<Scope>({
  mount(state, host) {
    const uid = ++_seq
    const headerId = `sui-dialog-header-${uid}`
    const bodyId = `sui-dialog-body-${uid}`

    const dialogEl = state.$signals.dialogEl() as HTMLDialogElement | null
    const closeBtn = state.$signals.closeBtn() as HTMLButtonElement | null
    const sentinelStart = state.$signals.sentinelStart() as HTMLElement | null
    const sentinelEnd = state.$signals.sentinelEnd() as HTMLElement | null

    if (!dialogEl) return

    // headerEl + bodyEl live inside the `{#if open()}` branch — they
    // come and go with each open cycle. Assign their `id` reactively
    // so aria-labelledby / aria-describedby keep resolving.
    const stopHeaderIdEffect = effect(() => {
      const h = state.$signals.headerEl() as HTMLElement | null
      if (h) h.id = headerId
    })
    const stopBodyIdEffect = effect(() => {
      const b = state.$signals.bodyEl() as HTMLElement | null
      if (b) b.id = bodyId
    })

    dialogEl.setAttribute('aria-labelledby', headerId)
    dialogEl.setAttribute('aria-describedby', bodyId)

    let hasScrollLock = false
    const focusReturn = new FocusReturn()

    function updateRole() {
      dialogEl!.setAttribute('role', state.$signals.roleOverride() || 'dialog')
      dialogEl!.setAttribute('aria-modal', 'true')
    }
    updateRole()

    function collectFocusables(): HTMLElement[] {
      const shadowFocusables = focusableWithin(dialogEl!).filter(
        el => !el.classList.contains('sentinel'),
      )
      const lightFocusables = focusableWithin(host)
      return [...lightFocusables, ...shadowFocusables]
    }

    function focusInitial() {
      const autofocus =
        host.querySelector<HTMLElement>('[autofocus]') ??
        dialogEl!.querySelector<HTMLElement>('[autofocus]')
      if (autofocus) { autofocus.focus(); return }
      const focusables = collectFocusables()
      const primary = focusables.find(el => el !== closeBtn) ?? focusables[0]
      if (primary) primary.focus()
      else dialogEl!.focus()
    }

    function doOpen() {
      focusReturn.save()
      try { dialogEl!.showModal() } catch { /* jsdom may not support */ }
      if (!hasScrollLock) { acquireScrollLock(); hasScrollLock = true }
      queueMicrotask(() => focusInitial())
      host.dispatchEvent(new CustomEvent('sui-open', { bubbles: true, composed: true }))
    }

    function doClose() {
      if ((dialogEl as HTMLDialogElement).open) dialogEl!.close()
    }

    const onNativeClose = () => {
      if (state.$signals.open()) state.$signals.open.set(false)
      if (hasScrollLock) { releaseScrollLock(); hasScrollLock = false }
      focusReturn.restore()
      host.dispatchEvent(new CustomEvent('sui-close', { bubbles: true, composed: true }))
    }

    const onBackdropClick = (e: MouseEvent) => {
      if (!state.$signals.dismissible()) return
      if (e.target === dialogEl) state.$signals.open.set(false)
    }

    // Escape on a modal dialog fires `cancel` before the native close. A
    // non-dismissible dialog must veto it here — preventDefault on cancel is
    // the platform's only hook to keep Escape from closing the dialog.
    const onCancel = (e: Event) => {
      if (!state.$signals.dismissible()) e.preventDefault()
    }

    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent('sui-dismiss', { bubbles: true, composed: true }))
      state.$signals.open.set(false)
    }

    const onSentinelStartFocus = () => {
      const items = collectFocusables()
      if (items.length) items[items.length - 1]!.focus()
    }
    const onSentinelEndFocus = () => {
      const items = collectFocusables()
      if (items.length) items[0]!.focus()
    }

    dialogEl.addEventListener('close', onNativeClose)
    dialogEl.addEventListener('cancel', onCancel)
    dialogEl.addEventListener('click', onBackdropClick)
    closeBtn?.addEventListener('click', onDismiss)
    sentinelStart?.addEventListener('focus', onSentinelStartFocus)
    sentinelEnd?.addEventListener('focus', onSentinelEndFocus)

    const stopOpenSync = effect(() => {
      const wantOpen = state.$signals.open()
      host.setAttribute('data-state', wantOpen ? 'open' : 'closed')
      if (wantOpen && !dialogEl.open) doOpen()
      else if (!wantOpen && dialogEl.open) doClose()
    })

    const stopRoleSync = effect(() => { updateRole() })

    ;(host as HTMLElement & { show(): void; showModal(): void; close(): void }).show = () =>
      state.$signals.open.set(true)
    ;(host as HTMLElement & { showModal(): void }).showModal = () => state.$signals.open.set(true)
    ;(host as HTMLElement & { close(): void }).close = () => state.$signals.open.set(false)

    return () => {
      if (hasScrollLock) releaseScrollLock()
      dialogEl.removeEventListener('close', onNativeClose)
      dialogEl.removeEventListener('cancel', onCancel)
      dialogEl.removeEventListener('click', onBackdropClick)
      closeBtn?.removeEventListener('click', onDismiss)
      sentinelStart?.removeEventListener('focus', onSentinelStartFocus)
      sentinelEnd?.removeEventListener('focus', onSentinelEndFocus)
      stopOpenSync()
      stopRoleSync()
      stopHeaderIdEffect()
      stopBodyIdEffect()
    }
  },
})
