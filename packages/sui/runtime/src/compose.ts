// compose.ts — mount a component into a host node (child composition).
//
// The mount-FORM of a component (`component(setup)`, which returns `{ setup }`)
// is how SUI composes a component WITHOUT registering it as a custom element:
// an internal/private part (an underscore-named `.sui` the compiler resolves
// and mounts inside its parent's render) rather than a published custom element
// the browser upgrades by tag. Registration is an EXPORT decision; composition
// is this.
//
// `mountComponent` is the primitive the compiled parent calls at the child's
// position: it runs the child's `setup(ctx)`, mounts its DOM under `host`, and
// owns the child's effects in a CHILD scope so they tear down when the parent
// unmounts — or when the returned disposer fires, whichever comes first.
//
// SSR/hydration adopt (the `hydrate(root, ctx)` half) is a later slice; this is
// the CSR build path: `setup → __mount`.

import type { MountModule } from './component-mount'
import { createChildScope, withScope } from './effect-scope'
import { withHost } from './signal'
import { LIFECYCLE_KEY, type ControllerLifecycle } from './controller-resolver'
import { componentMountState } from './state-proxy'

/**
 * Mount a component module into `host`, returning a disposer.
 *
 * @param module The compiled component's `{ setup }` default export (mount-form).
 * @param host   The element the child renders into — the parent creates it at
 *               the child's position in its own template.
 * @param ctx    The mount context the child's `setup` reads (ship/clients/route/
 *               navigate + any props the parent passes). Shaped by the caller.
 * @returns A disposer that unmounts the child's DOM and disposes its scope.
 *          Idempotent (the scope's own dispose guard).
 */
export function mountComponent(
  module: MountModule,
  host: HTMLElement,
  ctx: unknown,
): () => void {
  // A child scope owned by the parent's current scope: the child's effects /
  // live subscriptions / hooks dispose automatically when the parent unmounts,
  // and also when the returned disposer fires (whichever happens first). Outside
  // any parent scope (tests, standalone) it's simply an unowned scope — the
  // returned disposer is then the only teardown path, which is correct.
  const scope = createChildScope()
  let unmount: (() => void) | void
  // Controller lifecycle (mount/unmount) cleanups — kept separate from the
  // template `__mount` disposer so teardown order is: lifecycle unmount →
  // DOM teardown → scope dispose.
  const lifecycleCleanups: Array<() => void> = []
  withScope(scope, () =>
    withHost(host, () => {
      const setupRet = module.setup(ctx) as Record<string, unknown> | undefined
      if (!setupRet) return
      const m = setupRet['__mount']
      if (typeof m === 'function') {
        unmount = (m as (h: HTMLElement) => (() => void) | void)(host)
      }
      // Wire the controller's mount()/unmount() if its setup attached a
      // lifecycle bundle under LIFECYCLE_KEY. This is the SAME wiring the
      // compiled component path did inline (`componentMountState` + `mount(state,
      // host)`); centralizing it here makes `mountComponent` the one engine both
      // registration and internal composition call. Read via descriptor since
      // the bundle key may be non-enumerable.
      const lc = Object.getOwnPropertyDescriptor(setupRet, LIFECYCLE_KEY)?.value as
        | ControllerLifecycle
        | undefined
      if (lc) {
        const state = componentMountState(setupRet) as Record<string, unknown>
        if (lc.mount) {
          const mc = lc.mount(state, host)
          if (typeof mc === 'function') lifecycleCleanups.push(mc)
        }
        if (lc.unmount) {
          const unmountFn = lc.unmount
          lifecycleCleanups.push(() => unmountFn(state))
        }
      }
    }),
  )
  return () => {
    for (const c of lifecycleCleanups.splice(0)) {
      try {
        c()
      } catch (e) {
        console.error('[sui/compose] lifecycle cleanup threw:', e)
      }
    }
    try {
      if (typeof unmount === 'function') unmount()
    } catch (e) {
      console.error('[sui/compose] child unmount threw:', e)
    }
    scope.dispose()
  }
}
