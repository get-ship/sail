<!--
  <sail-color-picker> — hex text entry + swatch grid + HSL sliders in a popover
  panel. `value` is a `#rrggbb` hex; tone drives the focus ring; size fills the
  control channels. Native range inputs take the tone through the framework's
  accent-color contract.
-->

<scope>
  props: {
    name = ''
    value = '#000000'
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
  }
</scope>
<controller>
import { controller, effect, signal } from '@ship/sail-runtime'
import type { ControllerScope } from '@ship/sail-runtime'
import {
  type Swatch,
  deriveSwatches,
  hexToRgb,
  hslToHex,
  isHex,
  normHex,
  rgbToHsl,
} from './color-math'
import type { Scope } from './sail-color-picker.sail'

let idSeq = 0

export default controller((_ctx: ControllerScope<Scope>) => {
    const open = signal(false)
    const hue = signal(0)
    const sat = signal(0)
    const light = signal(0)
    const hueText = signal('0')
    const satText = signal('0')
    const lightText = signal('0')
    const hexDraft = signal('')
    const inputEl = signal<HTMLInputElement | null>(null)
    const id = ++idSeq
    const hueId = signal(`sail-color-picker-h-${id}`)
    const satId = signal(`sail-color-picker-s-${id}`)
    const lightId = signal(`sail-color-picker-l-${id}`)
    // Theme-derived at mount() from the host's computed tone tokens — never a
    // hardcoded palette. Seeded empty; populated once the host is connected.
    const swatches = signal<Swatch[]>([])

    mount((host) => {
      let suppress = false
      // `value` is a declared <prop> — writes below go through its own
      // compiler-installed accessor (the same signal `_ctx.value()` reads),
      // never a disconnected local copy, so external `el.value = ...` and
      // this component's own commits stay in sync.
      const hostEl = host as HTMLElement & { value: string }

      // Derive the swatch palette from the host's resolved design-system tone
      // tokens (+ a neutral ramp) now that the element is connected and the
      // cascade is in effect.
      swatches.set(deriveSwatches(host))

      const emit = (hex: string) => {
        host.dispatchEvent(
          new CustomEvent('sail-change', {
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
        hue.set(hsl.h)
        sat.set(hsl.s)
        light.set(hsl.l)
        hueText.set(String(hsl.h))
        satText.set(String(hsl.s))
        lightText.set(String(hsl.l))
        hexDraft.set(hex)
        suppress = false
      }

      const commitHex = (raw: string) => {
        const norm = normHex(raw)
        if (!norm) return false
        hostEl.value = norm
        syncFromHex(norm)
        emit(norm)
        return true
      }

      // External value → internal HSL.
      const stopValueSync = effect(() => {
        const v = _ctx.value()
        if (suppress) return
        if (isHex(v as string)) syncFromHex(v as string)
      })

      // Initial seed.
      if (isHex(_ctx.value() as string)) syncFromHex(_ctx.value() as string)

      // Slider edits → value.
      const sliderEffect = effect(() => {
        const h = Number(hueText())
        const s = Number(satText())
        const l = Number(lightText())
        if (suppress) return
        if (!Number.isFinite(h) || !Number.isFinite(s) || !Number.isFinite(l)) return
        const hex = hslToHex(h, s, l)
        hue.set(Math.round(h))
        sat.set(Math.round(s))
        light.set(Math.round(l))
        if (hex !== (_ctx.value() as string).toLowerCase()) {
          suppress = true
          hostEl.value = hex
          hexDraft.set(hex)
          suppress = false
          emit(hex)
        }
      })

      const inputElement = inputEl() as HTMLInputElement | null
      if (!inputElement) return

      const openPanel = () => {
        if (_ctx.disabled() || open()) return
        open.set(true)
      }
      const closePanel = () => {
        if (!open()) return
        open.set(false)
      }

      const onInputClick = () => openPanel()
      const onInputChange = () => {
        // User typed a hex in the trigger input.
        const v = inputElement.value.trim()
        if (v === '') return
        if (!commitHex(v)) inputElement.value = _ctx.value() as string
      }
      const onInputKeydown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && open()) {
          e.preventDefault()
          closePanel()
        } else if (e.key === 'ArrowDown' && !open()) {
          e.preventDefault()
          openPanel()
        }
      }
      inputElement.addEventListener('click', onInputClick)
      inputElement.addEventListener('change', onInputChange)
      inputElement.addEventListener('keydown', onInputKeydown)

      const onShadowClick = (e: Event) => {
        const t = e.target as HTMLElement | null
        const sw = t?.closest?.('button[data-hex]') as HTMLElement | null
        if (sw) {
          const hex = (sw.dataset.hex ?? '').toLowerCase()
          if (isHex(hex)) commitHex(hex)
        }
      }
      const onHexInputChange = (e: Event) => {
        const t = e.target as HTMLInputElement | null
        if (!t || !t.matches('input[type="text"]')) return
        if (!commitHex(t.value)) t.value = _ctx.value() as string
      }

      const onDocPointer = (e: Event) => {
        if (!open()) return
        const path = e.composedPath()
        if (path.includes(host)) return
        closePanel()
      }

      ;(host.shadowRoot ?? host).addEventListener('click', onShadowClick)
      ;(host.shadowRoot ?? host).addEventListener('change', onHexInputChange)
      document.addEventListener('pointerdown', onDocPointer)

      host.expose!({
        focus: () => inputElement.focus(),
        blur: () => inputElement.blur(),
        open: openPanel,
        close: closePanel,
      })

      return () => {
        stopValueSync()
        sliderEffect()
        inputElement.removeEventListener('click', onInputClick)
        inputElement.removeEventListener('change', onInputChange)
        inputElement.removeEventListener('keydown', onInputKeydown)
        ;(host.shadowRoot ?? host).removeEventListener('click', onShadowClick)
        ;(host.shadowRoot ?? host).removeEventListener('change', onHexInputChange)
        document.removeEventListener('pointerdown', onDocPointer)
      }
    })

    return {
      open, hue, sat, light, hueText, satText, lightText, hexDraft,
      inputEl, hueId, satId, lightId, swatches,
    }
  })

</controller>
<column gap="0" sx={style.position(relative).w(full)}>
<row align="center" gap="2" sx={style.w(full).minH(size-h).padY(0).padX(size-px).bg(background).fg(foreground).borderWidth(1).border(border).radius(md).when({ focusWithin: style.outline(ring) })}>
  <span aria-hidden="true" use:swatch={value() || '#000000'} sx={style.w(4).h(4).minW(4).radius(sm).borderWidth(1).border(border-subtle).bg(bind(swatchColor))}></span>
  <input type="text" flush sx={style.grow(1).minW(0).fontFamily(mono).textCase(lowercase)} name={name()} placeholder="#000000" disabled={disabled()} autocomplete="off" aria-haspopup="dialog" aria-expanded={open() ? 'true' : 'false'} bind:value={value} bind:this={inputEl} />
</row>
{#if open() && !disabled()}
  <column gap="2" role="dialog" aria-label="Choose color" sx={style.position(absolute).left(0).z(popover).w(panel-max-w-md).pad(3).mt(1).bg(popover).fg(popover-foreground).borderWidth(1).border(border).radius(md).shadow(md).fg(size-fs)}>
    <grid cols="6" gap="2">
      {#for sw of swatches() key={sw.hex}}
        <button
          type="button"
          aria-selected={sw.hex === value().toLowerCase() ? 'true' : 'false'}
          data-hex={sw.hex}
          aria-label={sw.label}
          use:swatch={sw.hex}
          sx={style.borderWidth(1).border(border-subtle).radius(sm).pad(0).aspect(square).cursor(pointer).bg(bind(swatchColor)).when({ selected: style.outline(primary) })}
        ></button>
      {/for}
    </grid>
    <column gap="1">
      <row align="center" gap="2">
        <label for={hueId()} sx={style.w(4).fg(muted-foreground)}>H</label>
        <input id={hueId()} type="range" min="0" max="360" step="1"
               bind:value={hueText} data-axis="h" sx={style.grow(1).minW(0)} />
        <span sx={style.w(8).textAlign(end).fg(muted-foreground)}>{hue()}°</span>
      </row>
      <row align="center" gap="2">
        <label for={satId()} sx={style.w(4).fg(muted-foreground)}>S</label>
        <input id={satId()} type="range" min="0" max="100" step="1"
               bind:value={satText} data-axis="s" sx={style.grow(1).minW(0)} />
        <span sx={style.w(8).textAlign(end).fg(muted-foreground)}>{sat()}%</span>
      </row>
      <row align="center" gap="2">
        <label for={lightId()} sx={style.w(4).fg(muted-foreground)}>L</label>
        <input id={lightId()} type="range" min="0" max="100" step="1"
               bind:value={lightText} data-axis="l" sx={style.grow(1).minW(0)} />
        <span sx={style.w(8).textAlign(end).fg(muted-foreground)}>{light()}%</span>
      </row>
    </column>
    <row align="center" gap="2">
      <span sx={style.fg(muted-foreground)}>Hex</span>
      <input
        type="text"
        bind:value={hexDraft}
        spellcheck="false"
        autocomplete="off"
        aria-label="Hex color"
        sx={style.grow(1).minW(0).fontFamily(mono).textCase(lowercase)}
      />
    </row>
  </column>
{/if}
</column>
