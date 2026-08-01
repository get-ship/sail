/**
 * Core → adapter subprocess spawn for RUN CONTRACT v1.
 */

import { spawn } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  CONTRACT_VERSION,
  type Phase,
  type PhaseResult,
  type RunManifest,
  checkContractVersion,
  emptyPhaseResult,
  isPhase,
} from './types.ts'

const HERE = dirname(fileURLToPath(import.meta.url))
/** Default adapter entry (bun-runnable). Override via SAIL_EVAL_ADAPTER. */
export const DEFAULT_ADAPTER_PATH = join(HERE, '../cli/eval-adapter.ts')

export interface InvokeOptions {
  /** Absolute path to eval-adapter entry (ts or bin). */
  adapterPath?: string
  /** bun/node binary. Default: bun. */
  runtime?: string
  timeoutMs?: number
  /** When true, call phase handler in-process (tests). */
  inProcess?: boolean
  /** In-process handler map (required when inProcess). */
  handlers?: Partial<Record<Phase, (m: RunManifest) => Promise<PhaseResult>>>
}

export interface InvokeOutcome {
  result: PhaseResult
  exitCode: number | null
  stderr: string
  timedOut: boolean
  contractError: boolean
}

/**
 * Invoke one adapter phase with the run manifest on stdin.
 * Never throws for adapter failures — returns structured outcome.
 */
export async function invokePhase(
  phase: Phase,
  manifest: RunManifest,
  opts: InvokeOptions = {},
): Promise<InvokeOutcome> {
  if (!isPhase(phase)) {
    return {
      result: {
        ...emptyPhaseResult(phase),
        error: { code: 'UNKNOWN_PHASE', message: `unknown phase: ${phase}` },
      },
      exitCode: 2,
      stderr: '',
      timedOut: false,
      contractError: true,
    }
  }

  const verErr = checkContractVersion(manifest.contract_version)
  if (verErr) {
    return {
      result: {
        ...emptyPhaseResult(phase),
        error: { code: 'VERSION', message: verErr },
      },
      exitCode: 2,
      stderr: '',
      timedOut: false,
      contractError: true,
    }
  }

  const body: RunManifest = { ...manifest, phase }

  if (opts.inProcess) {
    const handler = opts.handlers?.[phase]
    if (!handler) {
      return {
        result: {
          ...emptyPhaseResult(phase),
          error: { code: 'NO_HANDLER', message: `no in-process handler for ${phase}` },
        },
        exitCode: 1,
        stderr: '',
        timedOut: false,
        contractError: false,
      }
    }
    const t0 = Date.now()
    try {
      const result = await handler(body)
      return {
        result,
        exitCode: 0,
        stderr: '',
        timedOut: false,
        contractError: false,
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      return {
        result: {
          ...emptyPhaseResult(phase, Date.now() - t0),
          error: { code: 'ADAPTER_CRASH', message: msg },
        },
        exitCode: 1,
        stderr: msg,
        timedOut: false,
        contractError: false,
      }
    }
  }

  const runtime = opts.runtime ?? process.env.SAIL_BENCH_RUNTIME ?? 'bun'
  const adapterPath = opts.adapterPath ?? process.env.SAIL_EVAL_ADAPTER ?? DEFAULT_ADAPTER_PATH
  const timeoutMs =
    opts.timeoutMs ?? body.limits?.phase_timeout_ms ?? 600_000

  return spawnAdapter(runtime, adapterPath, phase, body, timeoutMs)
}

function spawnAdapter(
  runtime: string,
  adapterPath: string,
  phase: Phase,
  body: RunManifest,
  timeoutMs: number,
): Promise<InvokeOutcome> {
  return new Promise((resolve) => {
    const child = spawn(runtime, [adapterPath, phase], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: { ...process.env },
    })

    let stdout = ''
    let stderr = ''
    let timedOut = false
    const timer = setTimeout(() => {
      timedOut = true
      child.kill('SIGTERM')
      setTimeout(() => {
        try {
          child.kill('SIGKILL')
        } catch {
          /* already dead */
        }
      }, 2_000).unref?.()
    }, timeoutMs)

    child.stdout?.setEncoding('utf8')
    child.stderr?.setEncoding('utf8')
    child.stdout?.on('data', (c: string) => {
      stdout += c
    })
    child.stderr?.on('data', (c: string) => {
      stderr += c
    })

    child.on('error', (err) => {
      clearTimeout(timer)
      resolve({
        result: {
          ...emptyPhaseResult(phase),
          error: { code: 'SPAWN', message: err.message },
        },
        exitCode: 1,
        stderr,
        timedOut: false,
        contractError: false,
      })
    })

    child.on('close', (code) => {
      clearTimeout(timer)
      if (timedOut) {
        resolve({
          result: {
            ...emptyPhaseResult(phase),
            error: { code: 'ADAPTER_TIMEOUT', message: `phase ${phase} exceeded ${timeoutMs}ms` },
            flags: ['adapter_timeout'],
          },
          exitCode: code,
          stderr,
          timedOut: true,
          contractError: false,
        })
        return
      }

      const contractError = code === 2
      let result: PhaseResult
      try {
        const parsed = JSON.parse(stdout.trim() || '{}') as PhaseResult
        result = {
          ...emptyPhaseResult(phase),
          ...parsed,
          phase,
          contract_version: parsed.contract_version ?? CONTRACT_VERSION,
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        result = {
          ...emptyPhaseResult(phase),
          error: {
            code: code === 2 ? 'CONTRACT' : 'MALFORMED_STDOUT',
            message: `failed to parse adapter stdout: ${msg}; raw=${stdout.slice(0, 200)}`,
          },
          flags: ['malformed_adapter_stdout'],
        }
      }

      if (code === 1 && !result.error) {
        result = {
          ...result,
          ok: false,
          verdict: 'error',
          error: {
            code: 'ADAPTER_CRASH',
            message: stderr.trim() || `adapter exited ${code}`,
          },
          flags: [...(result.flags ?? []), 'adapter_crash'],
        }
      }

      resolve({
        result,
        exitCode: code,
        stderr,
        timedOut: false,
        contractError,
      })
    })

    child.stdin?.write(JSON.stringify(body))
    child.stdin?.end()
  })
}
