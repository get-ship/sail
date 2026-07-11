import { controller, signal, useDialog } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

function notify(message: string, tone: string, description?: string) {
  window.dispatchEvent(
    new CustomEvent('ship:notify', {
      bubbles: true,
      detail: { message, tone, description, duration: 4000 },
    }),
  )
}

export default controller<Scope>({
  scope: () => ({
    // Overlays
    dialogDemo: useDialog(),
    alertDialogOpen: signal(false),
    drawerOpen: signal(false),
    popoverOpen: signal(false),
    dropdownOpen: signal(false),
    hoverCardOpen: signal(false),
    contextMenuOpen: signal(false),
    // Error boundary
    triggerError: signal(false),
    // Toast dispatch helpers — plain functions, not signals
    showSuccessToast: () => notify('File saved', 'success'),
    showErrorToast: () => notify('Upload failed', 'destructive', 'Network error — try again'),
    showInfoToast: () => notify('Background job started', 'info'),
  }),
})
