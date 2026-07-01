import { controller, derived, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-alert-dialog.sui'

type DialogEl = HTMLElement & { show(): void; close(): void }

export default controller({
  scope: ({ open, tone }: ViewScope<Scope>, _host?: Element) => {
    const dialogRef = ref<HTMLElement>(null)

    const confirmTone = derived<'default' | 'destructive'>(() =>
      tone() === 'destructive' ? 'destructive' : 'default',
    )

    // The host element is not present in the scope factory (the post-
    // convergence "SUI names no host" compile stops passing it as the 2nd
    // arg), so read it through this signal — set in `mount` once the
    // runtime hands over the real element. The template wires
    // `on:click={onConfirm}`/`{onCancel}` so the emit must read the host
    // from a signal, not the dead scope `host` arg.
    const hostEl = signal<HTMLElement | null>(null)

    function emit(name: 'sui-confirm' | 'sui-cancel'): void {
      hostEl()?.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
      open.set(false)
    }

    function onConfirm(): void { emit('sui-confirm') }
    function onCancel(): void { emit('sui-cancel') }

    return { dialogRef, confirmTone, onConfirm, onCancel, hostEl }
  },

  mount(state, host) {
    // The runtime hands the real host element here. The scope's `emit`
    // closure reads through `hostEl` (the dead-`host` workaround) — publish
    // the element now that it exists, before any user click can fire.
    state.hostEl = host as HTMLElement
    // Drive inner sui-dialog open state from our own.
    const stopOpenSync = effect(() => {
      const wantOpen = state.open
      const dlg = state.dialogRef as DialogEl | null
      if (!dlg) return
      if (wantOpen && typeof dlg.show === 'function') dlg.show()
      else if (!wantOpen && typeof dlg.close === 'function') dlg.close()
    })

    // When the inner sui-dialog closes (ESC, backdrop, programmatic),
    // mirror that into our own open=false. The event bubbles through
    // composition to our host.
    const onDialogClose = () => {
      if (state.open) state.open = false
    }
    host.addEventListener('sui-close', onDialogClose)

    return () => {
      stopOpenSync()
      host.removeEventListener('sui-close', onDialogClose)
    }
  },
})
