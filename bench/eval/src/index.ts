/**
 * @ship/sail-bench — HARNESS-2 public surface.
 */

export { CONTRACT_VERSION, type RunManifest, type PhaseResult, type Phase } from './contract/types.ts'
export { invokePhase } from './contract/invoke.ts'
export { LocalRunner } from './runner/local.ts'
export { RunStore } from './store/db.ts'
export { writeBatchReport } from './report/batch.ts'
export { writeGrades } from './grade/index.ts'
export { allocateWordPair, isWordPairId } from './naming/word-pair.ts'
export { HARNESS_VER } from './config/types.ts'
export { runPhase, handlers } from './adapters/index.ts'
