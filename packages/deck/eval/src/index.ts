/**
 * @ship/sui-eval — design-quality regression suite for .sui views.
 *
 * Given a feature description, the primitive catalog, and a behavioral
 * spec, dispatch to a model and assert the output is a working
 * `.sui` (+ optional setup `.ts`) that compiles, stays in scope,
 * and behaves correctly inside jsdom.
 *
 * @packageDocumentation
 */

import { buildPrompt, parseModelReply } from './prompt.ts'
import { dispatch } from './dispatch.ts'
import { loadCatalog } from './catalog.ts'
import { scoreOutput } from './score.ts'

export type { PrimitiveCatalog, PrimitiveDef, PropDef, SlotDef, EventDef } from './catalog.ts'
export { loadCatalog, parseCatalog } from './catalog.ts'
export { mockCatalog } from './mock-catalog.ts'
export { createMockShip } from './mock-ship.ts'
export type { MockShip } from './mock-ship.ts'
export { buildPrompt, parseModelReply, renderCatalog } from './prompt.ts'
export type { PromptInputs, PromptParts } from './prompt.ts'
export { dispatch } from './dispatch.ts'
export type { DispatchOptions, DispatchResult } from './dispatch.ts'
export { scoreOutput, sniffEscapeHatches } from './score.ts'
export type { ScoreInput, ScoreReport, SubScores } from './score.ts'

/** A single eval task. JSON-shaped so it can live next to other fixtures. */
export interface EvalTask {
  id: string
  difficulty: 'easy' | 'medium' | 'hard'
  /** Feature spec the model receives. */
  description: string
  /** Optional starter source — usually empty. */
  fixtures: { template: string; setup?: string }
  /** Identifiers we expect to see in template scope (best-effort hint). */
  expectedBindings?: string[]
  /**
   * JS function-body source. Receives `{ container, expect, tick, ship }`.
   * Throws on failure; returns/resolves on success. The `ship` arg is
   * the default mock ship from `createMockShip()`.
   */
  behavioralTests: string
}

export interface EvalResult {
  taskId: string
  model: string
  passed: boolean
  scores: {
    compiled: boolean
    boundedScope: boolean
    behavioral: boolean
  }
  output: { template: string; setup?: string }
  diff?: string
  durationMs: number
  diagnostics?: {
    compile?: string
    boundedScope?: string[]
    behavioral?: string
    dispatch?: string
  }
  usage?: {
    inputTokens: number
    outputTokens: number
    cacheReadInputTokens?: number
    cacheCreationInputTokens?: number
  }
}

export interface RunEvalOptions {
  model: string
  apiKey?: string
  /** Inject for tests; otherwise the live Anthropic Messages API is used. */
  fetchImpl?: typeof fetch
  baseUrl?: string
  maxTokens?: number
}

/** Run one eval end-to-end: prompt → dispatch → parse → score. */
export async function runEval(
  task: EvalTask,
  opts: RunEvalOptions,
): Promise<EvalResult> {
  const t0 = Date.now()
  const apiKey = opts.apiKey ?? process.env['ANTHROPIC_API_KEY']
  if (!apiKey) {
    throw new Error(
      'runEval: missing apiKey (pass opts.apiKey or set ANTHROPIC_API_KEY)',
    )
  }
  const catalog = await loadCatalog()
  const parts = buildPrompt({
    description: task.description,
    catalog,
    fixtures: task.fixtures,
  })

  let dispatchResult
  try {
    dispatchResult = await dispatch(parts, {
      model: opts.model,
      apiKey,
      ...(opts.fetchImpl ? { fetchImpl: opts.fetchImpl } : {}),
      ...(opts.baseUrl ? { baseUrl: opts.baseUrl } : {}),
      ...(opts.maxTokens !== undefined ? { maxTokens: opts.maxTokens } : {}),
    })
  } catch (err) {
    return failedResult(task, opts.model, t0, {
      dispatch: err instanceof Error ? err.message : String(err),
    })
  }

  let parsed
  try {
    parsed = parseModelReply(dispatchResult.text)
  } catch (err) {
    return failedResult(task, opts.model, t0, {
      dispatch: err instanceof Error ? err.message : String(err),
    })
  }

  const scoreInput: Parameters<typeof scoreOutput>[0] = {
    template: parsed.template,
    behavioralTests: task.behavioralTests,
    ...(parsed.setup !== undefined ? { setup: parsed.setup } : {}),
  }
  const report = await scoreOutput(scoreInput)
  const scores = {
    compiled: report.compiled,
    boundedScope: report.boundedScope,
    behavioral: report.behavioral,
  }
  const passed = scores.compiled && scores.boundedScope && scores.behavioral

  const result: EvalResult = {
    taskId: task.id,
    model: opts.model,
    passed,
    scores,
    output: parsed,
    durationMs: Date.now() - t0,
  }
  if (
    report.diagnostics.compile ||
    report.diagnostics.boundedScope ||
    report.diagnostics.behavioral
  ) {
    result.diagnostics = report.diagnostics
  }
  if (dispatchResult.usage) result.usage = dispatchResult.usage
  return result
}

function failedResult(
  task: EvalTask,
  model: string,
  t0: number,
  diagnostics: NonNullable<EvalResult['diagnostics']>,
): EvalResult {
  return {
    taskId: task.id,
    model,
    passed: false,
    scores: { compiled: false, boundedScope: false, behavioral: false },
    output: { template: '' },
    durationMs: Date.now() - t0,
    diagnostics,
  }
}
