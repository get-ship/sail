import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-slider.sui'

export default controller<Scope>({
  mount(state, host) {

    // Light-DOM hidden input for form participation.
    const hiddenInput = document.createElement('input')
    hiddenInput.type = 'hidden'
    host.appendChild(hiddenInput)

    const clamp = (v: number): number => {
      const step = state.$signals.step() as number
      const min = state.$signals.min() as number
      const max = state.$signals.max() as number
      const stepped = Math.round((v - min) / step) * step + min
      return Math.max(min, Math.min(max, stepped))
    }

    const setValue = (v: number) => {
      const clamped = clamp(v)
      if (clamped === (state.$signals.value() as number)) return
      state.$signals.value.set(clamped)
      hiddenInput.name = state.$signals.name() as string
      hiddenInput.value = String(clamped)
      host.dispatchEvent(new CustomEvent('sui-change', {
        detail: { value: clamped },
        bubbles: true,
        composed: true,
      }))
    }

    const valueFromPointer = (clientX: number): number => {
      const track = state.$signals.trackEl() as HTMLElement | null
      if (!track) return state.$signals.value() as number
      const rect = track.getBoundingClientRect()
      const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
      return (state.$signals.min() as number) + pct * ((state.$signals.max() as number) - (state.$signals.min() as number))
    }

    // Sync fill + thumb position and hidden input on value/range changes.
    const stopPositionSync = effect(() => {
      const val = state.$signals.value() as number
      const min = state.$signals.min() as number
      const max = state.$signals.max() as number
      const range = max - min || 1
      const pct = ((val - min) / range) * 100

      const fill = state.$signals.fillEl() as HTMLElement | null
      const thumb = state.$signals.thumbEl() as HTMLElement | null
      if (fill) fill.style.width = `${pct}%`
      if (thumb) thumb.style.left = `${pct}%`

      hiddenInput.name = state.$signals.name() as string
      hiddenInput.value = String(val)
    })

    let dragging = false

    const onPointerDown = (e: PointerEvent) => {
      if (state.$signals.disabled() as boolean) return
      e.preventDefault()
      dragging = true
      setValue(valueFromPointer(e.clientX))
      const thumb = state.$signals.thumbEl() as HTMLElement | null
      thumb?.focus()

      const onMove = (ev: PointerEvent) => { if (dragging) setValue(valueFromPointer(ev.clientX)) }
      const onUp = () => {
        dragging = false
        document.removeEventListener('pointermove', onMove)
        document.removeEventListener('pointerup', onUp)
      }
      document.addEventListener('pointermove', onMove)
      document.addEventListener('pointerup', onUp)
    }

    const onKeydown = (e: KeyboardEvent) => {
      if (state.$signals.disabled() as boolean) return
      const step = state.$signals.step() as number
      let handled = true
      switch (e.key) {
        case 'ArrowRight':
        case 'ArrowUp':
          setValue((state.$signals.value() as number) + step)
          break
        case 'ArrowLeft':
        case 'ArrowDown':
          setValue((state.$signals.value() as number) - step)
          break
        case 'Home':
          setValue(state.$signals.min() as number)
          break
        case 'End':
          setValue(state.$signals.max() as number)
          break
        case 'PageUp':
          setValue((state.$signals.value() as number) + step * 10)
          break
        case 'PageDown':
          setValue((state.$signals.value() as number) - step * 10)
          break
        default:
          handled = false
      }
      if (handled) e.preventDefault()
    }

    const stopPointer = effect(() => {
      const container = state.$signals.containerEl() as HTMLElement | null
      if (!container) return
      container.addEventListener('pointerdown', onPointerDown as EventListener)
      return () => container.removeEventListener('pointerdown', onPointerDown as EventListener)
    })

    const stopKeydown = effect(() => {
      const thumb = state.$signals.thumbEl() as HTMLElement | null
      if (!thumb) return
      thumb.addEventListener('keydown', onKeydown)
      return () => thumb.removeEventListener('keydown', onKeydown)
    })

    return () => {
      stopPositionSync()
      stopPointer()
      stopKeydown()
      hiddenInput.remove()
    }
  },
})
