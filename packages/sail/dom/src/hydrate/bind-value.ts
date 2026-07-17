// `bind:value` hydrator — two-way binding to a writable signal.
//
// The compiler resolves the signal at emit time and passes it in.
// We wire two directions:
//   - DOM → signal: `input` event writes `el.value` back via `.set()`
//   - signal → DOM: `effect()` reads the signal and assigns `.value`

import { effect, type Signal } from '@ship/sail-runtime'

export function hydrateBindValue(
  el: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement,
  sig: Signal<unknown>,
): () => void {
  const onInput = (): void => {
    sig.set((el as HTMLInputElement).value)
  }
  el.addEventListener('input', onInput)
  const dispose = effect(() => {
    const v = sig()
    const next = v == null ? '' : String(v)
    if ((el as HTMLInputElement).value !== next) {
      ;(el as HTMLInputElement).value = next
    }
  })
  return () => {
    el.removeEventListener('input', onInput)
    dispose()
  }
}

export function hydrateBindChecked(
  el: HTMLInputElement,
  sig: Signal<unknown>,
): () => void {
  const onChange = (): void => {
    sig.set(el.checked)
  }
  el.addEventListener('change', onChange)
  const dispose = effect(() => {
    const next = !!sig()
    if (el.checked !== next) el.checked = next
  })
  return () => {
    el.removeEventListener('change', onChange)
    dispose()
  }
}
