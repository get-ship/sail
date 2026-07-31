/** @ship/sail-eval — public API */

export type * from './types.ts'
export { parseJsonc, stripJsonc } from './jsonc.ts'
export { sanitizePath, sanitizeText, sanitizeJson, sanitizeNote } from './sanitize.ts'
export {
  evalRoot,
  repoRoot,
  tasksDir,
  armsDir,
  runsDir,
  findSailBin,
  assertSafeSlug,
} from './paths.ts'
export {
  listArms,
  loadArm,
  loadArms,
  materializeArm,
  findSailArtifactLeaks,
  gradingForArm,
  SAIL_ARTIFACT_PATTERNS,
  type ArmDef,
  type LoadedArm,
  type ArmContext,
} from './arm.ts'
export {
  runPaced,
  parseInterval,
  isTripleComplete,
  findCompletedTriple,
  modelsMatch,
  modelSlug,
  tripleRunDir,
  loadNonSailHtml,
  type PaceOptions,
} from './pace.ts'
export { loadTask, listTasks, loadCorpus } from './task.ts'
export {
  realHost,
  toWorkspacePath,
  parseJsonStdout,
  type HostTooling,
  type TreeNode,
  type CheckResult,
} from './host.ts'
export { scaffoldWorkdir, copyTreeInto } from './scaffold.ts'
export {
  createDispatch,
  RecordedDispatch,
  OmpDispatch,
  buildOmpArgs,
  buildDispatchPrompt,
  resolveModelLabel,
  type DispatchAdapter,
  type DispatchRequest,
} from './dispatch/index.ts'
export {
  gradeCompile,
  gradeVocab,
  gradeGates,
  gradeBehavior,
  gradeBeauty,
  runGradeChain,
  computeVocabShare,
  runScenariosOnHtml,
  collectSailFiles,
  DEFAULT_MIN_VOCAB_SHARE,
  STUDIO_LOW_WATER,
} from './graders/index.ts'
export { runCorpus } from './runner.ts'
export { classifyRun } from './classify.ts'
export { formatResultsMd, writeResults } from './report.ts'
export { recordSession } from './recorder.ts'
