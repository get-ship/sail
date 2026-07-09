// useDialog — the blessed handle for a native <dialog>'s open state.
//
// ── Hook doctrine (the mold every overlay hook follows) ──────────────────────
// A hook is the OFFICIAL API for its concern, not sugar over a "real"
// imperative one: the primitive underneath (`bindOpen`) is private, and an
// author never reaches past the handle for it. Hooks are BOUNDED — only for a
// reusable, cohesive interaction pattern with a clear handle (dialog, form,
// selection, combobox…). They are NOT React-style catch-alls: local state is
// `signal()`, derived is `derived()`, element behavior is `use:action`, async
// is `<data>`/`{#async}`. If it isn't a reusable interaction pattern with a
// coherent handle, it isn't a hook.
//
// SHAPE: an element-bound hook returns a USE-ABLE HANDLE — a callable action
// (so `<dialog use:confirm>` attaches it) that also carries the reactive API.
// This is distinct from data hooks (useForm/useSelection) which are declarative
// ControllerHooks; element-bound behavior is an action, so it composes with the
// `use:` directive instead. No `ref`, no `effect` on the authoring surface —
// the `use:` directive owns the element lifecycle.
// ─────────────────────────────────────────────────────────────────────────────

import { signal } from '../signal'
import { bindOpen, type DialogOptions } from '../enhance-dialog'
import type { Action } from '../action'

/**
 * The programmatic surface of a dialog. It IS the `use:` action (so
 * `<dialog use:confirm>` wires it), plus the reactive open API.
 */
export interface DialogHandle extends Action<void> {
  /** Reactive open state. */
  isOpen(): boolean
  /** Open the dialog (native showModal via bindOpen). */
  open(): void
  /** Close it (native close). */
  close(): void
  /** Flip it. */
  toggle(): void
}

/**
 * Blessed handle for a native `<dialog>`.
 *
 * Create it in the controller's `scope` and attach it in the template with
 * `use:`. The handle owns the open signal; the `use:` directive hands it the
 * element on mount, where it wires {@link bindOpen} (showModal/close,
 * close-sync, backdrop dismiss) and tears down automatically on unmount. No
 * `ref`, no `bind:this`, no hand-written `effect`.
 *
 * @param opts {@link DialogOptions} — e.g. `{ dismissible: false }`.
 * @returns A {@link DialogHandle}.
 *
 * @example
 * ```ts
 * // controller
 * scope: () => ({ confirm: useDialog({ dismissible: false }) }),
 * ```
 * ```html
 * <button on:click={confirm.open}>Delete…</button>
 * <dialog use:confirm>
 *   <header>Delete project?</header>
 *   <footer>
 *     <button on:click={confirm.close}>Cancel</button>
 *     <button variant="solid" tone="destructive"
 *       on:click={() => { doDelete(); confirm.close() }}>Delete</button>
 *   </footer>
 * </dialog>
 * ```
 */
export function useDialog(opts: DialogOptions = {}): DialogHandle {
  const isOpen = signal(false)
  // The handle IS the action: `use:confirm` calls it with the <dialog> node,
  // it wires the native behavior and returns the teardown.
  const handle = ((node: Element) => {
    const dispose = bindOpen(node as HTMLDialogElement, isOpen, opts)
    return { destroy: dispose }
  }) as DialogHandle
  handle.isOpen = () => isOpen()
  handle.open = () => isOpen.set(true)
  handle.close = () => isOpen.set(false)
  handle.toggle = () => isOpen.set(!isOpen())
  return handle
}
