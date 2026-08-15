import type { Action } from '../action'

/**
 * `use:rangePair` — host for <slider>. Copies min/max/step/disabled/value
 * onto the first range input; when [range], also drives value-end on the
 * second. Keeps start ≤ end. rangeFill on each input paints the track.
 *
 * `bind:value` writes the host `.value` property (not the attribute).
 * We expose a real value accessor so that write moves the thumb and
 * user input reads back through the same property.
 */
export const rangePair: Action<void> = (node) => {
  const host = node as HTMLElement

  const thumbs = (): HTMLInputElement[] =>
    [...host.querySelectorAll<HTMLInputElement>('input[type="range"]')]

  const num = (name: string, fallback: number): number => {
    const raw = host.getAttribute(name)
    if (raw == null || raw === '') return fallback
    const n = parseFloat(raw)
    return Number.isFinite(n) ? n : fallback
  }

  const stamp = (el: HTMLInputElement, name: string, raw: string | null): void => {
    if (raw == null || raw === '') {
      if (el.hasAttribute(name)) el.removeAttribute(name)
    } else if (el.getAttribute(name) !== raw) {
      el.setAttribute(name, raw)
    }
  }

  const syncChrome = (): void => {
    const [a, b] = thumbs()
    const size = host.getAttribute('size')
    const ticks = host.getAttribute('ticks')
    const name = host.getAttribute('name')
    for (const el of [a, b]) {
      if (!el) continue
      stamp(el, 'size', size)
    }
    if (a) {
      stamp(a, 'ticks', ticks)
      stamp(a, 'name', name)
    }
  }

  const sync = (): void => {
    const [a, b] = thumbs()
    if (!a) return
    const min = num('min', 0)
    const max = num('max', 100)
    const step = host.getAttribute('step') ?? '1'
    const disabled = host.hasAttribute('disabled')
    const ranged = host.hasAttribute('range')
    let start = num('value', min)
    let end = num('value-end', max)
    if (ranged && start > end) {
      const t = start
      start = end
      end = t
    }
    start = Math.min(max, Math.max(min, start))
    end = Math.min(max, Math.max(min, end))

    const apply = (el: HTMLInputElement, value: number) => {
      el.min = String(min)
      el.max = String(max)
      el.step = step
      el.disabled = disabled
      if (el.value !== String(value)) el.value = String(value)
      el.dispatchEvent(new Event('input', { bubbles: false }))
    }

    apply(a, start)
    if (b) {
      apply(b, ranged ? end : max)
      b.tabIndex = ranged ? 0 : -1
      b.hidden = !ranged
    }
    syncChrome()
  }

  const writeHostValue = (raw: unknown): void => {
    const next = raw == null ? '' : String(raw)
    if (host.getAttribute('value') !== next) host.setAttribute('value', next)
    const [a] = thumbs()
    if (a && a.value !== next) {
      a.value = next
      a.dispatchEvent(new Event('input', { bubbles: false }))
    }
  }

  const pending = Object.prototype.hasOwnProperty.call(host, 'value')
    ? String((host as HTMLInputElement).value ?? '')
    : undefined
  Object.defineProperty(host, 'value', {
    configurable: true,
    enumerable: true,
    get(): string {
      return thumbs()[0]?.value ?? host.getAttribute('value') ?? ''
    },
    set(v: unknown) {
      writeHostValue(v)
    },
  })
  if (pending !== undefined && pending !== '') writeHostValue(pending)

  const onInput = (ev: Event): void => {
    const [a, b] = thumbs()
    const t = ev.target
    if (!(t instanceof HTMLInputElement)) return
    if (t === a) host.setAttribute('value', t.value)
    if (t === b) host.setAttribute('value-end', t.value)
    const start = parseFloat(a?.value ?? '0')
    const end = parseFloat(b?.value ?? a?.value ?? '0')
    if (host.hasAttribute('range') && start > end) {
      if (t === a) {
        host.setAttribute('value', String(end))
        if (a) a.value = String(end)
      } else {
        host.setAttribute('value-end', String(start))
        if (b) b.value = String(start)
      }
    }
  }

  sync()
  host.addEventListener('input', onInput)
  const obs = new MutationObserver(sync)
  obs.observe(host, {
    attributes: true,
    attributeFilter: [
      'value',
      'value-end',
      'min',
      'max',
      'step',
      'disabled',
      'range',
      'size',
      'ticks',
      'name',
    ],
    childList: true,
  })

  return {
    destroy: () => {
      host.removeEventListener('input', onInput)
      obs.disconnect()
      delete (host as { value?: unknown }).value
    },
  }
}
