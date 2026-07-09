// usePopover — the blessed handle for a native popover's open state.
//
// Same mold as useDialog (see hooks/dialog.ts for the full hook doctrine): an
// element-bound hook returns a USE-ABLE HANDLE — a callable action (so
// `<div use:menu>` attaches it) carrying the reactive API. It is the official,
// bounded surface for the popover/tooltip/dropdown/hover-card family; the
// primitive underneath (bindPopover) stays private. No ref, no effect on the
// authoring surface — the `use:` directive owns the element lifecycle.

import { signal } from '../signal'
import { bindPopover, type PopoverOptions } from '../enhance-popover'
import type { Action } from '../action'

/** The programmatic surface of a popover: the `use:` action + the open API. */
export interface PopoverHandle extends Action<void> {
  /** Reactive open state. */
  isOpen(): boolean
  /** Show it (native showPopover). */
  open(): void
  /** Hide it (native hidePopover). */
  close(): void
  /** Flip it. */
  toggle(): void
}

/**
 * Blessed handle for a native popover.
 *
 * Create it in the controller's `scope`, attach it in the template with
 * `use:`. The handle owns the open signal; `use:` hands it the element on
 * mount, where it wires {@link bindPopover} (showPopover/hidePopover +
 * toggle-sync) and tears down on unmount. No `ref`, no `effect`.
 *
 * @param opts {@link PopoverOptions} — e.g. `{ manual: true }` to disable
 *   light-dismiss.
 *
 * @example
 * ```ts
 * // controller
 * scope: () => ({ menu: usePopover() }),
 * ```
 * ```html
 * <button popovertarget on:click={menu.toggle}>Options</button>
 * <div use:menu>…</div>
 * ```
 */
export function usePopover(opts: PopoverOptions = {}): PopoverHandle {
  const isOpen = signal(false)
  const handle = ((node: Element) => {
    const dispose = bindPopover(node as HTMLElement, isOpen, opts)
    return { destroy: dispose }
  }) as PopoverHandle
  handle.isOpen = () => isOpen()
  handle.open = () => isOpen.set(true)
  handle.close = () => isOpen.set(false)
  handle.toggle = () => isOpen.set(!isOpen())
  return handle
}
