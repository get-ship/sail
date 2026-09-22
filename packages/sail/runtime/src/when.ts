// Structural viewport branch. The sail compiler turns
// `<when {tablet}>…</when><else>…</else>` into a call to bindWhen.
//
// Branch swap reuses bindIf (one engine for if/match/when). The media
// query is owned here: one shared matchMedia listener per distinct query
// string, refcounted across concurrent bindWhen sites. SSR / non-browser
// hosts report false (mobile-first default) so the else branch paints.

import { signal, type Signal } from './signal'
import { bindIf, type BranchRender, type BindIfTwoBranchOptions } from './if'
import type { Disposer } from './bind'

export interface BindWhenOptions {
  anchor: Comment
  /** Resolved media query for the theme breakpoint token. */
  query: string
  then_: () => BranchRender
  else_?: () => BranchRender
}

type SharedMq = {
  mql: MediaQueryList | null
  matches: Signal<boolean>
  count: number
  onChange: (() => void) | null
}

const shared = new Map<string, SharedMq>()

/** Test-only: drop shared listeners so fixture stubs re-bind cleanly. */
export function __resetMediaQueryCacheForTests(): void {
  for (const entry of shared.values()) {
    if (entry.mql && entry.onChange) {
      entry.mql.removeEventListener('change', entry.onChange)
    }
  }
  shared.clear()
}

/**
 * Shared matchMedia subscription for `query`. Returns a zero-arg signal
 * read of `.matches`. Call the disposer when the last consumer leaves.
 *
 * No `window.matchMedia` (SSR, workers): the signal stays `false` —
 * mobile-first SSR default; hydrate swaps when the client query matches.
 */
export function acquireMediaQuery(query: string): {
  matches: () => boolean
  release: () => void
} {
  let entry = shared.get(query)
  if (!entry) {
    const hasMq =
      typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    const mql = hasMq ? window.matchMedia(query) : null
    const matches = signal(mql ? mql.matches : false)
    let onChange: (() => void) | null = null
    if (mql) {
      onChange = () => matches.set(mql.matches)
      mql.addEventListener('change', onChange)
    }
    entry = { mql, matches, count: 0, onChange }
    shared.set(query, entry)
  }
  entry.count += 1
  const matches = entry.matches
  return {
    matches: () => matches(),
    release: () => {
      const cur = shared.get(query)
      if (!cur) return
      cur.count -= 1
      if (cur.count > 0) return
      if (cur.mql && cur.onChange) {
        cur.mql.removeEventListener('change', cur.onChange)
      }
      shared.delete(query)
    },
  }
}

/**
 * Render the matching branch for a viewport media query at `opts.anchor`.
 * Uses bindIf under the hood so branch cleanup / transitions stay one path.
 *
 * @returns Disposer that tears down the active branch and releases the
 * shared media listener when it is the last consumer.
 */
export function bindWhen(opts: BindWhenOptions): Disposer {
  const { anchor, query, then_, else_ } = opts
  const mq = acquireMediaQuery(query)
  const ifOpts: BindIfTwoBranchOptions = {
    anchor,
    cond: () => mq.matches(),
    then_,
    ...(else_ ? { else_ } : {}),
  }
  const disposeIf = bindIf(ifOpts)
  return () => {
    disposeIf()
    mq.release()
  }
}
