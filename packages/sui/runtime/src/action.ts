// action.ts — the runtime half of the `use:` directive.
//
// An "action" is the first-class element→behavior primitive: a function that
// receives the rendered element (and an optional reactive param) when it
// mounts, and returns an optional `{ update, destroy }`. `runAction` wires the
// lifecycle — call on mount, `update(param)` when the param's dependencies
// change, `destroy()` on unmount. This is what the compiler emits for
// `use:action` / `use:action={param}`, and it is the thing that removes
// author-facing `ref` + `effect` for the whole element-behavior category:
// autosize, click-outside, roving-tabindex, anchor-positioning, and — via
// useDialog — the overlay family, all become reusable actions.

import { effect } from './signal'
import { onCleanup } from './effect-scope'

/** What an action may return to hook into param changes and teardown. */
export interface ActionResult<P = unknown> {
  /** Called when the `use:action={param}` param's dependencies change. */
  update?(param: P): void
  /** Called once when the element unmounts. Tear down whatever was set up. */
  destroy?(): void
}

/** A reusable element behavior. Receives the node + optional param on mount. */
export type Action<P = unknown> = (
  node: Element,
  param: P,
) => ActionResult<P> | void

/**
 * Attach an action to an element with full lifecycle. Compiler target for
 * `use:action` (no param) and `use:action={param}` (reactive param).
 *
 * - On mount: `action(node, param)`.
 * - When `paramThunk`'s tracked deps change: `result.update(param)`.
 * - On unmount: `result.destroy()`.
 *
 * A param-less `use:` runs the action exactly once (no reactive deps). The
 * setup effect enrolls on the surrounding mount scope, so it — and the
 * `destroy` cleanup — tear down when the component unmounts.
 *
 * @param node The rendered element the action attaches to.
 * @param action The action function (or a hook handle that *is* one). A
 *   non-function is a no-op — the directive stays inert rather than throwing.
 * @param paramThunk Optional reactive read of the action's parameter.
 */
export function runAction<P>(
  node: Element,
  action: Action<P> | null | undefined,
  paramThunk?: () => P,
): void {
  if (typeof action !== 'function') return
  let result: ActionResult<P> | void
  let started = false
  effect(() => {
    const param = (paramThunk ? paramThunk() : undefined) as P
    if (!started) {
      started = true
      result = action(node, param)
    } else if (result) {
      result.update?.(param)
    }
  })
  onCleanup(() => {
    if (result) result.destroy?.()
  })
}
