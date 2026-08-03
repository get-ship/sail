/**
 * Dirty leave / switch guard for Themes — Save / Discard / Cancel.
 * Pure state machine; session/shell wire navigate + save + discard.
 */
export type LeaveChoice = 'save' | 'discard' | 'cancel'

export type LeaveRequest = {
  /** Destination href (router) or empty when only switching theme id. */
  href: string | null
  /** Open this theme after leave resolves (switch). */
  openThemeId: string | null
  /** Human reason for tests / a11y. */
  reason: string
}

export type LeaveGuardHandlers = {
  isDirty: () => boolean
  save: () => Promise<boolean>
  discard: () => void
  navigate: (href: string) => void
  openTheme: (id: string) => void
  /** After clean leave with no href/open — e.g. free session. */
  onCleanLeave?: () => void
}

export type LeaveGuard = ReturnType<typeof createLeaveGuard>

export function createLeaveGuard(handlers: LeaveGuardHandlers) {
  let open = false
  let pending: LeaveRequest | null = null
  const listeners = new Set<() => void>()

  const emit = () => {
    for (const fn of listeners) fn()
  }

  const subscribe = (fn: () => void) => {
    listeners.add(fn)
    return () => {
      listeners.delete(fn)
    }
  }

  const snapshot = () => ({
    open,
    pending,
    reason: pending?.reason ?? '',
  })

  /**
   * Attempt navigation / theme switch.
   * Returns true if the action proceeded immediately (clean);
   * false if the dialog is now open (caller must wait for choice).
   */
  const requestLeave = (req: LeaveRequest): boolean => {
    if (!handlers.isDirty()) {
      complete(req)
      return true
    }
    pending = req
    open = true
    emit()
    return false
  }

  const complete = (req: LeaveRequest) => {
    if (req.href) handlers.navigate(req.href)
    if (req.openThemeId) handlers.openTheme(req.openThemeId)
    if (!req.href && !req.openThemeId) handlers.onCleanLeave?.()
  }

  const choose = async (choice: LeaveChoice): Promise<boolean> => {
    if (!open || !pending) return true
    const req = pending
    if (choice === 'cancel') {
      open = false
      pending = null
      emit()
      return false
    }
    if (choice === 'discard') {
      handlers.discard()
      open = false
      pending = null
      emit()
      complete(req)
      return true
    }
    // save
    const ok = await handlers.save()
    if (!ok) {
      // Stay open so user can discard/cancel after a failed save.
      emit()
      return false
    }
    open = false
    pending = null
    emit()
    complete(req)
    return true
  }

  const cancel = () => void choose('cancel')
  const discard = () => void choose('discard')
  const save = () => choose('save')

  /** Force-close without acting (e.g. unmount). */
  const reset = () => {
    open = false
    pending = null
    emit()
  }

  return {
    requestLeave,
    choose,
    cancel,
    discard,
    save,
    reset,
    subscribe,
    snapshot,
    isOpen: () => open,
    pending: () => pending,
  }
}
