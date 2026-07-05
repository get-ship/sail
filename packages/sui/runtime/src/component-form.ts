// Form-association wiring for declarative components.
//
// The `component()` factory delegates here when a def declares
// `formValue:` / `formDisabled:`. We install `FormHooks` on the host
// (the form-aware element class in `component.ts` invokes them from
// the browser's `form*Callback` methods) and set up the
// `setFormValue` effect that keeps `ElementInternals` in sync with
// the prop signal.

import { camelOf } from './component'
import type { FormHooks } from './component'
import { decoderFor } from './component-decoders'
import type { ComponentDef, PropDef } from './component-factory'
import { effect, type Signal } from './signal'

interface HookHolder {
  __shipSetFormHooks?: FormHooks
  __shipSetFormValueCleanup?: () => void
}

/**
 * Install the form-association hooks + setFormValue effect on the
 * host. Returns the cleanup the caller should push onto the
 * component-level cleanup array; on disconnect the host's hook
 * holders are also cleared so a re-connect rebuilds them fresh.
 *
 * Throws synchronously on misconfiguration (missing prop, wrong
 * type) so the bug surfaces at first connect rather than as a silent
 * no-op in production.
 */
export function installFormHooks(
  host: HTMLElement,
  internals: ElementInternals | undefined,
  def: ComponentDef,
  propSignals: Record<string, Signal<unknown>>,
): (() => void) | undefined {
  if (!internals) return undefined
  const hooks: FormHooks = {}
  let stopValueEffect: (() => void) | undefined

  const valueAttr = def.formValue
  if (valueAttr !== undefined) {
    const { sig, propDef } = resolveProp(def, propSignals, valueAttr, 'formValue')
    const initial = propDef.default
    const decode = decoderFor(propDef)
    stopValueEffect = effect(() => {
      const v = sig()
      // Feature-guarded: jsdom (and other partial ElementInternals shims)
      // implement attachInternals without setFormValue — skipping the sync
      // beats crashing the effect in an environment with no form to sync.
      if (typeof internals.setFormValue === 'function') {
        internals.setFormValue(stringifyFormValue(v))
      }
    })
    hooks.reset = () => { sig.set(initial as unknown) }
    hooks.restore = (state: string | null) => { sig.set(decode(state)) }
  }

  const disabledAttr = def.formDisabled
  if (disabledAttr !== undefined) {
    const { sig, propDef } = resolveProp(def, propSignals, disabledAttr, 'formDisabled')
    if (propDef.type !== 'bool') {
      throw new Error(
        `[sui-runtime] formDisabled: "${disabledAttr}" must be a bool-typed prop`,
      )
    }
    hooks.disabled = (d: boolean) => { sig.set(!!d) }
  }

  ;(host as unknown as HookHolder).__shipSetFormHooks = hooks
  if (stopValueEffect) {
    ;(host as unknown as HookHolder).__shipSetFormValueCleanup = stopValueEffect
  }
  return stopValueEffect
}

/**
 * Tear down the form-association state on disconnect. Idempotent —
 * safe to call when no hooks were installed.
 */
export function clearFormHooks(host: HTMLElement): void {
  const holder = host as unknown as HookHolder
  delete holder.__shipSetFormHooks
  delete holder.__shipSetFormValueCleanup
}

function stringifyFormValue(v: unknown): string {
  if (v === null || v === undefined) return ''
  if (typeof v === 'boolean') return v ? 'on' : ''
  return String(v)
}

function resolveProp(
  def: ComponentDef,
  propSignals: Record<string, Signal<unknown>>,
  attr: string,
  channel: 'formValue' | 'formDisabled',
): { sig: Signal<unknown>; propDef: PropDef<unknown> } {
  const propDef = def.props[attr]
  if (!propDef) {
    throw new Error(
      `[sui-runtime] ${channel}: "${attr}" — no matching prop on this component`,
    )
  }
  const sig = propSignals[camelOf(attr)]
  if (!sig) {
    throw new Error(
      `[sui-runtime] ${channel}: "${attr}" — prop signal missing`,
    )
  }
  return { sig, propDef }
}
