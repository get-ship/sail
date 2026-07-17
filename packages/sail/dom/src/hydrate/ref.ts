// `bind:this={var}` hydrator.
//
// Mirrors `attach/bindings/ref.ts::wireRef`: if `scope[varName]` is
// already a writable signal (e.g. the controller pre-declared it),
// reuse it; otherwise create a fresh `signal(el)` and write it back
// onto the scope so subsequent reads find it. On unmount the signal
// is reset to `null`.

import { signal, type Signal } from '@ship/sail-runtime'

export function hydrateRef(
  el: Element,
  scope: Record<string, unknown>,
  varName: string,
): () => void {
  const existing = scope[varName]
  let ref: Signal<Element | null>
  if (isWritableSignal(existing)) {
    ;(existing as Signal<unknown>).set(el)
    ref = existing as Signal<Element | null>
  } else {
    ref = signal<Element | null>(el)
    scope[varName] = ref
  }
  return () => {
    ref.set(null)
  }
}

function isWritableSignal(v: unknown): v is Signal<unknown> {
  return (
    typeof v === 'function' &&
    'set' in (v as object) &&
    typeof (v as unknown as { set: unknown }).set === 'function'
  )
}
