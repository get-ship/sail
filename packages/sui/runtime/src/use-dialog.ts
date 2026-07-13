// useDialog — the handle for a native <dialog>'s modal state, as a plain
// composition. The handle is itself a `use:` action: bind it on the
// <dialog> element (`use:dlg`) and it wires bindOpen (showModal/close,
// close-sync, backdrop/ESC policy) for the element's lifetime; the methods
// carry the programmatic surface.
//
//   const dlg = useDialog()            // controller
//   <dialog use:dlg>…</dialog>         // template
//   dlg.open() / dlg.close() / dlg.toggle() / dlg.isOpen()

import { signal } from './signal'
import { bindOpen, type DialogOptions } from './enhance-dialog'

export interface DialogHandle {
  (el: Element): { destroy: () => void }
  isOpen: () => boolean
  open: () => void
  close: () => void
  toggle: () => void
}

export function useDialog(opts: DialogOptions = {}): DialogHandle {
  const openSig = signal(false)

  const action = ((el: Element) => {
    const dispose = bindOpen(el as HTMLDialogElement, openSig, opts)
    return { destroy: dispose }
  }) as DialogHandle

  action.isOpen = () => openSig()
  action.open = () => openSig.set(true)
  action.close = () => openSig.set(false)
  action.toggle = () => openSig.set(!openSig())
  return action
}
