import { signal, derived, batch } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'

export interface FormOptions<T extends Record<string, unknown>> {
  /** Initial field values. A function is re-read on `reset()`, so a form can
   *  reset back to *current* server state rather than to a stale snapshot. */
  initial: T | (() => T)
  /** Per-field errors. Return `null` for a field that is fine. */
  validate?: (values: T) => Partial<Record<keyof T, string | null>>
  /** Called by `submit()` when the form is valid. May be async. */
  submit?: (values: T) => unknown | Promise<unknown>
}

/**
 * `form({ initial, validate, submit })` — field signals, validation, submit state.
 *
 * Each field becomes its own `Signal`, so `bind:value={f.fields.name}` binds one
 * input to one signal and typing in it re-renders only what reads that field.
 *
 * ```ts
 * export const f = form({
 *   initial: () => settings(),
 *   validate: ({ accent }) => ({ accent: /^#[0-9a-f]{6}$/i.test(accent) ? null : 'must be a hex color' }),
 *   submit: (values) => ship.settings.write(values),
 * })
 * ```
 * ```html
 * <form on:submit={f.submit}>
 *   <input bind:value={f.fields.accent} invalid={!!f.errors().accent}>
 *   <button disabled={!f.valid() || f.pending()}>save</button>
 *   <button type="button" on:click={f.reset}>reset</button>
 * </form>
 * ```
 */
export function form<T extends Record<string, unknown>>(
  opts: FormOptions<T>,
): {
  fields: { [K in keyof T]: Signal<T[K]> }
  values: () => T
  errors: () => Partial<Record<keyof T, string | null>>
  valid: () => boolean
  dirty: () => boolean
  pending: () => boolean
  lastError: () => Error | null
  reset: () => void
  submit: (e?: Event) => Promise<void>
} {
  const read = (): T => (typeof opts.initial === 'function' ? (opts.initial as () => T)() : opts.initial)

  const initial = read()
  const keys = Object.keys(initial) as (keyof T)[]

  // One signal per field. Not one signal holding an object — that would re-render
  // every input on every keystroke, which is the whole reason forms feel slow.
  const fields = {} as { [K in keyof T]: Signal<T[K]> }
  for (const k of keys) fields[k] = signal(initial[k]) as Signal<T[typeof k]>

  const pending = signal(false)
  const lastError = signal<Error | null>(null)
  // The baseline `dirty` compares against. Re-captured on reset, so resetting a
  // form to fresh server values makes it clean again rather than instantly dirty.
  const baseline = signal(initial)

  const values = derived(() => {
    const out = {} as T
    for (const k of keys) out[k] = fields[k]()
    return out
  })

  const errors = derived(() => (opts.validate ? opts.validate(values()) : {}))

  const valid = derived(() => Object.values(errors()).every((e) => e == null))

  const dirty = derived(() => {
    const v = values()
    const b = baseline()
    return keys.some((k) => v[k] !== b[k])
  })

  const reset = (): void => {
    const fresh = read()
    // One batch: a reset is ONE state change, not N. Without this every field
    // would tick the effects that read `values`/`errors`/`dirty` in turn.
    batch(() => {
      for (const k of keys) fields[k].set(fresh[k])
      baseline.set(fresh)
      lastError.set(null)
    })
  }

  const submit = async (e?: Event): Promise<void> => {
    // The handler is meant to sit on `on:submit`, so it owns the preventDefault —
    // forgetting it is a full page reload, and that is not the author's problem.
    e?.preventDefault()
    if (!valid() || pending()) return
    if (!opts.submit) return
    pending.set(true)
    lastError.set(null)
    try {
      await opts.submit(values())
      // A successful submit makes the current values the new baseline: the form is
      // saved, therefore it is clean.
      baseline.set(values())
    } catch (err) {
      lastError.set(err instanceof Error ? err : new Error(String(err)))
    } finally {
      pending.set(false)
    }
  }

  return { fields, values, errors, valid, dirty, pending, lastError, reset, submit }
}
