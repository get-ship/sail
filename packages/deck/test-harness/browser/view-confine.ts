// #164 slice 2 — confine a REAL compiled view in a real browser under lockdown.
// Slice 1 proved the mechanics on a toy controller; this proves the actual
// thing #165 4b does: take a real compiled view.js, endow the REAL runtime +
// dom + the real `document`, confine it under lockdown, and confirm it
// evaluates to the `{ default: { setup } }` shape mount() consumes — with
// ambient authority still denied. Result on `window.__viewConfine`.
//
// NOTE: a bundled runtime/dom instance is fine here — this tests confinement
// mechanics, not cross-bundle reactivity (the single-instance #149 concern is
// the LIVE shell's, handled in #165 via the importmap-shared instance).

import { buildViewEndowments, confineViewModule, lockdownRealm } from '../../host/src/index.ts'
import * as runtime from '../../../sail/runtime/src/index.ts'
import * as dom from '../../../sail/dom/src/index.ts'

declare const __GIT_VIEW_SOURCE__: string

interface ViewConfineResult {
  lockdown: 'ok' | string
  confined: 'ok' | string
  hasSetup: boolean
  defaultKeys: string[]
  ambientProbeDenied: boolean
}

function run(): ViewConfineResult {
  const result: ViewConfineResult = {
    lockdown: '',
    confined: '',
    hasSetup: false,
    defaultKeys: [],
    ambientProbeDenied: false,
  }

  try {
    lockdownRealm()
    result.lockdown = 'ok'
  } catch (e) {
    result.lockdown = e instanceof Error ? e.message : String(e)
    return result
  }

  // The endowment surface the live shell will build (#165 4b): real runtime +
  // dom namespaces, the real browser `document` (narrowed to node factories),
  // a stub ship. Hardened inside buildViewEndowments.
  const endow = buildViewEndowments({
    runtime: runtime as unknown as Record<string, unknown>,
    dom: dom as unknown as Record<string, unknown>,
    ship: {},
    doc: document,
  })

  try {
    const mod = confineViewModule(__GIT_VIEW_SOURCE__, endow)
    const def = (mod.default ?? {}) as Record<string, unknown>
    result.confined = 'ok'
    result.defaultKeys = Object.keys(def)
    result.hasSetup = typeof def.setup === 'function'
  } catch (e) {
    result.confined = e instanceof Error ? e.message : String(e)
  }

  // Confirm the SAME endowment surface still denies ambient authority — a probe
  // module evaluated against `endow` must not see fetch/window.
  try {
    const probe = confineViewModule(
      'export default { fetch: typeof fetch, win: typeof window }',
      endow,
    )
    const p = probe.default as unknown as { fetch: string; win: string }
    result.ambientProbeDenied = p.fetch === 'undefined' && p.win === 'undefined'
  } catch {
    result.ambientProbeDenied = false
  }

  return result
}

;(window as unknown as { __viewConfine: ViewConfineResult }).__viewConfine = run()
