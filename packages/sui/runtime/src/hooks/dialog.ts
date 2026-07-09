// useDialog — the blessed handle for a <dialog>'s open state.
//
// ── Hook doctrine (the mold every overlay hook follows) ──────────────────────
// A hook is the OFFICIAL API for its concern, not sugar over a "real"
// imperative one: the primitive underneath (`bindOpen`) is private, and an
// author never reaches past the handle for it. But hooks are BOUNDED — they
// exist only for a reusable, cohesive bundle of state + behavior + events that
// forms an interaction pattern with a clear handle (dialog, form, selection,
// combobox…). They are NOT React-style catch-alls: local state is `signal()`,
// derived is `derived()`, side-effects are `effect()`/lifecycle, async data is
// `<data>`/`{#async}`, structure is the template. If it isn't a reusable
// interaction pattern with a coherent handle, it isn't a hook.
// ─────────────────────────────────────────────────────────────────────────────
//
// The handle carries the whole programmatic surface of a dialog:
//   confirm.isOpen   — reactive open state
//   confirm.open()   — open it   (calls native showModal via bindOpen)
//   confirm.close()  — close it  (calls native close)
//   confirm.toggle() — flip it
// It is layer-invisible: `open()` rides the platform's showModal() today; the
// handle's surface never leaks that.

import { signal, effect } from '../signal'
import type { ControllerHook } from '../controller'
import type { Ref } from '../ref'
import { bindOpen, type DialogOptions } from '../enhance-dialog'

/**
 * Blessed handle for a native `<dialog>`'s open state.
 *
 * Bind the dialog element with `bind:this` and pass the ref in; the hook owns
 * the open signal and wires the native modal behavior (showModal/close,
 * close-sync, backdrop dismiss) via {@link bindOpen} when the element mounts.
 * The wiring is a plain `effect`, so it tears down with the controller.
 *
 * @param dialogRef Ref to the `<dialog>` element (`bind:this={dialogEl}`).
 * @param opts {@link DialogOptions} — e.g. `{ dismissible: false }` for a
 *   destructive confirm the user must answer.
 * @returns A `ControllerHook` resolving to `{ isOpen, open, close, toggle }`.
 *
 * @example
 * ```ts
 * // controller
 * scope: () => ({ dialogEl: ref(null) }),
 * use: ({ dialogEl }) => ({ confirm: useDialog(dialogEl, { dismissible: false }) }),
 * ```
 * ```html
 * <button on:click={confirm.open}>Delete…</button>
 * <dialog bind:this={dialogEl}>
 *   <header>Delete project?</header>
 *   <footer>
 *     <button on:click={confirm.close}>Cancel</button>
 *     <button variant="solid" tone="destructive"
 *       on:click={() => { doDelete(); confirm.close() }}>Delete</button>
 *   </footer>
 * </dialog>
 * ```
 */
export function useDialog(
  dialogRef: Ref<HTMLDialogElement>,
  opts: DialogOptions = {},
): ControllerHook<Record<string, never>> {
  // The hook OWNS the open signal — `isOpen` is exposed reactively, the
  // methods close over it, and bindOpen two-way-syncs it with the element.
  const isOpen = signal(false)

  // Wire the native behavior when the element mounts (ref fills). The effect
  // enrolls on the controller's mount scope, so its cleanup (bindOpen's
  // disposer) runs on unmount — no manual teardown.
  effect(() => {
    const el = dialogRef()
    if (el) return bindOpen(el, isOpen, opts)
  })

  return {
    derived: {
      isOpen: () => isOpen(),
    },
    on: {
      open: () => isOpen.set(true),
      close: () => isOpen.set(false),
      toggle: () => isOpen.set(!isOpen()),
    },
  }
}
