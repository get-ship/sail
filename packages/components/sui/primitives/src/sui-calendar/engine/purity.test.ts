// TESTS-FIRST — Slice 0: the ENGINE-PURITY grep gate (copied from
// sui-table/engine/purity.test.ts, retargeted to sui-calendar/engine/).
//
// STATIC SCAN of the engine source (not an import): asserts the boundary holds
// (Cardinal 4 + the sui-* boundary). The engine is pure reactive state — no
// SDK, no network, no <data>. Engine purity is NOT lint-enforced (sui-lint
// targets .sui/.controller.ts and never sees these .ts modules) — it is
// TEST-asserted here. Authority: build-spec §"Backbone #1" + §Slices +
// behavioral spec #15.

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const ENGINE_DIR = dirname(fileURLToPath(import.meta.url))

function engineSourceFiles(): string[] {
  return readdirSync(ENGINE_DIR)
    .filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
    .map((f) => join(ENGINE_DIR, f))
}

const FORBIDDEN: { token: string; why: string }[] = [
  { token: '@ship/sdk-client', why: 'SDK import — the engine is pure reactive state' },
  { token: 'ship.', why: 'ship.* call — backend coupling belongs in the consumer controller' },
  { token: 'fetch(', why: 'network call — the engine never performs I/O' },
  { token: 'EventSource', why: "server-sent events — not the engine's concern" },
  { token: '<data', why: '<data> directive — a view/controller concern, never the engine' },
]

describe('engine purity (grep gate — Cardinal 4 + sui-* boundary)', () => {
  it('the engine has SOURCE modules to scan (precondition)', () => {
    expect(engineSourceFiles().length).toBeGreaterThan(0)
  })

  it('imports the runtime ONLY — no @ship/sdk-client anywhere under engine/', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      expect(src, `${file} must not import @ship/sdk-client`).not.toContain('@ship/sdk-client')
    }
  })

  it('contains NONE of the forbidden boundary tokens (ship.*/fetch/EventSource/<data)', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      for (const { token, why } of FORBIDDEN) {
        expect(src.includes(token), `${file} contains forbidden "${token}" — ${why}`).toBe(false)
      }
    }
  })

  it('every engine module using reactivity imports it from @ship/sail-runtime', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      // Match import/usage of the reactivity primitives (not just any prose).
      const usesReactivity = /\b(signal|derived|batch|untrack)\s*[(<]/.test(src)
      if (usesReactivity) {
        expect(src, `${file} uses reactivity but does not import @ship/sail-runtime`).toContain(
          '@ship/sail-runtime',
        )
      }
    }
  })
})
