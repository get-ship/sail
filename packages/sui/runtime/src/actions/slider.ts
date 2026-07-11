import type { Action } from '../action'

/**
 * `use:slider` — the L1 `<slider>` element's behavior. The track/fill/thumb
 * surface is pure CSS; this action owns everything interactive:
 *  - the value lives on the `value` attribute (clamped to [min,max] on `step`);
 *  - it positions the fill width + thumb offset and keeps aria-valuenow/min/max
 *    (+ aria-label / aria-disabled / tabindex) in sync;
 *  - pointer drag + click-on-track, and the full keyboard suite (arrows / Home /
 *    End / PageUp / PageDown);
 *  - a hidden <input> mirrors value for native form participation;
 *  - each committed change fires sui-change {value}.
 * A reactive `value={sig()}` at the use-site re-syncs via the attribute
 * observer. Written once in the toolchain — no per-instance controller.
 */
export const slider: Action<void> = (node) => {
  const el = node as HTMLElement
  const track = el.querySelector('.track') as HTMLElement
  const fill = el.querySelector('.fill') as HTMLElement
  const thumb = el.querySelector('.thumb') as HTMLElement

  const num = (attr: string, def: number): number => {
    const v = parseFloat(el.getAttribute(attr) ?? '')
    return Number.isFinite(v) ? v : def
  }
  const min = (): number => num('min', 0)
  const max = (): number => num('max', 100)
  const step = (): number => num('step', 1) || 1
  const value = (): number => num('value', 0)
  const disabled = (): boolean => el.hasAttribute('disabled')

  const clamp = (v: number): number => {
    const mn = min(), st = step()
    const stepped = Math.round((v - mn) / st) * st + mn
    return Math.max(mn, Math.min(max(), stepped))
  }

  const hidden = document.createElement('input')
  hidden.type = 'hidden'
  el.appendChild(hidden)

  const sync = (): void => {
    const v = value(), mn = min(), mx = max()
    const pct = ((v - mn) / ((mx - mn) || 1)) * 100
    fill.style.width = `${pct}%`
    thumb.style.left = `${pct}%`
    thumb.setAttribute('aria-valuenow', String(v))
    thumb.setAttribute('aria-valuemin', String(mn))
    thumb.setAttribute('aria-valuemax', String(mx))
    thumb.setAttribute('aria-label', el.getAttribute('label') || 'slider')
    const dis = disabled()
    thumb.setAttribute('aria-disabled', dis ? 'true' : 'false')
    thumb.setAttribute('tabindex', dis ? '-1' : '0')
    hidden.name = el.getAttribute('name') || ''
    hidden.value = String(v)
  }

  const setValue = (v: number): void => {
    const c = clamp(v)
    if (c === value()) return
    el.setAttribute('value', String(c))
    sync()
    el.dispatchEvent(new CustomEvent('sui-change', { detail: { value: c }, bubbles: true, composed: true }))
  }

  const fromPointer = (clientX: number): number => {
    const r = track.getBoundingClientRect()
    const pct = Math.max(0, Math.min(1, (clientX - r.left) / r.width))
    return min() + pct * (max() - min())
  }

  let dragging = false
  const onPointerDown = (e: PointerEvent): void => {
    if (disabled()) return
    e.preventDefault()
    dragging = true
    setValue(fromPointer(e.clientX))
    thumb.focus()
    const onMove = (ev: PointerEvent): void => { if (dragging) setValue(fromPointer(ev.clientX)) }
    const onUp = (): void => {
      dragging = false
      document.removeEventListener('pointermove', onMove)
      document.removeEventListener('pointerup', onUp)
    }
    document.addEventListener('pointermove', onMove)
    document.addEventListener('pointerup', onUp)
  }
  el.addEventListener('pointerdown', onPointerDown)

  const onKeydown = (e: KeyboardEvent): void => {
    if (disabled()) return
    const st = step()
    let handled = true
    switch (e.key) {
      case 'ArrowRight': case 'ArrowUp': setValue(value() + st); break
      case 'ArrowLeft': case 'ArrowDown': setValue(value() - st); break
      case 'Home': setValue(min()); break
      case 'End': setValue(max()); break
      case 'PageUp': setValue(value() + st * 10); break
      case 'PageDown': setValue(value() - st * 10); break
      default: handled = false
    }
    if (handled) e.preventDefault()
  }
  thumb.addEventListener('keydown', onKeydown)

  sync()
  const obs = new MutationObserver(sync)
  obs.observe(el, { attributes: true, attributeFilter: ['value', 'min', 'max', 'disabled', 'name', 'label'] })

  return {
    destroy: () => {
      el.removeEventListener('pointerdown', onPointerDown)
      thumb.removeEventListener('keydown', onKeydown)
      obs.disconnect()
      hidden.remove()
    },
  }
}
