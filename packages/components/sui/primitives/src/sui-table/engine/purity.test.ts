// TESTS-FIRST — Backbone #1, Slice 4: the ENGINE-PURITY grep gate.
//
// Status: RED until the engine ships. This is a STATIC SCAN of the engine
// source (not an import of it), so its red is "no engine source files exist
// yet" — the precondition `engine source files are present` fails. Once the
// engine lands, the scan asserts the boundary holds (Cardinal 4 + the sui-*
// boundary): the engine is pure reactive state — no SDK, no network, no <data>.
//
// Authority: build-spec §"Backbone #1" ("Imports { signal, derived, batch,
// untrack } from @ship/sui-runtime ONLY — no @ship/sdk-client, no ship.*, no
// fetch, no <data>. It emits no DOM and makes no SDK calls"), §Slices ("Engine
// purity is NOT lint-enforced … Assert via a grep test"), behavioral spec #14
// ("a grep/test scan confirms no @ship/sdk-client/ship.*/fetch/EventSource/<data>
// in sui-table/ or engine/ … Test-asserted; not sui-lint-asserted").
//
// NOTE (per build-spec): engine purity is TEST-asserted here, NOT sui-lint —
// sui-lint targets .sui/.controller.ts and never sees these .ts modules.

import { readFileSync, readdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const ENGINE_DIR = dirname(fileURLToPath(import.meta.url))

// Every engine .ts EXCEPT the *.test.ts specs (tests legitimately name the
// forbidden tokens in comments/strings; the impl must not).
function engineSourceFiles(): string[] {
  return readdirSync(ENGINE_DIR)
    .filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
    .map((f) => join(ENGINE_DIR, f))
}

// The forbidden boundary tokens. Each is a substring scan over the source text.
const FORBIDDEN: { token: string; why: string }[] = [
  { token: '@ship/sdk-client', why: 'SDK import — the engine is pure reactive state' },
  { token: 'ship.', why: 'ship.* call — backend coupling belongs in the consumer controller' },
  { token: 'fetch(', why: 'network call — the engine never performs I/O' },
  { token: 'EventSource', why: 'server-sent events — not the engine\'s concern' },
  { token: '<data', why: '<data> directive — a view/controller concern, never the engine' },
]

describe('engine purity (Slice 4 grep gate — Cardinal 4 + sui-* boundary)', () => {
  it('the engine has SOURCE modules to scan (precondition — RED while greenfield)', () => {
    // While the engine is unimplemented this is empty → fails. Once Slice 1b+
    // ship the assembler + siblings, this passes and the scans below take over.
    expect(engineSourceFiles().length).toBeGreaterThan(0)
  })

  it('imports the runtime ONLY — no @ship/sdk-client anywhere under engine/', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      expect(src, `${file} must not import @ship/sdk-client`).not.toContain(
        '@ship/sdk-client',
      )
    }
  })

  it('contains NONE of the forbidden boundary tokens (ship.*/fetch/EventSource/<data)', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      for (const { token, why } of FORBIDDEN) {
        expect(src.includes(token), `${file} contains forbidden "${token}" — ${why}`).toBe(
          false,
        )
      }
    }
  })

  it('every engine module that imports the runtime imports it from @ship/sui-runtime', () => {
    for (const file of engineSourceFiles()) {
      const src = readFileSync(file, 'utf8')
      // If a module references signal/derived it must come from the runtime
      // barrel — not a relative re-implementation or a forbidden package.
      const importsReactivity = /\b(signal|derived|batch|untrack)\b/.test(src)
      if (importsReactivity) {
        expect(src, `${file} uses reactivity but does not import @ship/sui-runtime`).toContain(
          '@ship/sui-runtime',
        )
      }
    }
  })
})
