// Promise-driven branch render. Sail's compiler turns
// `{#await PROMISE}loading{:then VAL}then{:catch ERR}catch{/await}`
// into a call to bindAwait.
//
// Loading branch shows immediately; resolution swaps to then/catch.
// If the promise expression returns a NEW promise (e.g., the dep
// changes), the previous active branch is torn down and the loading
// branch renders again.

import { effect } from './signal'
import type { Disposer } from './bind'
import type { BranchRender } from './if'

/** Inputs to `bindAwait`. */
export interface BindAwaitOptions {
  /** Anchor placeholder; the active branch's nodes go BEFORE it. */
  anchor: Comment
  /** Reactive promise source — each new Promise restarts the cycle. */
  promise: () => Promise<unknown>
  /** Render the loading branch. Shown until `then_` or `catch_` resolves. */
  loading: () => BranchRender
  /** Render the resolved branch. Receives the resolved value. */
  then_?: (value: unknown) => BranchRender
  /** Render the rejected branch. Receives the rejection reason. */
  catch_?: (err: unknown) => BranchRender
}

/**
 * Promise-driven branch render. Shows the loading branch immediately,
 * then swaps to the then- or catch-branch when the current promise
 * settles. If the promise expression returns a new promise (e.g.
 * because a dep changed), any in-flight resolution is invalidated and
 * the loading branch renders again.
 *
 * Compiler target: `{#await PROMISE}loading{:then VAL}then{:catch ERR}catch{/await}`.
 *
 * @param opts Anchor, promise source, and branch renderers.
 * @returns Disposer that invalidates pending resolutions and tears down
 *   the active branch.
 *
 * @example
 * ```ts
 * const id = signal('agent-1')
 * bindAwait({
 *   anchor,
 *   promise: () => fetch(`/api/agents/${id()}`).then((r) => r.json()),
 *   loading: () => ({ nodes: [skeleton()] }),
 *   then_: (data) => ({ nodes: [renderAgent(data as Agent)] }),
 *   catch_: (err) => ({ nodes: [renderError(err)] }),
 * })
 * ```
 *
 * @see bindIf for synchronous branching.
 */
export function bindAwait(opts: BindAwaitOptions): Disposer {
  const { anchor, promise, loading, then_, catch_ } = opts
  let active: BranchRender | null = null
  let token = 0

  const tearDown = () => {
    if (!active) return
    for (const n of active.nodes) {
      try { n.parentNode?.removeChild(n) } catch {}
    }
    active.cleanup?.()
    active = null
  }

  const insert = (br: BranchRender) => {
    active = br
    const parent = anchor.parentNode
    if (!parent) return
    for (const n of br.nodes) parent.insertBefore(n, anchor)
  }

  const dispose = effect(() => {
    const myToken = ++token
    const p = promise()
    tearDown()
    insert(loading())
    p.then(
      (v) => {
        if (myToken !== token) return
        if (then_) {
          tearDown()
          insert(then_(v))
        }
      },
      (e) => {
        if (myToken !== token) return
        if (catch_) {
          tearDown()
          insert(catch_(e))
        }
      },
    )
  })

  return () => {
    token++ // invalidate any in-flight resolutions
    dispose()
    tearDown()
  }
}
