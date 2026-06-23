import { controller, derived, effect, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-alert-dialog.sui'

type DialogEl = HTMLElement & { show(): void; close(): void }

export default controller({
  scope: ({ open, tone }: ViewScope<Scope>, host) => {
    const dialogRef = ref<HTMLElement>(null)

    const confirmTone = derived<'default' | 'destructive'>(() =>
      tone() === 'destructive' ? 'destructive' : 'default',
    )

    function emit(name: 'sui-confirm' | 'sui-cancel'): void {
      host?.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }))
      open.set(false)
    }

    function onConfirm(): void { emit('sui-confirm') }
    function onCancel(): void { emit('sui-cancel') }

    return { dialogRef, confirmTone, onConfirm, onCancel }
  },

  mount(state, host) {
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
