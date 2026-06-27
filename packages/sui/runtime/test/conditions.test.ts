// conditions.test.ts — the runtime target's behavioral conditions, executable.
//
// This suite is organized BY the conditions on the `runtime` target
// (helm framework-boundaries mission, scratch/target-runtime.md), so the
// intent doc's conditions are checkable here rather than asserted on paper.
//
// HONESTY POLICY: conditions we cannot prove today are `it.todo(...)` with the
// reason — they show as PENDING, never as green. We do not fake a gap closed to
// make the framework look more complete than it is.

import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { signal, derived, effect, batch } from '../src/signal'
import { mount, hostContext } from '../src/index'
import type { ViewPayload } from '../src/index'

// JSDOM has no EventSource; mount() can reference it on the live path. A no-live
// payload never uses it — stub minimally so construction is never the failure.
if (typeof (globalThis as { EventSource?: unknown }).EventSource === 'undefined') {
  ;(globalThis as { EventSource?: unknown }).EventSource = class {
    readyState = 1
    addEventListener(): void {}
    removeEventListener(): void {}
    close(): void {}
  }
}

// ── Condition 2 — a signal write updates every dependent exactly once, in one
//    synchronous pass, with no stale intermediate read. ───────────────────────
describe('runtime/2 — glitch-free, exactly-once propagation', () => {
  //        a
  //       / \
  //      b   c        b = a+1 , c = a*10
  //       \ /
  //        d          d = b + c   (depends on BOTH paths from a)
  function diamond() {
    const a = signal(1)
    const b = derived(() => a() + 1)
    const c = derived(() => a() * 10)
    const d = derived(() => b() + c())
    const seen: number[] = []
    effect(() => seen.push(d()))
    return { a, seen } // initial: a=1 → b=2, c=10, d=12
  }

  // GAP (condition 2 unmet by default): a bare write fans out a→b and a→c as
  // two notifications, so the effect runs once on the b-update with c stale
  // (3+10=13), then again at 23 → seen = [12, 13, 23]. Not glitch-free, and the
  // effect ran twice for one write. This `it.fails` pins that reality and flips
  // red (→ promote to a real assertion) the day the scheduler dedupes.
  it.fails('a bare diamond write updates the dependent exactly once with no stale read', () => {
    const { a, seen } = diamond()
    a.set(2)
    expect(seen).toEqual([12, 23])
  })

  // GAP, sharper: wrapping the write in batch() does NOT coalesce the diamond
  // fan-out either — still [12, 13, 23]. So this is a scheduler property, not a
  // missing batch boundary; glitch-freedom can't be reached from author code.
  it.fails('a batched diamond write is glitch-free and runs the effect once', () => {
    const { a, seen } = diamond()
    batch(() => a.set(2))
    expect(seen).toEqual([12, 23])
  })

  it('an equal-value write notifies nobody', () => {
    const s = signal(7)
    const seen: number[] = []
    effect(() => seen.push(s()))
    s.set(7)
    expect(seen).toEqual([7]) // initial run only
  })
})

// ── Condition 3 — effects/bindings created while mounted are disposed on
//    unmount; the author writes no cleanup. ────────────────────────────────────
describe('runtime/3 — disposal stops reactivity and runs cleanups', () => {
  it('disposing an effect halts re-runs and fires its registered cleanup', () => {
    const s = signal(0)
    const runs: number[] = []
    const cleanups: number[] = []

    const dispose = effect(() => {
      const v = s()
      runs.push(v)
      // The effect's returned function is its teardown — runs before the next
      // re-run and once on dispose (the mechanism component unmount relies on).
      return () => cleanups.push(v)
    })

    s.set(1) // re-run: cleanup(0) fires, then run records 1
    dispose() // cleanup(1) fires
    s.set(2) // disposed — no re-run, no new value observed

    expect(runs).toEqual([0, 1])
    expect(cleanups).toEqual([0, 1])
  })
})

// ── Condition 4 — the runtime names nothing host-shaped; a component runs with
//    NO host context supplied. (Proven positively — we never grep for "ship".) ─
describe('runtime/4 — host-agnostic mount', () => {
  it('reads the host seam safely when no host is registered', () => {
    // The runtime must not assume a host has plugged in — calling the seam
    // before any setHostContext() is safe.
    expect(() => hostContext()).not.toThrow()
  })

  it('mounts and renders a view with no ship/route/navigate supplied', async () => {
    const target = document.createElement('div')
    document.body.appendChild(target)
    const payload: ViewPayload = {
      name: 'host-free',
      html: '<p>hello</p>',
      css: '',
      scope: {},
      moduleUrl: '/x',
    }
    // No host context in opts — a bare module whose setup reads nothing.
    const handle = await mount(target, payload, {
      module: { default: { setup: () => ({}) } },
    })
    expect(target.querySelector('p')?.textContent).toBe('hello')
    handle.dispose()
    target.remove()
  })
})

// ── Condition 5 — the runtime carries no test-harness / host package as a
//    PRODUCTION dependency (the harness is devDep-only). ───────────────────────
describe('runtime/5 — clean production dependency surface', () => {
  it('does not declare @ship/test-setup (or any test harness) as a prod dependency', () => {
    const pkgPath = join(dirname(fileURLToPath(import.meta.url)), '..', 'package.json')
    const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as {
      dependencies?: Record<string, string>
    }
    const prod = Object.keys(pkg.dependencies ?? {})
    expect(prod).not.toContain('@ship/test-setup')
  })
})

// ── Gaps — conditions we cannot honestly prove today. Tracked, not faked. ──────
describe('runtime — open conditions (gaps)', () => {
  // Condition 1
  it.todo(
    'a built primitive ships no more JS than its lit equivalent — needs a size + lit-baseline harness',
  )
  // Condition 6
  it.todo(
    'an author reaches reactivity through one primitive family — the legacy controller<Scope> form still exists (e.g. sui-input)',
  )
  // Condition 7
  it.todo(
    'a derived computes identically on client and server — cross-runtime (Rust sui-signals) parity harness not built',
  )
})
