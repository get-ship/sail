// while-* boundaries — mount/unmount UI based on a LiveSignal's state.
//
// Both helpers follow `bindIf`'s shape: a Comment anchor marks the
// insertion point, an effect watches the signal, and the active
// branch's nodes get torn down on transition. Solid-style — once data
// resolves to ok, the loading boundary unmounts cleanly and stays
// unmounted unless the source re-fires.

import { effect } from './signal'
import type { Disposer } from './bind'
import type { BranchRender } from './if'
import type { LiveSignal } from './live-state'

/** Inputs to `bindWhileLoading`. */
export interface BindWhileLoadingOptions {
  /** Anchor placeholder; the loading branch's nodes go BEFORE it. */
  anchor: Comment
  /** The live signal whose `kind === 'loading'` triggers the branch. */
  signal: LiveSignal<unknown>
  /** Render the loading branch. */
  render: () => BranchRender
}

/** Inputs to `bindWhileError`. */
export interface BindWhileErrorOptions<E = Error> {
  /** Anchor placeholder; the error branch's nodes go BEFORE it. */
  anchor: Comment
  /** The live signal whose `kind === 'error'` triggers the branch. */
  signal: LiveSignal<unknown>
  /** Render the error branch — receives the error. */
  render: (error: E) => BranchRender
}

const tearDownBranch = (active: BranchRender | null): null => {
  if (!active) return null
  for (const n of active.nodes) {
    try { n.parentNode?.removeChild(n) } catch {}
  }
  active.cleanup?.()
  return null
}

const insertBranch = (anchor: Comment, br: BranchRender): void => {
  const parent = anchor.parentNode
  if (!parent) return
  for (const n of br.nodes) parent.insertBefore(n, anchor)
}

/**
 * Mount `render()` while `signal().kind === 'loading'`; unmount on
 * transition to ok or error. The branch re-mounts if the signal
 * returns to loading later (e.g. the source re-fires after a tracked
 * dep change).
 *
 * Compiler target: `{#whileLoading SIG}…{/whileLoading}`.
 *
 * @param opts Anchor, signal, and branch renderer.
 * @returns Disposer that tears down the active branch and stops watching.
 *
 * @example
 * ```ts
 * const agent = query(() => ship.agents.get(id()))
 * bindWhileLoading({
 *   anchor,
 *   signal: agent,
 *   render: () => ({ nodes: [skeleton()] }),
 * })
 * ```
 *
 * @see bindWhileError for the error-state companion; live for the
 *   signal source.
 */
export function bindWhileLoading(opts: BindWhileLoadingOptions): Disposer {
  const { anchor, signal: sig, render } = opts
  let active: BranchRender | null = null
  let mounted = false

  const dispose = effect(() => {
    const want = sig().kind === 'loading'
    if (want === mounted) return
    mounted = want
    if (want) {
      active = render()
      insertBranch(anchor, active)
    } else {
      active = tearDownBranch(active)
    }
  })

  return () => {
    dispose()
    active = tearDownBranch(active)
  }
}

/**
 * Mount `render(error)` while `signal().kind === 'error'`; unmount on
 * transition to ok or loading. The error value is passed to `render`
 * each time the branch is mounted.
 *
 * Compiler target: `{#whileError SIG as ERR}…{/whileError}`.
 *
 * @param opts Anchor, signal, and branch renderer (receives the error).
 * @returns Disposer that tears down the active branch and stops watching.
 *
 * @example
 * ```ts
 * bindWhileError({
 *   anchor,
 *   signal: agent,
 *   render: (err) => ({ nodes: [renderError(err)] }),
 * })
 * ```
 *
 * @see bindWhileLoading for the loading-state companion.
 */
export function bindWhileError<E = Error>(
  opts: BindWhileErrorOptions<E>,
): Disposer {
  const { anchor, signal: sig, render } = opts
  let active: BranchRender | null = null
  let mounted = false

  const dispose = effect(() => {
    const v = sig()
    const want = v.kind === 'error'
    if (want === mounted) return
    mounted = want
    if (want && v.kind === 'error') {
      active = render(v.error as unknown as E)
      insertBranch(anchor, active)
    } else {
      active = tearDownBranch(active)
    }
  })

  return () => {
    dispose()
    active = tearDownBranch(active)
  }
}
