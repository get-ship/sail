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
    /** swatch = color fill only (no hex field / "Pick color" chrome) — for design studio cards. */
    variant: 'default' | 'swatch' = 'default'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
  }
</scope>
<controller>

import type { Action, ControllerScope } from '@ship/sail-runtime'
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
        ;(host as any).onChange?.({
          value: hex,
          token: host.getAttribute('token') ?? host.getAttribute('token-path') ?? undefined,
        })
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

      const rootNode = () => (host.shadowRoot ?? host) as ParentNode
      const getTrigger = () =>
        (inputEl() as HTMLElement | null) ||
        (rootNode().querySelector('[data-color-trigger-btn]') as HTMLElement | null)
      const getHexInput = () =>
        rootNode().querySelector(
          'input[type="text"][aria-label="Hex color"]',
        ) as HTMLInputElement | null

      const showPanelPopover = (): boolean => {
        const panel = rootNode().querySelector('[data-color-panel]') as HTMLElement & {
          showPopover?: () => void
        } | null
        if (!panel) return false
        try {
          panel.showPopover?.()
        } catch {
          /* ignore API errors */
        }
        // popover API can leave panel closed if showPopover races mount —
        // force top-layer visibility as last resort for studio pickers.
        const st = getComputedStyle(panel)
        if (st.display === 'none' || !(panel as HTMLElement & { matches: (s: string) => boolean }).matches?.(':popover-open')) {
          try {
            // Re-try showPopover once more after style read
            panel.showPopover?.()
          } catch {
            /* ignore */
          }
        }
        return true
      }

      /** Panel is gated by open() — if-render is async; retry until mounted. */
      const ensurePanelShown = (attempts = 16) => {
        if (showPanelPopover()) return
        if (attempts <= 0) return
        requestAnimationFrame(() => ensurePanelShown(attempts - 1))
      }

      const openPanel = () => {
        if (_ctx.disabled() || open()) return
        const cur = normHex(String(_ctx.value() ?? '')) || String(_ctx.value() ?? '').toLowerCase()
        openHex = isHex(cur) ? cur : ''
        userEdited = false
        if (openHex) syncFromHex(openHex)
        open.set(true)
        ensurePanelShown()
      }
      const closePanel = () => {
        if (!open()) return
        open.set(false)
        userEdited = false
        openHex = ''
        const panel = rootNode().querySelector('[data-color-panel]') as HTMLElement & {
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

      const onTriggerClick = (e: Event) => {
        e.preventDefault()
        e.stopPropagation()
        openPanel()
      }
      const onInputChange = () => {
        const hexInput = getHexInput()
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

      // Attach listeners when trigger exists (capture may lag mount by a frame).
      let attached: HTMLElement | null = null
      let hexAttached: HTMLInputElement | null = null
      const attachTrigger = () => {
        const triggerEl = getTrigger()
        if (!triggerEl || triggerEl === attached) return !!triggerEl
        if (attached) {
          attached.removeEventListener('click', onTriggerClick)
          attached.removeEventListener('keydown', onInputKeydown)
        }
        triggerEl.addEventListener('click', onTriggerClick)
        triggerEl.addEventListener('keydown', onInputKeydown)
        attached = triggerEl
        const hexInput = getHexInput()
        if (hexInput && hexInput !== hexAttached) {
          hexAttached?.removeEventListener('click', onTriggerClick)
          hexAttached?.removeEventListener('change', onInputChange)
          hexAttached?.removeEventListener('keydown', onInputKeydown)
          hexInput.addEventListener('click', onTriggerClick)
          hexInput.addEventListener('change', onInputChange)
          hexInput.addEventListener('keydown', onInputKeydown)
          hexAttached = hexInput
        }
        return true
      }
      // Immediate + rAF retries (swatch if-branch capture)
      if (!attachTrigger()) {
        let tries = 20
        const tick = () => {
          if (attachTrigger() || tries-- <= 0) return
          requestAnimationFrame(tick)
        }
        requestAnimationFrame(tick)
      }

      const onShadowClick = (e: Event) => {
        const t = e.target as HTMLElement | null
        if (!t) return
        // Delegate trigger open — survives if/branch re-renders (no capture race).
        const trigger = t.closest?.('[data-color-trigger-btn], [data-color-trigger]') as HTMLElement | null
        if (trigger && !t.closest?.('[data-color-panel]')) {
          // Don't steal clicks from hex text input (default variant).
          if (!(t instanceof HTMLInputElement)) {
            onTriggerClick(e)
            return
          }
        }
        const sw = t.closest?.('button[data-hex]') as HTMLElement | null
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
        focus: () => (getHexInput() ?? getTrigger())?.focus?.(),
        blur: () => (getHexInput() ?? getTrigger())?.blur?.(),
        open: openPanel,
        close: closePanel,
      })

      return () => {
        stopValueSync()
        sliderEffect()
        if (attached) {
          attached.removeEventListener('click', onTriggerClick)
          attached.removeEventListener('keydown', onInputKeydown)
        }
        if (hexAttached) {
          hexAttached.removeEventListener('click', onTriggerClick)
          hexAttached.removeEventListener('change', onInputChange)
          hexAttached.removeEventListener('keydown', onInputKeydown)
        }
        rootEl.removeEventListener('click', onShadowClick)
        rootEl.removeEventListener('change', onHexInputChange)
        rootEl.removeEventListener('pointerdown', onRangePointerDown)
        rootEl.removeEventListener('pointerup', onRangePointerUp)
        rootEl.removeEventListener('change', onRangeChange)
        document.removeEventListener('pointerdown', onDocPointer)
        document.removeEventListener('pointerup', onRangePointerUp)
      }
    })

    const isSwatchOnly = () => {
      const v = _ctx.variant() as unknown
      return v === 'swatch'
    }
    const showHexUi = () => {
      if (isSwatchOnly()) return false
      const v = _ctx.showHex() as unknown
      return v !== false && v !== 'false' && v !== 0
    }

    const captureInput: Action<void> = (node) => {
      inputEl.set(node as HTMLElement)
      return { destroy: () => inputEl.set(null) }
    }
    return {
      open, hue, sat, light, hueText, satText, lightText, hexDraft,
      captureInput, hueId, satId, lightId, panelId, swatches, showHexUi, isSwatchOnly,
    }
  })

</controller>
<!--
  Host is display:contents — host `sx` size is ignored. Swatch mode must absolute-fill
  a position:relative parent (brand/hero stack or token chip wrapper); % width collapses
  to 0 in auto grid tracks (2px border-only hit target → "picker does not launch").
-->
<if {isSwatchOnly()}>
  <column gap="0" data-color-picker-root data-variant="swatch" style({ box: position(absolute).top(0).left(0).right(0).bottom(0).minW(0).minH(0) })>
    <button
      type="button"
      data-color-trigger
      data-color-trigger-btn
      data-variant="swatch"
      size="sm"
      variant="ghost"
      disabled={disabled() ? true : null}
      aria-haspopup="dialog"
      aria-expanded={open() ? 'true' : 'false'}
      aria-label="Choose color"
      use:captureInput
      use:swatch={value() || '#000000'} style({ box: w(full).h(full).minW(0).minH(0).pad(0), paint: radius(md).borderWidth(1).border(border-subtle).bg(bind(swatchColor)).cursor(pointer).shadow(none), when: { focusWithin: { paint: outline(ring) } } })></button>
    <if {open() && !disabled()}>
      <div
        id={panelId()}
        data-color-panel
        popover="manual"
        use:anchor
        placement="bottom-start"
        role="dialog"
        aria-label="Choose color" style({ box: w(panel-max-w-md).pad(3), paint: z(popover).bg(popover).fg(popover-foreground).borderWidth(1).border(border).radius(md).shadow(md) })>        <span style({ box: display(contents), text: fontSize(size-fs) })>          <column gap="3">
          <div
            use:swatch={value() || '#000000'}

            aria-hidden="true" style({ box: w(full).h(16), paint: radius(md).borderWidth(1).border(border).bg(bind(swatchColor)).shadow(sm) })></div>
          <grid cols="6" gap="2">
            <for {swatches() as sw} key={sw.hex}>
              <button
                type="button"
                aria-selected={sw.hex === value().toLowerCase() ? 'true' : 'false'}
                data-hex={sw.hex}
                aria-label={sw.label}
                use:swatch={sw.hex} style({ box: pad(0).aspect(square), paint: borderWidth(1).border(border-subtle).radius(sm).cursor(pointer).bg(bind(swatchColor)), when: { selected: { paint: outline(primary) } } })></button>
            </for>
          </grid>
          <column gap="2">
            <row align="center" gap="2">
              <label for={hueId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>H</label>
              <input id={hueId()} type="range" min="0" max="360" step="1" bind:value={hueText} data-axis="h" use:rangeFill aria-label="Hue" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{hue()}°</span>
            </row>
            <row align="center" gap="2">
              <label for={satId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>S</label>
              <input id={satId()} type="range" min="0" max="100" step="1" bind:value={satText} data-axis="s" use:rangeFill aria-label="Saturation" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{sat()}%</span>
            </row>
            <row align="center" gap="2">
              <label for={lightId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>L</label>
              <input id={lightId()} type="range" min="0" max="100" step="1" bind:value={lightText} data-axis="l" use:rangeFill aria-label="Lightness" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{light()}%</span>
            </row>
          </column>
          <!-- Always expose hex — studio shows hex on tokens; picker must accept paste. -->
          <row align="center" gap="2">
            <span style({ paint: fg(muted-foreground), text: fontSize(xs) })>Hex</span>
            <input type="text" bind:value={hexDraft} spellcheck="false" autocomplete="off" aria-label="Hex color" style({ box: grow(1).minW(0), text: fontFamily(mono).textCase(lowercase).fontSize(xs) })>
          </row>
        </column>
        </span>
</div>
    </if>
  </column>
</if>
<if {!isSwatchOnly()}>
  <column gap="0" data-color-picker-root data-variant="default" style({ box: position(relative).w(full).minH(0) })>
    <row
      data-color-trigger
      align="center"
      gap="2" style({ box: w(full).minH(size-h).padY(1).padX(1_5), paint: bg(background).fg(foreground).borderWidth(1).border(border).radius(md).cursor(pointer), when: { focusWithin: { paint: outline(ring) } } })>
      <button type="button" data-color-trigger-btn disabled={disabled() ? true : null} aria-haspopup="dialog" aria-expanded={open() ? 'true' : 'false'} aria-label="Choose color" use:captureInput use:swatch={value() || '#000000'} style({  box: shrink(0).size(10).minW(10).pad(0), paint: radius(md).borderWidth(1).border(border).bg(bind(swatchColor)).cursor(pointer)  })></button>
      <if {showHexUi()}>
        <input type="text" flush name={name()} placeholder="#000000" disabled={disabled()} autocomplete="off" aria-label="Hex color" bind:value={value} style({ box: grow(1).minW(0), paint: cursor(pointer), text: fontFamily(mono).textCase(lowercase) })>
      </if>
    </row>
    <if {open() && !disabled()}>
      <div
        id={panelId()}
        data-color-panel
        popover="manual"
        use:anchor
        placement="bottom-start"
        role="dialog"
        aria-label="Choose color" style({ box: w(panel-max-w-md).pad(3), paint: z(popover).bg(popover).fg(popover-foreground).borderWidth(1).border(border).radius(md).shadow(md) })>        <span style({ box: display(contents), text: fontSize(size-fs) })>          <column gap="3">
          <div
            use:swatch={value() || '#000000'}

            aria-hidden="true" style({ box: w(full).h(16), paint: radius(md).borderWidth(1).border(border).bg(bind(swatchColor)).shadow(sm) })></div>
          <grid cols="6" gap="2">
            <for {swatches() as sw} key={sw.hex}>
              <button
                type="button"
                aria-selected={sw.hex === value().toLowerCase() ? 'true' : 'false'}
                data-hex={sw.hex}
                aria-label={sw.label}
                use:swatch={sw.hex} style({ box: pad(0).aspect(square), paint: borderWidth(1).border(border-subtle).radius(sm).cursor(pointer).bg(bind(swatchColor)), when: { selected: { paint: outline(primary) } } })></button>
            </for>
          </grid>
          <column gap="2">
            <row align="center" gap="2">
              <label for={hueId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>H</label>
              <input id={hueId()} type="range" min="0" max="360" step="1" bind:value={hueText} data-axis="h" use:rangeFill aria-label="Hue" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{hue()}°</span>
            </row>
            <row align="center" gap="2">
              <label for={satId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>S</label>
              <input id={satId()} type="range" min="0" max="100" step="1" bind:value={satText} data-axis="s" use:rangeFill aria-label="Saturation" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{sat()}%</span>
            </row>
            <row align="center" gap="2">
              <label for={lightId()} style({ box: w(4), paint: fg(muted-foreground), text: fontSize(xs) })>L</label>
              <input id={lightId()} type="range" min="0" max="100" step="1" bind:value={lightText} data-axis="l" use:rangeFill aria-label="Lightness" style({ box: grow(1).minW(0) })>
              <span style({ box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontSize(xs).fontFamily(mono) })>{light()}%</span>
            </row>
          </column>
          <if {showHexUi()}>
            <row align="center" gap="2">
              <span style({ paint: fg(muted-foreground), text: fontSize(xs) })>Hex</span>
              <input type="text" bind:value={hexDraft} spellcheck="false" autocomplete="off" aria-label="Hex color" style({ box: grow(1).minW(0), text: fontFamily(mono).textCase(lowercase).fontSize(xs) })>
            </row>
          </if>
        </column>
        </span>
</div>
    </if>
  </column>
</if>
