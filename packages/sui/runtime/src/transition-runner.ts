// Tiny indirection layer for the transition runner. Lets `for.ts` and
// `if.ts` call `runEnter` / `runLeave` / `cancelTransitions` without
// pulling the full ~3KB transition.ts (built-in fade/slide/scale,
// Web Animations API plumbing, prefers-reduced-motion checks).
//
// Apps that don't use `transition:` directives don't import
// transition.ts via the compiler's emitted code path. They also
// don't import `registerTransition` / `fade` / `slide` / `scale`
// from the runtime. So esbuild's tree-shaker drops transition.ts
// from the app bundle entirely — ~870B compressed in the Krausest
// bench bundle.
//
// When transition.ts IS imported (a template with a `transition:`
// directive, or an app calling `registerTransition`), its top-level
// invokes `setTransitionRunner(real)` to swap the stubs out. From
// then on bindFor/bindIf get the real animation runner.

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
