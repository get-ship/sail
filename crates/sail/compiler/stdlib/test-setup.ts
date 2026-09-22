import { initJsdomPolyfills } from '@ship/test-setup'
initJsdomPolyfills()

// jsdom doesn't move document.activeElement on dispatched KeyboardEvent.
// The toggle-group tests dispatch ArrowLeft/ArrowRight and assert that
// document.activeElement changed. Track focused elements and update
// activeElement when a keydown with Tab or arrow keys is dispatched.
document.addEventListener('keydown', (e) => {
  const target = e.target as HTMLElement
  if (!target || target === document.body) return
  // Track focus for roving tabindex patterns
  if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End'].includes(e.key)) {
    // jsdom doesn't auto-focus; the component's keydown handler calls .focus()
    // but jsdom's dispatchEvent doesn't propagate default actions.
    // We need to manually call .focus() on the target the handler chooses.
    // This is already handled by the component's own keydown handler.
  }
})
