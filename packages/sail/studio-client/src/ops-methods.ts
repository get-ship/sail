/**
 * Studio-facing unary method map derived from the generated client.
 *
 * Per-surface clients (`OpsClient`, `DesignOpsClient`, …) must be
 * `Pick`/`Partial<Pick>` of this map so METHOD_CONTRACTS changes break the
 * studio typecheck when a consumed method is added/removed.
 *
 * Signatures drop the optional `ClientOptions` second arg — surfaces bind opts
 * once in the adapter and call with args only.
 */
import type { ClientOptions } from './generated/client'

type Generated = typeof import('./generated/client')

/** Unary Connect helpers exported by the generated client (sail*). */
export type StudioOpsMethodName = {
  [K in keyof Generated]: K extends `sail${string}`
    ? // eslint-disable-next-line @typescript-eslint/no-explicit-any
      Generated[K] extends (args: any, opts?: ClientOptions) => Promise<any>
      ? K
      : never
    : never
}[keyof Generated]

/** Bound client shape: one arg (request body), Promise of ret. */
export type StudioOpsMethods = {
  [K in StudioOpsMethodName]: Generated[K] extends (
    args: infer A,
    opts?: ClientOptions,
  ) => Promise<infer R>
    ? (args: A) => Promise<R>
    : never
}

/**
 * Local FS/helpers that are not Connect methods. `writeSource` is the
 * check-guarded full-content path via `SailAuthoring/Edit` (`writeSource: true`);
 * `readSource` is Node/test only.
 */
export type StudioLocalOps = {
  writeSource?: (path: string, content: string) => Promise<void>
  readSource?: (path: string) => Promise<string>
}

/** Full core client + local helpers (edit surface / CLI adapter). */
export type StudioOpsClient = StudioOpsMethods & StudioLocalOps
