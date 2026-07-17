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
  withScope(scope, () =>
    withHost(host, () => {
      const setupRet = module.setup(ctx) as Record<string, unknown> | undefined
      const m = setupRet?.['__mount']
      if (typeof m === 'function') {
        unmount = (m as (h: HTMLElement) => (() => void) | void)(host)
      }
      // Ambient mount() callbacks the child's setup (or a composition it
      // called) registered on `scope` — run now that the child's DOM is
      // built. Disposers enroll on `scope` itself (see `runMounts`), so
      // they tear down with `scope.dispose()` below.
      scope.runMounts(host)
    }),
  )
  return () => {
    // `scope.dispose()` runs mount()-registered + onCleanup()-registered
    // teardown before the template's own DOM unmount.
    scope.dispose()
    try {
      if (typeof unmount === 'function') unmount()
    } catch (e) {
      console.error('[sui/compose] child unmount threw:', e)
    }
  }
}
