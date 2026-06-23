import { controller, effect, ref, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import {
  type Swatch,
  deriveSwatches,
  hexToRgb,
  hslToHex,
  isHex,
  normHex,
  rgbToHsl,
} from './color-math'
import type { Scope } from './sui-color-picker.sui'

let idSeq = 0

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const open = signal(false)
    const hue = signal(0)
    const sat = signal(0)
    const light = signal(0)
    const hueText = signal('0')
    const satText = signal('0')
    const lightText = signal('0')
    const hexDraft = signal('')
    const inputEl = ref<HTMLInputElement>(null)
    const panelEl = ref<HTMLDivElement>(null)
    const hexInputEl = ref<HTMLInputElement>(null)
    const id = ++idSeq
    const hueId = signal(`sui-color-picker-h-${id}`)
    const satId = signal(`sui-color-picker-s-${id}`)
    const lightId = signal(`sui-color-picker-l-${id}`)
    // Theme-derived at mount() from the host's computed tone tokens — never a
    // hardcoded palette. Seeded empty; populated once the host is connected.
    const swatches = signal<Swatch[]>([])
    return {
      open, hue, sat, light, hueText, satText, lightText, hexDraft,
      inputEl, panelEl, hexInputEl, hueId, satId, lightId, swatches,
    }
  },

  mount(state, host) {
    let suppress = false

    // Derive the swatch palette from the host's resolved design-system tone
    // tokens (+ a neutral ramp) now that the element is connected and the
    // cascade is in effect.
    state.swatches = deriveSwatches(host)

    const emit = (hex: string) => {
      host.dispatchEvent(
        new CustomEvent('sui-change', {
          detail: { value: hex },
          bubbles: true,
          composed: true,
        }),
      )
    }

    const syncFromHex = (hex: string) => {
      const rgb = hexToRgb(hex)
      if (!rgb) return
      const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b)
      suppress = true
      state.hue = hsl.h
      state.sat = hsl.s
      state.light = hsl.l
      state.hueText = String(hsl.h)
      state.satText = String(hsl.s)
      state.lightText = String(hsl.l)
      state.hexDraft = hex
      suppress = false
    }

    const commitHex = (raw: string) => {
      const norm = normHex(raw)
      if (!norm) return false
      state.value = norm
      syncFromHex(norm)
      emit(norm)
      return true
    }

    // External value → internal HSL.
    const stopValueSync = effect(() => {
      const v = state.value
      if (suppress) return
      if (isHex(v as string)) syncFromHex(v as string)
    })

    // Initial seed.
    if (isHex(state.value as string)) syncFromHex(state.value as string)

    // Slider edits → value.
    const sliderEffect = effect(() => {
      const h = Number(state.hueText)
      const s = Number(state.satText)
      const l = Number(state.lightText)
      if (suppress) return
      if (!Number.isFinite(h) || !Number.isFinite(s) || !Number.isFinite(l)) return
      const hex = hslToHex(h, s, l)
      state.hue = Math.round(h)
      state.sat = Math.round(s)
      state.light = Math.round(l)
      if (hex !== (state.value as string).toLowerCase()) {
        suppress = true
        state.value = hex
        state.hexDraft = hex
        suppress = false
        emit(hex)
      }
    })

    const inputEl = state.inputEl as HTMLInputElement | null
    if (!inputEl) return

    const openPanel = () => {
      if (state.disabled || state.open) return
      state.open = true
    }
    const closePanel = () => {
      if (!state.open) return
      state.open = false
    }

    const onInputClick = () => openPanel()
    const onInputChange = () => {
      // User typed a hex in the trigger input.
      const v = inputEl.value.trim()
      if (v === '') return
      if (!commitHex(v)) inputEl.value = state.value as string
    }
    const onInputKeydown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && state.open) {
        e.preventDefault()
        closePanel()
      } else if (e.key === 'ArrowDown' && !state.open) {
        e.preventDefault()
        openPanel()
      }
    }
    inputEl.addEventListener('click', onInputClick)
    inputEl.addEventListener('change', onInputChange)
    inputEl.addEventListener('keydown', onInputKeydown)

    const onShadowClick = (e: Event) => {
      const t = e.target as HTMLElement | null
      const sw = t?.closest?.('button.swatch[data-hex]') as HTMLElement | null
      if (sw) {
        const hex = (sw.dataset.hex ?? '').toLowerCase()
        if (isHex(hex)) commitHex(hex)
      }
    }
    const onHexInputChange = (e: Event) => {
      const t = e.target as HTMLInputElement | null
      if (!t || !t.matches('.hex-row input')) return
      if (!commitHex(t.value)) t.value = state.value as string
    }

    const onDocPointer = (e: Event) => {
      if (!state.open) return
      const path = e.composedPath()
      if (path.includes(host)) return
      closePanel()
    }

    host.shadowRoot?.addEventListener('click', onShadowClick)
    host.shadowRoot?.addEventListener('change', onHexInputChange)
    document.addEventListener('pointerdown', onDocPointer)

    host.expose!({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      open: openPanel,
      close: closePanel,
    })

    return () => {
      stopValueSync()
      sliderEffect()
      inputEl.removeEventListener('click', onInputClick)
      inputEl.removeEventListener('change', onInputChange)
      inputEl.removeEventListener('keydown', onInputKeydown)
      host.shadowRoot?.removeEventListener('click', onShadowClick)
      host.shadowRoot?.removeEventListener('change', onHexInputChange)
      document.removeEventListener('pointerdown', onDocPointer)
    }
  },
})
