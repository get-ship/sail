<!--
  <sail-color-picker> — visual color pick: live swatch + popover.
  Internal control value is `#rrggbb` (slider math is HSL for simple tracks);
  theme studio converts oklch ↔ hex at the boundary so tokens stay oklch.
  `showHex=false` hides hex text. Popover: native popover + use:anchor.
  ALWAYS show the color being edited (large preview + trigger swatch).
-->

<scope>
  props: {
    name = ''
    value = '#000000'
    disabled = false
    showHex = true
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
  }
</scope>
<controller>

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
    const inputEl = signal<HTMLElement | null>(null)
    const id = ++idSeq
    const hueId = signal(`sail-color-picker-h-${id}`)
    const satId = signal(`sail-color-picker-s-${id}`)
    const lightId = signal(`sail-color-picker-l-${id}`)
    const panelId = signal(`sail-color-picker-panel-${id}`)
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

      // True while a range slider is being dragged — parent must not thrash us.
      let gesture = false
      /** Hex at open — dismiss without a real edit must not emit (no false dirty). */
      let openHex = ''
      /** User changed color this open session (slider / swatch / hex text). */
      let userEdited = false

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

      const applyLocalHex = (norm: string) => {
        suppress = true
        hostEl.value = norm
        hexDraft.set(norm)
        suppress = false
      }

      /** Emit only when hex actually differs from open snapshot / current prop. */
      const emitIfChanged = (hex: string) => {
        const norm = normHex(hex) || hex.toLowerCase()
        if (!isHex(norm)) return false
        // Read previous BEFORE applyLocalHex — hostEl.value updates the prop signal.
        const prev = String(_ctx.value() ?? '').toLowerCase()
        if (norm === openHex || norm === prev) {
          applyLocalHex(norm)
          return false
        }
        applyLocalHex(norm)
        emit(norm)
        return true
      }

      const commitHex = (raw: string) => {
        const norm = normHex(raw)
        if (!norm) return false
        // Capture prop before local write; applyLocalHex mutates host value.
        const prev = String(_ctx.value() ?? '').toLowerCase()
        userEdited = true
        syncFromHex(norm)
        applyLocalHex(norm)
        // Intentional swatch/text: emit when different from prior prop.
        if (norm !== prev) emit(norm)
        return true
      }

      // External value → internal HSL.
      // While the popover is open (or a drag gesture is active), HSL is SSOT —
      // ignore parent oklch↔hex rewrites so sliders do not jump mid-drag.
      const stopValueSync = effect(() => {
        const v = _ctx.value()
        if (suppress || gesture || open()) return
        if (isHex(v as string)) syncFromHex(v as string)
      })

      // Initial seed.
      if (isHex(_ctx.value() as string)) syncFromHex(_ctx.value() as string)

      // Slider edits → local HSL/preview only. Never emit from this effect:
      // HSL↔hex roundtrip on mount/open would mark every token dirty and thrash parents.
      // Commit paths: pointerup / range change (commitFromSliders), swatch, hex text.
      const sliderEffect = effect(() => {
        const h = Number(hueText())
        const s = Number(satText())
        const l = Number(lightText())
        if (suppress) return
        if (!open()) return
        if (!Number.isFinite(h) || !Number.isFinite(s) || !Number.isFinite(l)) return
        const hex = hslToHex(h, s, l)
        hue.set(Math.round(h))
        sat.set(Math.round(s))
        light.set(Math.round(l))
        if (hex !== String(_ctx.value() ?? '').toLowerCase()) {
          applyLocalHex(hex)
        }
      })

      const commitFromSliders = () => {
        if (!open()) return
        if (!userEdited) return
        const hex = hslToHex(Number(hueText()), Number(satText()), Number(lightText()))
        if (!isHex(hex)) return
        emitIfChanged(hex)
      }

      const triggerEl = inputEl() as HTMLElement | null
      if (!triggerEl) return
      const rootForQuery = host.shadowRoot ?? host
      const hexInput = rootForQuery.querySelector(
        'input[type="text"][aria-label="Hex color"]',
      ) as HTMLInputElement | null

      const openPanel = () => {
        if (_ctx.disabled() || open()) return
        // Snapshot hex at open; dismiss without edit must not rewrite parent oklch.
        const cur = normHex(String(_ctx.value() ?? '')) || String(_ctx.value() ?? '').toLowerCase()
        openHex = isHex(cur) ? cur : ''
        userEdited = false
        if (openHex) syncFromHex(openHex)
        open.set(true)
        // Panel is gated by `if {open()}` — showPopover after it mounts.
        queueMicrotask(() => {
          const root = host.shadowRoot ?? host
          const panel = root.querySelector('[data-color-panel]') as HTMLElement & {
            showPopover?: () => void
          } | null
          try {
            panel?.showPopover?.()
          } catch {
            /* open signal still shows panel via if */
          }
        })
      }
      const closePanel = () => {
        if (!open()) return
        open.set(false)
        userEdited = false
        openHex = ''
        const root = host.shadowRoot ?? host
        const panel = root.querySelector('[data-color-panel]') as HTMLElement & {
          hidePopover?: () => void
        } | null
        try {
          panel?.hidePopover?.()
        } catch {
          /* ignore */
        }
      }

      /** Commit only if the user edited this session; never lossy-rewrite on bare dismiss. */
      const dismissPanel = () => {
        if (!open()) return
        if (userEdited) commitFromSliders()
        closePanel()
      }

      const onTriggerClick = () => openPanel()
      const onInputChange = () => {
        // User typed a hex in the trigger input.
        if (!hexInput) return
        const v = hexInput.value?.trim?.() ?? ''
        if (v === '') return
        if (!commitHex(v)) {
          hexInput.value = _ctx.value() as string
        }
      }
      const onInputKeydown = (e: KeyboardEvent) => {
        if (e.key === 'Escape' && open()) {
          e.preventDefault()
          dismissPanel()
        } else if (e.key === 'ArrowDown' && !open()) {
          e.preventDefault()
          openPanel()
        }
      }
      triggerEl.addEventListener('click', onTriggerClick)
      hexInput?.addEventListener('click', onTriggerClick)
      hexInput?.addEventListener('change', onInputChange)
      hexInput?.addEventListener('keydown', onInputKeydown)
      triggerEl.addEventListener('keydown', onInputKeydown)

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
        dismissPanel()
      }

      const rootEl = host.shadowRoot ?? host

      const onRangePointerDown = (e: Event) => {
        const t = e.target as HTMLElement | null
        if (t?.matches?.('input[type="range"]')) {
          gesture = true
          userEdited = true
        }
      }
      const onRangePointerUp = (e: Event) => {
        const t = e.target as HTMLElement | null
        if (!t?.matches?.('input[type="range"]') && !gesture) return
        if (gesture) {
          gesture = false
          commitFromSliders()
        }
      }
      // Keyboard slider steps: change fires after value settles.
      const onRangeChange = (e: Event) => {
        const t = e.target as HTMLElement | null
        if (!t?.matches?.('input[type="range"]')) return
        gesture = false
        userEdited = true
        commitFromSliders()
      }

      rootEl.addEventListener('click', onShadowClick)
      rootEl.addEventListener('change', onHexInputChange)
      rootEl.addEventListener('pointerdown', onRangePointerDown)
      rootEl.addEventListener('pointerup', onRangePointerUp)
      rootEl.addEventListener('change', onRangeChange)
      document.addEventListener('pointerdown', onDocPointer)
      // pointerup outside host still ends a drag that started on a range.
      document.addEventListener('pointerup', onRangePointerUp)

      host.expose!({
        focus: () => (hexInput ?? triggerEl).focus?.(),
        blur: () => (hexInput ?? triggerEl).blur?.(),
        open: openPanel,
        close: closePanel,
      })

      return () => {
        stopValueSync()
        sliderEffect()
        triggerEl.removeEventListener('click', onTriggerClick)
        triggerEl.removeEventListener('keydown', onInputKeydown)
        hexInput?.removeEventListener('click', onTriggerClick)
        hexInput?.removeEventListener('change', onInputChange)
        hexInput?.removeEventListener('keydown', onInputKeydown)
        rootEl.removeEventListener('click', onShadowClick)
        rootEl.removeEventListener('change', onHexInputChange)
        rootEl.removeEventListener('pointerdown', onRangePointerDown)
        rootEl.removeEventListener('pointerup', onRangePointerUp)
        rootEl.removeEventListener('change', onRangeChange)
        document.removeEventListener('pointerdown', onDocPointer)
        document.removeEventListener('pointerup', onRangePointerUp)
      }
    })

    const showHexUi = () => {
      const v = _ctx.showHex() as unknown
      return v !== false && v !== 'false' && v !== 0
    }

    return {
      open, hue, sat, light, hueText, satText, lightText, hexDraft,
      inputEl, hueId, satId, lightId, panelId, swatches, showHexUi,
    }
  })

</controller>
<column gap="0" sx={style.position(relative).w(full)}>
  <!-- Trigger: the color itself is the control (never "Edit color" text alone). -->
  <row
    data-color-trigger
    align="center"
    gap="2"
    sx={style.w(full).minH(size-h).padY(1).padX(1_5).bg(background).fg(foreground).borderWidth(1).border(border).radius(md).cursor(pointer).when({ focusWithin: style.outline(ring) })}
  >
    <button
      type="button"
      disabled={disabled() ? true : null}
      aria-haspopup="dialog"
      aria-expanded={open() ? 'true' : 'false'}
      aria-label="Choose color"
      popovertarget={panelId()}
      bind:this={inputEl}
      use:swatch={value() || '#000000'}
      shrink="0"
      sx={style.size(10).minW(10).radius(md).borderWidth(1).border(border).bg(bind(swatchColor)).cursor(pointer).pad(0)}
    ></button>
    <if {showHexUi()}>
      <input
        type="text"
        flush
        grow
        min-w="0"
        sx={style.fontFamily(mono).textCase(lowercase).cursor(pointer)}
        name={name()}
        placeholder="#000000"
        disabled={disabled()}
        autocomplete="off"
        aria-label="Hex color"
        bind:value={value}
      />
    </if>
    <if {!showHexUi()}>
      <span grow min-w="0" sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
        Pick color
      </span>
    </if>
  </row>
  <if {open() && !disabled()}>
    <div
      id={panelId()}
      data-color-panel
      popover="manual"
      use:anchor
      placement="bottom"
      role="dialog"
      aria-label="Choose color"
      sx={style.z(popover).w(panel-max-w-md).pad(3).bg(popover).fg(popover-foreground).borderWidth(1).border(border).radius(md).shadow(md).fontSize(size-fs)}
    >
      <column gap="3">
        <!-- LIVE preview of the color under edit (must update with every slider drag). -->
        <div
          use:swatch={value() || '#000000'}
          sx={style.w(full).h(16).radius(md).borderWidth(1).border(border).bg(bind(swatchColor)).shadow(sm)}
          aria-hidden="true"
        ></div>
        <grid cols="6" gap="2">
          <for {swatches()} as="sw" key={sw.hex}>
            <button
              type="button"
              aria-selected={sw.hex === value().toLowerCase() ? 'true' : 'false'}
              data-hex={sw.hex}
              aria-label={sw.label}
              use:swatch={sw.hex}
              sx={style.borderWidth(1).border(border-subtle).radius(sm).pad(0).aspect(square).cursor(pointer).bg(bind(swatchColor)).when({ selected: style.outline(primary) })}
            ></button>
          </for>
        </grid>
        <column gap="2">
          <row align="center" gap="2">
            <label for={hueId()} sx={style.w(4).fg(muted-foreground).fontSize(xs)}>H</label>
            <input
              id={hueId()}
              type="range"
              min="0"
              max="360"
              step="1"
              bind:value={hueText}
              data-axis="h"
              use:rangeFill
              grow
              min-w="0"
              aria-label="Hue"
            />
            <span sx={style.w(8).textAlign(end).fg(muted-foreground).fontSize(xs).fontFamily(mono)}>{hue()}°</span>
          </row>
          <row align="center" gap="2">
            <label for={satId()} sx={style.w(4).fg(muted-foreground).fontSize(xs)}>S</label>
            <input
              id={satId()}
              type="range"
              min="0"
              max="100"
              step="1"
              bind:value={satText}
              data-axis="s"
              use:rangeFill
              grow
              min-w="0"
              aria-label="Saturation"
            />
            <span sx={style.w(8).textAlign(end).fg(muted-foreground).fontSize(xs).fontFamily(mono)}>{sat()}%</span>
          </row>
          <row align="center" gap="2">
            <label for={lightId()} sx={style.w(4).fg(muted-foreground).fontSize(xs)}>L</label>
            <input
              id={lightId()}
              type="range"
              min="0"
              max="100"
              step="1"
              bind:value={lightText}
              data-axis="l"
              use:rangeFill
              grow
              min-w="0"
              aria-label="Lightness"
            />
            <span sx={style.w(8).textAlign(end).fg(muted-foreground).fontSize(xs).fontFamily(mono)}>{light()}%</span>
          </row>
        </column>
        <if {showHexUi()}>
          <row align="center" gap="2">
            <span sx={style.fg(muted-foreground).fontSize(xs)}>Hex</span>
            <input
              type="text"
              bind:value={hexDraft}
              spellcheck="false"
              autocomplete="off"
              aria-label="Hex color"
              grow min-w="0"
              sx={style.fontFamily(mono).textCase(lowercase).fontSize(xs)}
            />
          </row>
        </if>
      </column>
    </div>
  </if>
</column>
