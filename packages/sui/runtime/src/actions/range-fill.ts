import type { Action } from '../action'

/**
 * `use:rangeFill` — the ONE thing a native `<input type="range">` can't draw on
 * Chrome/WebKit: the filled portion of the track. Everything else (value, step,
 * drag, keyboard, role=slider + aria-valuenow, form submission) is the
 * platform's. This action mirrors the value into a `--rng-pct` custom property
 * so RANGE_SKIN_CSS's webkit-track gradient fills to the thumb (Firefox already
 * fills via ::-moz-range-progress). ~10 lines, no controller, compiles away.
 */
export const rangeFill: Action<void> = (node) => {
  const el = node as HTMLInputElement

  const sync = (): void => {
    const min = parseFloat(el.min || '0')
    const max = parseFloat(el.max || '100')
    const val = parseFloat(el.value || '0')
    const pct = max > min ? ((val - min) / (max - min)) * 100 : 0
    el.style.setProperty('--rng-pct', `${pct}%`)
  }
  sync()
  el.addEventListener('input', sync)
  // catch attribute-set value / min / max (e.g. a reactive value={sig()}).
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['value', 'min', 'max'] })

  return {
    destroy: () => {
      el.removeEventListener('input', sync)
      obs.disconnect()
    },
  }
}
