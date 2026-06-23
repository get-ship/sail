// useForm — reactive form state with validation, submit, reset.
//
// Bundles a per-field state map (one signal per key from the typed
// `initial` shape), validation errors, dirty/pending tracking, and
// submit/reset handlers as a coherent controller-hook slice.
//
// Per-field signals: each key from `initial` becomes a state signal,
// readable as `form.<key>()` in templates and assignable as
// `form.<key> = v` in handlers — including via `bind:value={form.<key>}`.
// `form.values()` returns a snapshot of all fields (a plain object,
// re-built on each call inside its tracked scope).
//
// Errors: `form.errors()` returns a `Record<keyof T, string | null>`.
// `null` means no error; `string` is the error message. The keys
// always cover `keyof T`; missing entries from the user's `validate`
// fall back to `null`.
//
// Submit: `on:submit={form.submit}` on a `<form>` element. The handler
// calls `event.preventDefault()`, runs validation, bails on errors,
// and (if `submit` is provided) awaits it. Sets `form.pending = true`
// during the submit and `form.lastError = err` on failure.
//
// Reset: `on:click={form.reset}` rewrites every field signal back to
// the initial value.

import type { ControllerHook } from '../controller'

/**
 * Hook handlers see hook-internal state as a plain assignable record —
 * each hook-state key is an index. The form hook uses dynamic keys
 * (per `T`'s field set), so a simple `Record<string, unknown>` is the
 * right shape; the per-controller `MutableState<Scope>` type is for
 * the outer controller's writable proxy, not hook-internal state.
 */
type HookMutableState = Record<string, unknown>

export interface UseFormOptions<T extends Record<string, unknown>> {
  /** Initial values. Plain object or a thunk returning one. The keys
   * of the resolved object become the form's field signals. */
  initial: T | (() => T)
  /** Validation function. Returns a partial map of `field → error`.
   * Missing entries (or entries set to `null`/`undefined`) are treated
   * as "no error". Called inside a tracked scope so it auto-fires on
   * any field change. */
  validate?: (values: T) => Partial<Record<keyof T, string | null | undefined>>
  /** Submit handler. Receives the current values; may be async. The
   * hook awaits it, manages `pending`, captures errors as `lastError`. */
  submit?: (values: T) => void | Promise<void>
}

/** Resolve `initial` (T or thunk) into a plain values object. */
function resolveInitial<T extends Record<string, unknown>>(
  init: T | (() => T),
): T {
  return typeof init === 'function' ? (init as () => T)() : init
}

/** Snapshot the current values from a state proxy by walking the
 * known keys. Each `state[k]` read on the writable proxy returns the
 * current signal value; on the read-only view the same access also
 * returns the value (signal is callable). */
function snapshotValues<T extends Record<string, unknown>>(
  state: Record<string, unknown>,
  keys: string[],
): T {
  const out = {} as Record<string, unknown>
  for (const k of keys) {
    const v = (state as Record<string, unknown>)[k]
    out[k] = typeof v === 'function' ? (v as () => unknown)() : v
  }
  return out as T
}

/**
 * Reactive form state — fields, validation, dirty/pending, submit/reset.
 *
 * Returns a controller hook whose state shape is `T & FormMeta`, where
 * `FormMeta = { pending: boolean, lastError: Error | null }`.
 *
 * @typeParam T A plain record of field name → value type. Inferred from
 *   `opts.initial`.
 *
 * @example
 * ```ts
 * use: ({ ship, settings }) => ({
 *   form: useForm({
 *     initial:  () => settings(),
 *     validate: ({ accent }) => ({
 *       accent: /^#[0-9a-f]{6}$/i.test(accent) ? null : 'must be a hex color',
 *     }),
 *     submit:   (values) => ship.settings.write(values),
 *   }),
 * })
 * ```
 *
 * Template:
 * ```html
 * <form on:submit={form.submit}>
 *   <input bind:value={form.accent} />
 *   {#if form.errors().accent}
 *     <p>{form.errors().accent}</p>
 *   {/if}
 *   <button disabled={form.pending() || !form.dirty()}>save</button>
 *   <button type="button" on:click={form.reset}>reset</button>
 * </form>
 * ```
 */
export function useForm<T extends Record<string, unknown>>(
  opts: UseFormOptions<T>,
): ControllerHook<T & { pending: boolean; lastError: Error | null }> {
  const init = resolveInitial(opts.initial)
  const fieldKeys = Object.keys(init)

  const initialValues: Record<string, unknown> = {
    ...init,
    pending: false,
    lastError: null as Error | null,
  }

  return {
    state: initialValues as T & { pending: boolean; lastError: Error | null },

    derived: {
      values: (s) => snapshotValues<T>(s as Record<string, unknown>, fieldKeys),
      errors: (s) => {
        const values = snapshotValues<T>(s as Record<string, unknown>, fieldKeys)
        const validated = opts.validate ? opts.validate(values) : ({} as Partial<Record<keyof T, string | null | undefined>>)
        const out: Record<string, string | null> = {}
        for (const k of fieldKeys) {
          const v = validated[k as keyof T]
          out[k] = v === undefined || v === null ? null : v
        }
        return out
      },
      valid: (s) => {
        // Re-evaluate against the same snapshot for consistency.
        const values = snapshotValues<T>(s as Record<string, unknown>, fieldKeys)
        if (!opts.validate) return true
        const validated = opts.validate(values)
        for (const k of fieldKeys) {
          const v = validated[k as keyof T]
          if (v !== undefined && v !== null) return false
        }
        return true
      },
      dirty: (s) => {
        for (const k of fieldKeys) {
          const v = (s as Record<string, () => unknown>)[k]!()
          if (!Object.is(v, (init as Record<string, unknown>)[k])) return true
        }
        return false
      },
    },

    on: {
      reset: (s: HookMutableState) => {
        for (const k of fieldKeys) {
          s[k] = (init as Record<string, unknown>)[k]
        }
        s.pending = false
        s.lastError = null
      },

      submit: async (
        s: HookMutableState,
        event?: Event,
      ) => {
        // Preventing default on a form-submit Event is the canonical
        // gateway into JS-driven submit. Tolerate handler invocations
        // without an event (programmatic submit, tests).
        if (event && typeof event.preventDefault === 'function') {
          event.preventDefault()
        }
        const proxy = s

        // Bail on validation errors before pending=true.
        const values = snapshotValues<T>(proxy as Record<string, unknown>, fieldKeys)
        if (opts.validate) {
          const validated = opts.validate(values)
          for (const k of fieldKeys) {
            const v = validated[k as keyof T]
            if (v !== undefined && v !== null) return
          }
        }

        if (!opts.submit) return

        proxy.pending = true
        proxy.lastError = null
        try {
          await opts.submit(values)
        } catch (err) {
          proxy.lastError = err instanceof Error ? err : new Error(String(err))
        } finally {
          proxy.pending = false
        }
      },
    },
  } as ControllerHook<T & { pending: boolean; lastError: Error | null }>
}
