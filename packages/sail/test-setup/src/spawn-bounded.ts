// Bounded spawn helper for sail-compiler (and any other subprocess where a
// runaway must not take down the host). Wraps the target command in
// `prlimit --as=N timeout S …` so the child dies cleanly long before the
// kernel OOM-killer fires.
//
// A buggy `.sail` input once drove sail-compiler to 19+GB RSS before the OOM
// killer took down the surrounding WSL VM. The compiler now self-binds via
// setrlimit + a wall-clock watchdog, but every external caller still wraps
// its invocations through this helper so a single buggy compile never
// escapes its budget.
//
// Defaults: 2GB virtual memory, 30s wall clock. Override via opts.

import { spawnSync, type SpawnSyncOptions } from 'node:child_process'

const DEFAULT_MEM_BYTES = 2 * 1024 * 1024 * 1024
const DEFAULT_TIMEOUT_SECS = 30

export type SpawnBoundedOpts = {
  cwd?: string
  timeoutSecs?: number
  memBytes?: number
  encoding?: BufferEncoding
  stdio?: SpawnSyncOptions['stdio']
  env?: NodeJS.ProcessEnv
}

/** Spawn `cmd` under prlimit + timeout so a runaway compile cannot take the host. */
export function spawnBounded(cmd: string, args: string[], opts: SpawnBoundedOpts = {}) {
  const {
    cwd = process.cwd(),
    timeoutSecs = DEFAULT_TIMEOUT_SECS,
    memBytes = DEFAULT_MEM_BYTES,
    encoding = 'utf8',
    stdio = ['ignore', 'pipe', 'pipe'],
    env = process.env,
  } = opts

  const wrapped = ['prlimit', `--as=${memBytes}`, 'timeout', String(timeoutSecs), cmd, ...args];
  const result = spawnSync(wrapped[0], wrapped.slice(1), { cwd, encoding, stdio, env });

  let capError = null;
  if (result.status === 124) {
    capError = `timed out after ${timeoutSecs}s`;
  } else if (result.signal === 'SIGKILL' || result.status === 137) {
    const mb = Math.round(memBytes / (1024 * 1024));
    capError = `exceeded ${mb}MB virtual memory cap`;
  }

  return { ...result, capError };
}

export function spawnBoundedOrThrow(
  cmd: string,
  args: string[],
  label: string,
  opts: SpawnBoundedOpts = {},
) {
  const result = spawnBounded(cmd, args, opts);
  if (result.status === 0) return result;
  const reason = result.capError ?? `exit ${result.status}`;
  const detail = result.stderr || result.stdout || '(no output)';
  throw new Error(`${label}: ${reason}\n${detail}`);
}
