// Main-thread Promise API over the worker. Use from any editor:
//
//   import { createSailLspClient } from '@ship/sail-lsp-browser'
//   const lsp = createSailLspClient(new Worker('/sail-lsp.worker.js', { type: 'module' }))
//   const diags = await lsp.diagnose(text)
//   const completions = await lsp.completionsAt(text, cursorOffset)
//   const hover = await lsp.hoverAt(text, cursorOffset)
//
// On editor focus or tutorial nav, call `lsp.preload()` once — it
// kicks off the lazy oxc-parser load so first-hover latency is near
// zero. Safe to call repeatedly; no-op after the first warm-up.
//
// One worker per editor instance is fine — the worker is stateless
// for the cheap endpoints; requests are matched by an incrementing id.

import type { SailDiagnostic } from './diagnostics'
import type { SailCompletionSource } from './completions'
import type { SailHoverInfo } from './hover'

export type { SailDiagnostic } from './diagnostics'
export type { SailCompletionSource } from './completions'
export type { SailHoverInfo } from './hover'

export const SAIL_BROWSER_LSP = {
  structure: true,
  typescript: false,
  note: 'TypeScript type-checking runs in native `sail lsp`. This worker is structure-only.',
} as const

interface Pending {
  resolve: (value: unknown) => void
  reject: (err: Error) => void
}

interface OkResponse<T> { id: number; ok: true; data: T }
interface ErrResponse    { id: number; ok: false; error: string }
type Response<T> = OkResponse<T> | ErrResponse

export interface SailLspClient {
  diagnose(text: string): Promise<SailDiagnostic[]>
  completionsAt(text: string, offset: number): Promise<SailCompletionSource | null>
  allSources(): Promise<SailCompletionSource[]>
  hoverAt(text: string, offset: number): Promise<SailHoverInfo | null>
  /** Warm up the lazy oxc-parser bundle for identifier-aware hover.
   *  Idempotent; first call eats the network cost (~500 KB gz), every
   *  later call returns the same cached promise. */
  preload(): Promise<boolean>
  dispose(): void
}

export function createSailLspClient(worker: Worker): SailLspClient {
  let nextId = 1
  const pending = new Map<number, Pending>()
  let preloadCall: Promise<boolean> | null = null

  worker.addEventListener('message', (event: MessageEvent<Response<unknown>>) => {
    const res = event.data
    const p = pending.get(res.id)
    if (!p) return
    pending.delete(res.id)
    if (res.ok) p.resolve(res.data)
    else p.reject(new Error(res.error))
  })

  function call<T>(payload: { kind: string; [k: string]: unknown }): Promise<T> {
    const id = nextId++
    return new Promise<T>((resolve, reject) => {
      pending.set(id, { resolve: resolve as (v: unknown) => void, reject })
      worker.postMessage({ id, ...payload })
    })
  }

  return {
    diagnose: (text) => call<SailDiagnostic[]>({ kind: 'diagnose', text }),
    completionsAt: (text, offset) => call<SailCompletionSource | null>({ kind: 'completeAt', text, offset }),
    allSources: () => call<SailCompletionSource[]>({ kind: 'allSources' }),
    hoverAt: (text, offset) => call<SailHoverInfo | null>({ kind: 'hoverAt', text, offset }),
    preload: () => {
      if (!preloadCall) preloadCall = call<boolean>({ kind: 'preload' })
      return preloadCall
    },
    dispose: () => {
      worker.terminate()
      for (const p of pending.values()) p.reject(new Error('worker disposed'))
      pending.clear()
    },
  }
}
