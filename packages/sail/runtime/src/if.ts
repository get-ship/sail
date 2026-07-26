// Conditional render. The sail compiler turns
// `{#if cond}then{:elif …}{:else}else{/if}` (and `{#match}`) into a
// call to bindIf.
//
// Anchor-based: a Comment placeholder marks the insertion point;
// branches are rendered lazily and swapped on selector change. Each
// branch gets its own dispose to tear down its bindings when swapped
// out (avoiding leaks from inactive branches' effects).
//
// Two input shapes are accepted, both ending in the same swap engine:
//
//   - Two-branch (the original API): `{ anchor, cond, then_, else_ }`.
//     `cond()` returns a boolean. Lives on for hand-rolled callers.
//
//   - Multi-branch (compiler emit): `{ anchor, test, branches }`.
//     `test()` returns the integer index of the branch to render
//     (`-1` selects nothing). The compiler emits this for `{#if}` —
//     including elif chains — and for `{#match}`. The else-branch (or
//     match's `default`) is appended to `branches` so the test
//     function can return its index instead of a sentinel.
//

import { effect } from './signal'
import { renderInScope } from './effect-scope'
import type { Disposer } from './bind'
import { runProtected } from './error-boundary'
import { runEnter, runLeave, cancelTransitions } from './transition-runner'

/**
 * One branch's rendered nodes plus an optional teardown. Returned by
 * the `then_` / `else_` / `loading` / branch-render callbacks of every
 * branching binding (`bindIf`).
 */
export interface BranchRender {
  /** Top-level nodes the branch contributes to the tree. */
  nodes: Node[]
  /** Optional teardown — called when the branch is swapped out. */
  cleanup?: () => void
}

/** Two-branch input shape — the original hand-author API. */
export interface BindIfTwoBranchOptions {
  anchor: Comment
  cond: () => boolean
  then_: () => BranchRender
  else_?: () => BranchRender
}

/**
 * Multi-branch input shape — the compiler emit for `{#if}` /
 * `{#match}`. `test()` returns the index of the branch in `branches`
 * to render, or `-1` for "render nothing".
 */
export interface BindIfMultiBranchOptions {
  anchor: Comment
  test: () => number
  branches: Array<() => BranchRender>
}

/** Public union — `bindIf` accepts either shape. */
export type BindIfOptions = BindIfTwoBranchOptions | BindIfMultiBranchOptions

/**
 * Conditionally render one of N branches at `opts.anchor`. Branches
 * are rendered lazily — only the selected branch's renderer is
 * invoked. The previous branch's `cleanup` runs on swap, freeing its
 * bindings.
 *
 * Compiler target: `{#if cond}then{:elif …}{:else}else{/if}` and
 * `{#match}`. The compiler always passes the multi-branch shape; the
 * two-branch shape is preserved for hand-rolled callers.
 *
 * @returns Disposer that tears down the active branch and stops the effect.
 *
 * @example two-branch (hand-rolled):
 * ```ts
 * const open = signal(false)
 * bindIf({
 *   anchor,
 *   cond: () => open(),
 *   then_: () => ({ nodes: [renderDialog()] }),
 *   else_: () => ({ nodes: [] }),
 * })
 * ```
 *
 * @example multi-branch (compiler emit):
 * ```ts
 * bindIf({
 *   anchor,
 *   test: () => kind() === 'ok' ? 0 : kind() === 'loading' ? 1 : -1,
 *   branches: [() => renderOk(), () => renderLoading()],
 * })
 * ```
 *
 * @see bindShow when the element should remain mounted (just hidden).
 */
export function bindIf(opts: BindIfOptions): Disposer {
  const { anchor } = opts
  // Normalize both shapes into a single (test, branches) pair so the
  // swap engine has one path. The two-branch shape collapses to
  // branches: [then_, else_?]; cond() → 0 picks then, 1 (when else_
  // exists) picks else, otherwise -1.
  const { test, branches } = isMulti(opts)
    ? { test: opts.test, branches: opts.branches }
    : twoBranchToMulti(opts)

  let active: BranchRender | null = null
  let lastIdx = -2 // sentinel: not -1 (which is a real "no match")

  // A branch whose out-transition is in flight. Kept in DOM until the
  // animation resolves; rescued back into `active` if the same branch
  // index is re-selected before removal. Without this, a rapid
  // visible→hidden→visible toggle would stall (waiting on out) or
  // double-remove (out completes after the new branch already
  // unmounted it).
  type Pending = { idx: number; nodes: Node[]; cleanup?: () => void }
  let pendingOut: Pending | null = null

  const removeNodes = (nodes: Iterable<Node>) => {
    for (const n of nodes) {
      try {
        n.parentNode?.removeChild(n)
      } catch {
        // already detached
      }
    }
  }

  const dispose = effect(() => {
    const idx = test()
    if (active && idx === lastIdx) return

    // Same branch index re-selected while its out-transition is
    // running: cancel the departure and re-promote the nodes. The
    // in-transition picks up from the current animated state instead
    // of restarting from the off-screen frame.
    if (pendingOut && idx === pendingOut.idx && idx >= 0 && idx < branches.length) {
      const promoted = pendingOut
      pendingOut = null
      for (const n of promoted.nodes) cancelTransitions(n)
      active = { nodes: promoted.nodes, ...(promoted.cleanup && { cleanup: promoted.cleanup }) }
      lastIdx = idx
      runEnter(active.nodes)
      return
    }

    // Tear down the currently-active branch. With transitions, the
    // nodes move to `pendingOut`; runLeave starts the out-transition;
    // removal + cleanup are deferred until the animation finishes.
    // Without transitions, runLeave returns null and we tear down
    // synchronously — preserving the old (transition-free) semantics.
    if (active) {
      const dep: Pending = {
        idx: lastIdx,
        nodes: active.nodes,
        ...(active.cleanup && { cleanup: active.cleanup }),
      }
      const leaving = runLeave(dep.nodes)
      active = null
      if (leaving === null) {
        removeNodes(dep.nodes)
        dep.cleanup?.()
      } else {
        pendingOut = dep
        leaving.then(() => {
          if (pendingOut !== dep) return // rescued or superseded
          removeNodes(dep.nodes)
          dep.cleanup?.()
          pendingOut = null
        })
      }
    }

    lastIdx = idx
    if (idx < 0 || idx >= branches.length) return
    // Render the branch in its OWN scope so its effects — including a nested
    // {#for}'s list effect — are owned by the branch's lifetime and disposed
    // on SWAP, not on every re-run of this bindIf effect. A same-branch re-run
    // (the guard above) must not tear the branch's nested effects down: they
    // live in the branch scope, not this effect's per-run scope. The scope's
    // disposer is folded into the branch cleanup, which every teardown path
    // already invokes.
    const [rendered, disposeBranch] = renderInScope(() =>
      runProtected(anchor, 'render', () => branches[idx]!()),
    )
    if (!rendered) {
      disposeBranch()
      return
    }
    active = {
      nodes: rendered.nodes,
      cleanup: () => {
        rendered.cleanup?.()
        disposeBranch()
      },
    }
    const parent = anchor.parentNode
    if (!parent) return
    for (const n of active.nodes) parent.insertBefore(n, anchor)
    // A branch that carries a <slot> re-entered the tree empty — the host's
    // captured light children still exist and must land in it NOW (renders
    // are synchronous in this runtime, so projection is too).
    if (
      active.nodes.some(
        (n) =>
          n.nodeType === 1 &&
          ((n as Element).tagName === 'SLOT' || (n as Element).querySelector('slot') !== null),
      )
    ) {
      const host = (parent as Element).closest?.('[data-component]') as
        | (HTMLElement & { __suiReproject?: () => void })
        | null
      host?.__suiReproject?.()
    }
    runEnter(active.nodes)
  })

  return () => {
    dispose()
    // Hard teardown — skip animations, remove everything synchronously.
    if (active) {
      removeNodes(active.nodes)
      active.cleanup?.()
      active = null
    }
    if (pendingOut) {
      removeNodes(pendingOut.nodes)
      pendingOut.cleanup?.()
      pendingOut = null
    }
  }
}

function isMulti(o: BindIfOptions): o is BindIfMultiBranchOptions {
  return typeof (o as BindIfMultiBranchOptions).test === 'function'
}

function twoBranchToMulti(
  o: BindIfTwoBranchOptions,
): { test: () => number; branches: Array<() => BranchRender> } {
  const { cond, then_, else_ } = o
  const branches: Array<() => BranchRender> = else_ ? [then_, else_] : [then_]
  return {
    test: () => (cond() ? 0 : else_ ? 1 : -1),
    branches,
  }
}
