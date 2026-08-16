// `bind:value` hydrator — two-way binding to a writable signal.
//
// The compiler resolves the signal at emit time and passes it in.
// We wire two directions:
//   - DOM → signal: `input` event writes `el.value` back via `.set()`
//   - signal → DOM: `effect()` reads the signal and assigns `.value`

import { effect, type Signal } from '@ship/sail-runtime'

function isValueControl(
  el: EventTarget | null,
): el is HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement {
  return (
    el instanceof HTMLInputElement ||
    el instanceof HTMLTextAreaElement ||
    el instanceof HTMLSelectElement
  )
}

function writeHydrateValue(el: Element, next: string): void {
  if (isValueControl(el)) {
    if (el.value !== next) el.value = next
    return
  }
  if (el.getAttribute('value') !== next) el.setAttribute('value', next)
  const host = el as HTMLInputElement
  if (host.value !== next) host.value = next
  const inner = el.querySelector<HTMLInputElement>('input:not([hidden]), textarea, select')
  if (inner && inner.value !== next) inner.value = next
}

export function hydrateBindValue(
  el: Element,
  sig: Signal<unknown>,
): () => void {
  const onInput = (ev: Event): void => {
    const t = ev.target
    const v = isValueControl(t)
      ? t.value
      : isValueControl(el)
        ? el.value
        : (el.querySelector('input, textarea, select') as HTMLInputElement | null)?.value ??
          (el as HTMLInputElement).value ??
          ''
    sig.set(v)
  }
  el.addEventListener('input', onInput)
  const dispose = effect(() => {
    const v = sig()
    writeHydrateValue(el, v == null ? '' : String(v))
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
