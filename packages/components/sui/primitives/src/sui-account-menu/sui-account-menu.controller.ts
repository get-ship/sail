// <sui-account-menu> controller — generic, SDK-free account affordance.
//
// The original @ship/chrome version derived identity from a live
// `ship.account.state()` subscription and called `navigate()` + the
// `@ship/sdk-client` account helpers inline. Here NONE of that exists: identity
// is plain props (status/name/email/image/plan/provider) and every action is an
// emitted event. The consuming app owns the subscription, the router, and the
// SDK calls. RUNTIME-FREE — imports `@ship/sui-runtime` only.

import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-account-menu.sui'

type PopoverEl = HTMLElement & {
  showPopover?: () => void
  hidePopover?: () => void
}

/** Two-letter initials from a display name (fallback "··"). */
function deriveInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (parts.length === 0) return '··'
  if (parts.length === 1) return parts[0]!.slice(0, 2).toUpperCase()
  return (parts[0]![0]! + parts[parts.length - 1]![0]!).toUpperCase()
}

export default controller({
  scope: ({ status, name, provider }: ViewScope<Scope>) => {
    const panelOpen = signal(false)

    const initials = derived(() => deriveInitials(name()))
    const triggerLabel = derived(() => {
      if (status() === 'signed-in') return name() !== '' ? name() : 'Account'
      if (status() === 'pending') return 'Completing…'
      return 'Sign in'
    })
    const triggerAria = derived(() => {
      const label = triggerLabel()
      return status() === 'signed-in' ? `Account: ${label}` : 'Sign in'
    })
    const signInLabel = derived(() =>
      provider() !== '' ? `Sign in with ${provider()}` : 'Sign in',
    )
    const disconnectedHint = derived(() =>
      provider() !== ''
        ? `Connect this workspace to ${provider()}`
        : 'Sign in to sync your workspace',
    )

    const triggerBtn = ref<HTMLButtonElement>(null)
    const panelEl = ref<HTMLDivElement>(null)
    const internals: { panel: PopoverEl | null; host: HTMLElement | null } = {
      panel: null,
      host: null,
    }

    function togglePanel(): void {
      const panel = internals.panel
      if (!panel) return
      if (panelOpen()) panel.hidePopover?.()
      else panel.showPopover?.()
    }
    function closePanel(): void {
      internals.panel?.hidePopover?.()
    }

    // Each action closes the panel, then emits — the app does the actual work
    // (router navigation, SDK calls). No coupling here.
    function fire(eventName: string): void {
      closePanel()
      internals.host?.dispatchEvent(
        new CustomEvent(eventName, { bubbles: true, composed: true }),
      )
    }

    return {
      panelOpen,
      initials,
      triggerLabel,
      triggerAria,
      signInLabel,
      disconnectedHint,
      triggerBtn,
      panelEl,
      togglePanel,
      onSignIn: () => fire('sui-account-signin'),
      onSignOut: () => fire('sui-account-signout'),
      onSettings: () => fire('sui-account-settings'),
      onBilling: () => fire('sui-account-billing'),
      _internals: internals,
    }
  },

  mount(state, host) {
    state._internals.host = host

    function anchorPanel(): void {
      const panel = state.panelEl
      const trigger = state.triggerBtn
      if (!panel || !trigger) return
      const r = trigger.getBoundingClientRect()
      panel.style.top = `${r.bottom + 4}px`
      panel.style.left = `${Math.max(8, r.right - panel.offsetWidth)}px`
    }

    const stopPanelSync = effect(() => {
      const panel = state.panelEl as PopoverEl | null
      state._internals.panel = panel
      if (!panel) return
      const onToggle = (e: Event) => {
        const open = (e as ToggleEvent).newState === 'open'
        if (state.panelOpen !== open) state.panelOpen = open
        if (open) anchorPanel()
      }
      panel.addEventListener('toggle', onToggle)
      return () => panel.removeEventListener('toggle', onToggle)
    })

    function onPanelKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        ;(state.panelEl as PopoverEl | null)?.hidePopover?.()
      }
    }
    const stopKeyBind = effect(() => {
      const panel = state.panelEl
      if (!panel) return
      panel.addEventListener('keydown', onPanelKeydown)
      return () => panel.removeEventListener('keydown', onPanelKeydown)
    })

    host.emit?.('sui-account-menu-ready', {})

    return () => {
      stopPanelSync()
      stopKeyBind()
    }
  },
})
