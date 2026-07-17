// The `TransitionRunner` seam: `runEnter` / `runLeave` / `cancelTransitions`
// / `setTransitionRunner` / `getTransitionRunner`. `if.ts` and `for.ts` call
// through this indirection when removing/inserting nodes, instead of
// depending on a concrete animation implementation directly.
//
// The seam is backed by `motion.ts`'s `motion:enter` / `motion:exit`
// lifecycle animations. Apps that don't use `motion:` directives never call
// `installLifecycleRunner()`, so the stub runners here (no-ops) stay in
// place and esbuild's tree-shaker drops `motion.ts` from the app bundle
// entirely. Apps that do use `motion:` have their compiled output call
// `installLifecycleRunner()`, which swaps the stubs out for the real
// lifecycle-animation runner — from then on bindFor/bindIf drive real
// enter/exit animations through this same seam.

type RunEnter = (nodes: Iterable<Node>) => void
type RunLeave = (nodes: Iterable<Node>) => Promise<void> | null
type CancelTransitions = (node: Node) => void

/** Stub: animations are opt-in by importing transition.ts. */
let _runEnter: RunEnter = () => {}
let _runLeave: RunLeave = () => null
let _cancelTransitions: CancelTransitions = () => {}

/** Re-exported by for.ts / if.ts. Stable references — they capture
 *  the live impl via closure so the swap takes effect immediately
 *  when transition.ts loads. */
export const runEnter: RunEnter = (nodes) => _runEnter(nodes)
export const runLeave: RunLeave = (nodes) => _runLeave(nodes)
export const cancelTransitions: CancelTransitions = (node) =>
  _cancelTransitions(node)

/** Called by transition.ts at module-init time to swap stubs for
 *  the real impl. Idempotent — re-importing transition.ts is a
 *  no-op (last writer wins, but the impls are the same). */
export function setTransitionRunner(impl: {
  runEnter: RunEnter
  runLeave: RunLeave
  cancelTransitions: CancelTransitions
}): void {
  _runEnter = impl.runEnter
  _runLeave = impl.runLeave
  _cancelTransitions = impl.cancelTransitions
}
