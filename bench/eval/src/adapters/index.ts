/**
 * Adapter phase dispatcher — used by eval-adapter CLI and in-process tests.
 */

import type { Phase, PhaseResult, RunManifest } from '../contract/types.ts'
import { emptyPhaseResult } from '../contract/types.ts'
import { preparePhase } from './prepare.ts'
import { gatePhase } from './gate.ts'
import { renderPhase } from './render.ts'
import { collectPhase } from './collect.ts'

export { preparePhase, gatePhase, renderPhase, collectPhase }

export async function runPhase(phase: Phase, manifest: RunManifest): Promise<PhaseResult> {
  switch (phase) {
    case 'prepare':
      return preparePhase(manifest)
    case 'gate':
      return gatePhase(manifest)
    case 'render':
      return renderPhase(manifest)
    case 'collect':
      return collectPhase(manifest)
    default: {
      const r = emptyPhaseResult(phase as Phase)
      r.error = { code: 'UNKNOWN_PHASE', message: `unknown phase: ${phase}` }
      return r
    }
  }
}

export const handlers = {
  prepare: preparePhase,
  gate: gatePhase,
  render: renderPhase,
  collect: collectPhase,
}
