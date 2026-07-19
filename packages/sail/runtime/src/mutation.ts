// wireFormMutation — compile-target helper for `<form mutation="ship.X.Y">`.
//
// Emitted by the SAIL compiler into compiled route modules as:
//   cleanups.push(wireFormMutation(formEl, "ship.agents.create", $scope))
//
// Behavior mirrors the attach-time wirer in ui-dom, but runs in the
// compiled module context where `$scope` is the reactive scope object.
// Intercepts the native submit, resolves the SDK chain, calls it with
// FormData, and exposes `$mutation: LiveSignal<unknown>` on `$scope`.

import { signal } from './signal'
import type { LiveState } from './live-state'

const toError = (err: unknown): Error =>
  err instanceof Error ? err : new Error(String(err))

function resolveChain(
  scope: Record<string, unknown>,
  chain: string,
): ((...args: unknown[]) => unknown) | undefined {
  const parts = chain.split('.')
  let cur: unknown = scope
  for (const part of parts) {
    if (cur == null || typeof cur !== 'object') return undefined
    cur = (cur as Record<string, unknown>)[part]
  }
  return typeof cur === 'function' ? (cur as (...args: unknown[]) => unknown) : undefined
}

/**
 * Wire a `<form mutation="...">` element in the compiled module context.
 *
 * Injects `$mutation` into `scope`, intercepts submit events, calls the
 * resolved SDK chain with `FormData`, and updates `$mutation` through
 * the `loading → ok | error` lifecycle. Fires `success` / `error`
 * `CustomEvent`s on the form for `on:success` / `on:error` handlers.
 *
 * @returns A cleanup disposer that removes the submit listener.
 */
export function wireFormMutation(
  el: Element,
  chain: string,
  scope: Record<string, unknown>,
): () => void {
  const mut = signal<LiveState<unknown>>({ kind: 'ok', value: undefined })
  scope['$mutation'] = mut

  const listener = async (e: Event): Promise<void> => {
    e.preventDefault()
    mut.set({ kind: 'loading' })

    const fn = resolveChain(scope, chain)
    if (!fn) {
      const error = new Error(`[ship-set] mutation chain "${chain}" not found in scope`)
      console.error(error)
      mut.set({ kind: 'error', error })
      el.dispatchEvent(new CustomEvent('error', { detail: { error }, bubbles: true }))
      return
    }

    const formData = el instanceof HTMLFormElement ? new FormData(el) : new FormData()
    try {
      const value = await Promise.resolve(fn(formData))
      mut.set({ kind: 'ok', value })
      el.dispatchEvent(new CustomEvent('success', { detail: { value }, bubbles: true }))
    } catch (err) {
      const error = toError(err)
      mut.set({ kind: 'error', error })
      el.dispatchEvent(new CustomEvent('error', { detail: { error }, bubbles: true }))
    }
  }

  el.addEventListener('submit', listener as EventListener)
  return () => el.removeEventListener('submit', listener as EventListener)
}
