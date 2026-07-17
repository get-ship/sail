// Bounded spawn helper for sail-compiler (and any other subprocess where a
// runaway must not take down the host). Wraps the target command in
// `prlimit --as=N timeout S …` so the child dies cleanly long before the
// kernel OOM-killer fires.
//
// A buggy `.sui` input once drove sail-compiler to 19+GB RSS before the OOM
// killer took down the surrounding WSL VM. The compiler now self-binds via
// setrlimit + a wall-clock watchdog, but every external caller still wraps
// its invocations through this helper so a single buggy compile never
// escapes its budget.
//
// Defaults: 2GB virtual memory, 30s wall clock. Override via opts.

import { spawnSync } from 'node:child_process';

const DEFAULT_MEM_BYTES = 2 * 1024 * 1024 * 1024;
const DEFAULT_TIMEOUT_SECS = 30;

/**
 * Spawn `cmd` with `args` under prlimit + timeout caps.
 *
 * @param {string} cmd - executable to run (resolved via PATH unless absolute)
 * @param {string[]} args - argv for the executable
 * @param {object} [opts]
 * @param {string} [opts.cwd] - working directory (default: process.cwd())
 * @param {number} [opts.timeoutSecs=30] - wall-clock cap in seconds
 * @param {number} [opts.memBytes] - virtual memory cap in bytes (default 2GB)
 * @param {string} [opts.encoding='utf8'] - stdio encoding
 * @param {*} [opts.stdio] - spawnSync stdio shape (default ['ignore','pipe','pipe'])
 * @param {object} [opts.env] - environment variables (defaults to process.env)
 * @returns {ReturnType<typeof spawnSync> & { capError: string | null }}
 *   - spawnSync result extended with `capError`: a human-readable string
 *     when a cap was hit (timeout, OOM), or null otherwise. `capError` is
 *     non-null whenever the child died because of a resource limit; status
 *     is still surfaced for any other non-zero exit.
 */
export function spawnBounded(cmd, args, opts = {}) {
  const {
    cwd = process.cwd(),
    timeoutSecs = DEFAULT_TIMEOUT_SECS,
    memBytes = DEFAULT_MEM_BYTES,
    encoding = 'utf8',
    stdio = ['ignore', 'pipe', 'pipe'],
    env = process.env,
  } = opts;

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

/**
 * Convenience: invoke spawnBounded and throw a labelled Error on any
 * non-zero exit. Caller supplies a `label` (typically the file/operation
 * being processed) so the thrown error is self-describing.
 *
 * @throws Error with `${label}: <reason>:\n<stderr>` when the child fails.
 */
export function spawnBoundedOrThrow(cmd, args, label, opts = {}) {
  const result = spawnBounded(cmd, args, opts);
  if (result.status === 0) return result;
  const reason = result.capError ?? `exit ${result.status}`;
  const detail = result.stderr || result.stdout || '(no output)';
  throw new Error(`${label}: ${reason}\n${detail}`);
}
