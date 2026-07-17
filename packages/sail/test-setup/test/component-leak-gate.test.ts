// COMPONENT LEAK GATE — the first real guardrail (G4).
//
// Runs the leak-budget harness over a representative set of checked-in sail-*
// primitives through their PRODUCTION mount/unmount lifecycle (compile via the
// native sail-compiler → register the custom element → connect → disconnect),
// asserting across many cycles that none of the three growth classes appear:
//
//   • NODE RETENTION  — detached roots left in the document (bodyDelta)
//   • REGISTRY GROWTH — append-only customElements.define per cycle (#258)
//   • CLOSURE/EFFECT RETENTION — the host removed but an undisposed effect /
//     subscription still pins it (WeakRef + FinalizationRegistry + forced GC)
//
// This is the gate that would catch a #258-class regrowth: if a primitive's
// `disconnectedCallback` stopped running its effect disposers, the retention
// signal flips red. It is jsdom/in-process only with bounded cycle counts — no
// daemon, no browser, no OOM surface (per the leak-arc safety constraint).
//
// ADD A PRIMITIVE: drop another entry in PRIMITIVES below. Each is a checked-in
// `.sail` under packages/components/sail/primitives/src; controller-less ones need
// no wiring. Keep CYCLES bounded.

import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { describe, expect, it, beforeEach, afterEach } from 'vitest'
import { compileSui, evalCompiledJs } from '@ship/sail-dom/test-utils'
import { measureMountLeakAsync, gcAvailable, type RetentionCycle } from '../src/leak-retention'

// Resolve the sui primitives package by walking up to the workspace root and
// landing on the package by NAME — NOT by counting `../` from this test file
// (the brittle traversal that broke the moment test-setup moved one dir).
//
// Why not `require.resolve('@ship/sail-primitives')`: it is not a declared
// dependency of @ship/test-setup, and bun's per-package node_modules layout
// leaves it unresolvable from here (verified: MODULE_NOT_FOUND) — declaring it
// would mean editing the lockfile. So we use the repo's OTHER sanctioned
// resolution pattern: walk to the `workspaces` root (the same heuristic as
// test-setup's own findRepoRoot). This survives THIS test moving, and confirms
// the package by its manifest `name` so a relocation of the primitives package
// fails loudly instead of with a silent wrong-path ENOENT.
function workspaceRoot(): string {
  let dir = __dirname
  for (let i = 0; i < 20; i++) {
    const manifest = resolve(dir, 'package.json')
    if (existsSync(manifest)) {
      try {
        const { workspaces } = JSON.parse(readFileSync(manifest, 'utf8')) as { workspaces?: unknown }
        if (workspaces != null) return dir
      } catch {
        /* malformed package.json — keep walking */
      }
    }
    const parent = dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  throw new Error(`[component-leak-gate] could not locate the workspaces root from ${__dirname}`)
}

function primitivesSrc(): string {
  const pkgDir = resolve(workspaceRoot(), 'packages/components/sail/primitives')
  const pkgJson = resolve(pkgDir, 'package.json')
  const name = existsSync(pkgJson)
    ? (JSON.parse(readFileSync(pkgJson, 'utf8')) as { name?: string }).name
    : undefined
  if (name !== '@ship/sail-primitives') {
    throw new Error(
      `[component-leak-gate] expected @ship/sail-primitives at ${pkgDir} but found ${name ?? '(missing)'} — ` +
        `update this resolver if the primitives package moved.`,
    )
  }
  return resolve(pkgDir, 'src')
}

const PRIM_ROOT = primitivesSrc()

// Bounded per the safety constraint: enough cycles that a per-cycle leak (1
// node/define each) is unmistakable, small enough to stay sub-second in jsdom.
const CYCLES = 30

interface Primitive {
  /** The checked-in `.sail` file, relative to PRIM_ROOT. */
  file: string
  /** A unique custom-element tag to register the compiled component under.
   *  Use a `-gate` suffix so the gate never collides with a real catalog tag
   *  another suite may have registered in the same worker. */
  tag: string
  /** Attributes to set on each mounted instance. */
  attrs?: Record<string, string>
}

// A handful of PURE sail-* primitives (controller-less: template + props + style
// only). They exercise the production component factory's connect/disconnect +
// reflect-effect lifecycle, which is exactly where an effect-retention leak
// would live. Data surfaces with engine controllers (sail-table, sail-calendar)
// are a deliberate next step — the pattern above is how to add them.
const PRIMITIVES: Primitive[] = [
  // sail-button, sail-separator dissolved to bare <button>/<hr> (L1 contract) —
  // no longer checked-in .sail files to gate.
  { file: 'sail-badge/sail-badge.sail', tag: 'sail-badge-gate', attrs: { tone: 'success' } },
  { file: 'sail-spinner/sail-spinner.sail', tag: 'sail-spinner-gate' },
]

/**
 * Build a bounded mount/unmount cycle for a primitive. The custom element is
 * compiled + registered ONCE (the legitimate define-once); each cycle only
 * creates → connects → disconnects an instance — exactly the production
 * lifecycle of a long-lived shell that mounts and unmounts the same component
 * repeatedly. The host node is handed back as the retention probe.
 */
function cycleFor(prim: Primitive): () => RetentionCycle {
  const source = readFileSync(resolve(PRIM_ROOT, prim.file), 'utf8')
  // Warmup: compile + eval once → `customElements.define(tag, …)`. Idempotent
  // in the runtime (defineComponent no-ops a duplicate tag), so the registry
  // settles after this one call — the define-count signal must read 0/cycle.
  const compiled = compileSui(source, { componentName: prim.tag, mode: 'component' })
  evalCompiledJs(compiled.js, undefined as never)

  return () => {
    const host = document.createElement(prim.tag)
    for (const [k, v] of Object.entries(prim.attrs ?? {})) host.setAttribute(k, v)
    document.body.appendChild(host) // connectedCallback → factory + reflect effects
    return {
      teardown: () => host.remove(), // disconnectedCallback → effect disposers
      probe: host,
    }
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})
afterEach(() => {
  document.body.innerHTML = ''
})

describe('component leak gate — sail-* primitives mount/unmount cleanly', () => {
  for (const prim of PRIMITIVES) {
    it(`${prim.tag} — no node / registry / effect-retention growth over ${CYCLES} cycles`, async () => {
      const r = await measureMountLeakAsync(cycleFor(prim), { iterations: CYCLES })

      // 1. NODE RETENTION — nothing detached left behind.
      expect(r.bodyDelta).toBe(0)
      // 2. REGISTRY GROWTH — define-once after warmup, registry does not grow.
      expect(r.definesPerIteration).toBeLessThan(1)
      // 3. CLOSURE/EFFECT RETENTION — every probed host collected after GC.
      //    Conclusive only with `global.gc` (config wires --expose-gc); if a CI
      //    runner ever drops it, this degrades to a documented no-op rather than
      //    a false green.
      expect(r.retainedProbed).toBe(CYCLES)
      if (!r.gcUnavailable) {
        expect(r.retainedAfterTeardown).toBe(0)
      }

      // Overall verdict — green across all three classes (or gc-inconclusive).
      expect(r.ok || r.gcUnavailable).toBe(true)
    })
  }

  it('the gate is running with a CONCLUSIVE retention signal (gc exposed)', () => {
    // Guards the guardrail: if vitest.config.ts ever loses `execArgv:
    // ['--expose-gc']`, the retention signal silently degrades to a no-op and
    // the #258 effect-retention regrowth this gate exists to catch would slip
    // through green. Fail loudly instead so the missing flag is fixed.
    expect(gcAvailable()).toBe(true)
  })
})

// NEGATIVE CONTROL — proves the gate is not vacuously green. A synthetic
// component whose "disconnect" removes the node but leaks an undisposed
// subscription (the #258 effect-retention class) MUST trip the gate's retention
// signal red. If this ever passed, the gate would be asleep.
describe('component leak gate — negative control (the gate has teeth)', () => {
  it('flags a component that removes its node but leaks an effect-subscription', async () => {
    if (!gcAvailable()) return // retention is inconclusive without gc; nothing to assert
    const leakedSubscriptions: object[] = []

    const r = await measureMountLeakAsync((): RetentionCycle => {
      const host = document.createElement('div')
      document.body.appendChild(host)
      // A subscription closure that captures the host and is NEVER disposed on
      // teardown — the exact shape a missing effect-disposer in
      // disconnectedCallback would produce.
      const subscription = () => void host.childElementCount
      leakedSubscriptions.push(subscription)
      return { teardown: () => host.remove(), probe: host }
    }, { iterations: CYCLES })

    // bodyDelta is clean (the node WAS removed) — only the retention signal
    // catches it, which is the whole point of extending the harness.
    expect(r.bodyDelta).toBe(0)
    expect(r.retainedAfterTeardown).toBeGreaterThan(0)
    expect(r.ok).toBe(false)
  })
})
