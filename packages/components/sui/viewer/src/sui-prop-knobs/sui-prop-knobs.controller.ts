// <sui-prop-knobs> controller — the CEM-driven playground control set
// (build-spec §The viewing components #2).
//
// Controller-purity: this file does NO view-markup DOM construction. It owns
// the reactive store + pure classification + lifecycle wiring only. The
// `{#for}`/`{#match}` over the classified knobs lives in the template; the
// composed sui-toggle-group / sui-switch / sui-slider / sui-input render the
// controls. The controller writes no HTML-string properties at all.
//
// Pure transform: CEM `attributes[]` → a FOUR-way classification —
//   values[] non-empty            → enum   (segmented sui-toggle-group)
//   type 'boolean'                → bool   (sui-switch)
//   type 'number' (no values)     → number (sui-slider | sui-input type=number)
//   else                          → string (sui-input)
// Holds `propValues` as the single source; seeds defaults (prefers the largest
// size-enum variant); emits `sui-prop-change {name,value,values}` (deduped) +
// `sui-prop-reset {values}`.
//
// Structured inputs (the CEM array `props`, the controlled `value` snapshot)
// arrive via `prop:*` IDENTITY — set as JS properties on the host, never a
// json/object attribute (Cardinal: `<prop type=json>` does not exist).

import { controller, signal, derived, effect, untrack, useShared } from '@ship/sui-runtime'
import type { Signal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-prop-knobs.sui'
import {
  classifyKnob,
  seedDefaults,
  coerceValue,
  enumOptions,
  type CemAttr,
  type Knob,
  type KnobValues,
} from './classify'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const cemAttrs = signal<CemAttr[]>([])
    const propValues = signal<KnobValues>({})

    // STABLE per-knob value signals, cached by prop name. The control binds a
    // reactive zero-arg read (`prop:value={knob.value()}`), so a controlled
    // value change after mount re-fires the binding — the keyed {#for} row
    // (keyed by name) is NEVER re-created on a value edit.
    const valueSigs = new Map<string, Signal<string>>()
    const checkedSigs = new Map<string, Signal<boolean>>()
    const valueSig = (name: string): Signal<string> => {
      let s = valueSigs.get(name)
      if (!s) {
        s = signal('')
        valueSigs.set(name, s)
      }
      return s
    }
    const checkedSig = (name: string): Signal<boolean> => {
      let s = checkedSigs.get(name)
      if (!s) {
        s = signal(false)
        checkedSigs.set(name, s)
      }
      return s
    }

    // The knob LIST derives from the CEM array ONLY (stable rows); each item
    // exposes its stable value/checked signal reads for the template.
    const knobs = derived<Knob[]>(() =>
      cemAttrs().map((attr) => {
        const kind = classifyKnob(attr)
        const vs = valueSig(attr.name)
        const cs = checkedSig(attr.name)
        return {
          name: attr.name,
          kind,
          options: kind === 'enum' ? enumOptions(attr) : [],
          value: () => vs(),
          checked: () => cs(),
        }
      }),
    )

    // Push the controlled `propValues` into the per-knob signals — the control
    // binding re-fires off these, decoupled from the for-row identity. Tracks
    // `cemAttrs` too so a freshly-classified knob's signal is seeded the moment
    // the row appears (not only on a later value change).
    effect(() => {
      const values = propValues()
      for (const attr of cemAttrs()) {
        const v = values[attr.name]
        valueSig(attr.name).set(v == null ? '' : String(v))
        checkedSig(attr.name).set(v === true)
      }
    })

    return { cemAttrs, propValues, knobs }
  },

  mount(state, host) {
    // --- host prop:* → signals (identity; no json attribute) ---
    // `props` = the CEM `attributes[]`; `value` = the controlled propValues
    // snapshot the URL/deep-link seam drives. Both set as JS properties.
    type AnySig = { (): unknown; set: (u: (p: unknown) => unknown) => void }
    const identity = (v: unknown): unknown => v
    const defineHostProp = (
      name: string,
      sig: { (): unknown; set: (v: never) => void },
      coerce: (v: unknown) => unknown = identity,
    ): void => {
      const s = sig as unknown as AnySig
      const existing = (host as unknown as Record<string, unknown>)[name]
      if (existing !== undefined) s.set(() => coerce(existing))
      Object.defineProperty(host, name, {
        configurable: true,
        get: () => s(),
        set: (v: unknown) => s.set(() => coerce(v)),
      })
    }
    // `props` = the CEM `attributes[]`. The composing viewer threads
    // `prop:props={cem()}` where `cem()` is null during the initial render
    // window (its own source signal seeds only in mount) — a null must degrade
    // to "no knobs", never crash the `cemAttrs().map` / seed-effect iteration.
    // Coalesce here so the `CemAttr[]` contract holds at every read site.
    defineHostProp('props', state.$signals.cemAttrs, (v) => v ?? [])
    // `value` is the controlled snapshot — feed it straight into propValues.
    defineHostProp('value', state.$signals.propValues)

    // shared-context contract: when nested under <sui-component-viewer> the
    // provider publishes the propValues signal under 'viewer:propValues'.
    // Standalone there is NO provider → useShared returns null and the tag
    // operates purely off prop:value/prop:props (the null-fallback).
    const sharedValues = useShared<{ (): KnobValues; set: (v: KnobValues) => void }>(
      host,
      'viewer:propValues',
    )

    // --- the single source of truth + dedup guard ---
    let lastEmitted = '' // `${name}=${JSON.stringify(value)}` of the last edit

    const writeValues = (next: KnobValues): void => {
      state.propValues = next
      // mirror into the shared signal when a provider is present (lockstep)
      if (sharedValues) sharedValues.set(next)
    }

    // --- default seeding: prefer the largest size-enum variant ---
    // Seed once whenever the CEM array changes (a fresh target). The seeded
    // snapshot announces itself via sui-prop-reset so siblings re-sync.
    let lastSeededFor = ''
    const seedIfNeeded = (attrs: CemAttr[]): void => {
      const key = attrs.map((a) => a.name).join(',')
      if (key === lastSeededFor) return
      lastSeededFor = key
      const seeded = seedDefaults(attrs)
      // a host-supplied controlled value wins over a seeded default
      const merged = { ...seeded, ...untrack(() => state.propValues) }
      writeValues(merged)
      lastEmitted = ''
      host.emit!('sui-prop-reset', { values: merged })
    }

    const stop = effect(() => {
      const attrs = state.$signals.cemAttrs()
      untrack(() => seedIfNeeded(attrs))
    })

    // --- the ONE edit path: coerce → dedup → write → emit ---
    const applyKnob = (name: string, raw: unknown): void => {
      const attr = untrack(() => state.cemAttrs).find((a) => a.name === name)
      if (!attr) return
      const value = coerceValue(attr, raw)
      const key = `${name}=${JSON.stringify(value)}`
      if (key === lastEmitted) return // dedup before emit (the lastEmitted guard)
      lastEmitted = key
      const next = { ...untrack(() => state.propValues), [name]: value }
      writeValues(next)
      host.emit!('sui-prop-change', { name, value, values: next })
    }

    // The composed controls bubble their native events (composed:true) up to
    // the host shadow root. One delegated listener per event type reads the
    // owning knob name off the `[data-knob]` row and routes to applyKnob.
    const knobNameOf = (target: EventTarget | null): string | null => {
      const el = target as Element | null
      const row = el?.closest?.('[data-knob]') as HTMLElement | null
      return row?.dataset.knob ?? null
    }

    // sui-toggle-group (enum) + sui-switch/sui-checkbox (bool) + sui-slider
    // (number) all emit `sui-change`; detail carries `value` (enum/number) or
    // `checked` (bool).
    const onChange = (e: Event): void => {
      const name = knobNameOf(e.target)
      if (name == null) return
      const detail = (e as CustomEvent).detail as { value?: unknown; checked?: unknown } | undefined
      const raw = detail && 'checked' in detail ? detail.checked : detail?.value
      applyKnob(name, raw)
    }
    // sui-input (string + number-input) emits `sui-input` with detail.value.
    const onInput = (e: Event): void => {
      const name = knobNameOf(e.target)
      if (name == null) return
      const detail = (e as CustomEvent).detail as { value?: unknown } | undefined
      applyKnob(name, detail?.value)
    }
    host.addEventListener('sui-change', onChange)
    host.addEventListener('sui-input', onInput)

    return () => {
      stop()
      host.removeEventListener('sui-change', onChange)
      host.removeEventListener('sui-input', onInput)
    }
  },
})
