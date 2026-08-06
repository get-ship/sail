<!--
  <theme-picker> — catalog + optional mode-only.

  Full popover:
  - Light | Dark | System = filter tabs only (never paint the site)
  - One theme per row: live [theme=…] specimen (theme fonts +
    primary/accent color dots + type sample) — name once, no "Theme" spam
  - Compact, no h-scroll, mobile-safe
  - Theme click applies; panel stays open

  modeOnly: sun/moon/system cycle (demos).

  Behavior: use:themeSelect.
-->
<scope>
  props: {
    value = ''
    inline = false
    modeOnly = false
  }

  const host = style({ paint: group() })

  const trigger = style({ box: size(control-h-sm).pad(0), paint: fg(theme-picker-trigger-foreground).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer) })
  const modeTrigger = style({ box: size(control-h-sm).minW(control-h-sm).pad(0), paint: fg(theme-picker-trigger-foreground).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer) })
  const triggerIcon = style({ box: size(icon-size-md).display(block) })
  const glyph = style({ paint: fg(theme-picker-trigger-foreground), text: fontSize(sm).leading(none) })

  const panel = style({ box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(0).w(panel-max-w-sm).maxW(panel-w-toast).minW(0).maxH(panel-vh-md).overflowX(hidden).overflowY(auto), paint: overscroll(contain).fg(theme-picker-foreground).bg(theme-picker).borderWidth(border-width).border(border).radius(lg).shadow(lg).z(popover) })

  const header = style({ box: w(full).minW(0).padX(3).padT(3).padB(0) })
  const headerEyebrow = style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform).leading(none) })
  const headerTitle = style({ paint: fg(theme-picker-foreground), text: fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(base).leading(tight).truncate() })
  const headerMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).leading(none) })

  // Filter tabs — ALWAYS readable on popover chrome.
  // Selected: strong fill using hover-bg + primary underline (never primary fill — breaks on brut).
  const modeSeg = style({ box: w(full).minW(0).gap(0).padX(1), paint: borderBWidth(1).borderB(border) })
  const modeSegBtn = style({ box: minW(0).padY(2).padX(2), paint: fg(muted-foreground).bg(transparent).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).cursor(pointer).select(none).tapHighlight(none), text: fontSize(xs).fontWeight(medium).textAlign(center).whitespace(nowrap) })

  const catalogHead = style({ box: w(full).minW(0).padX(3).padT(2).padB(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform) })

  const panelList = style({ box: w(full).minW(0).pad(1_5) })
  const option = style({ box: w(full).minW(0).pad(0).overflow(hidden), paint: bg(transparent).borderWidth(border-width).border(transparent).radius(md).cursor(pointer), text: textAlign(start) })
  const optionCol = style({ box: w(full).minW(0) })

  // Live specimen — full-bleed via theme= (theme fonts + tokens).
  // Same height as before (space-20); content rebalanced, not resized.
  const swatch = style({ box: position(relative).w(full).minW(0).h(20).minH(20).overflow(hidden), paint: radius(md).isolate().bg(background).fg(foreground) })
  const overlay = style({ box: position(absolute).inset(0), paint: z(1).pointerEvents(none).opacity(bind(surface-overlay-opacity)).mixBlend(bind(surface-overlay-blend)).bg(bind(surface-overlay)) })
  const specimen = style({ box: position(relative).w(full).h(full).minW(0).pad(2_5), paint: z(2) })
  // Name once — heading face of the previewed theme
  const title = style({ paint: fg(foreground), text: fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(sm).leading(tight).truncate() })
  // Chromatic trio only: primary · secondary · accent (no card/fg that match the surface)
  const stripPrimary = style({ box: h(3).minW(0), paint: radius(sm).bg(primary) })
  const stripSecondary = style({ box: h(3).minW(0), paint: radius(sm).bg(secondary) })
  const stripAccent = style({ box: h(3).minW(0), paint: radius(sm).bg(accent) })
  // Real control chrome fills the lower row (body face from the theme)
  const btnGhost = style({ box: padY(px).padX(1_5), paint: radius(sm).bg(transparent).fg(foreground).borderWidth(border-width).border(border), text: fontFamily(body).fontWeight(medium).fontSize(xs).leading(none) })
  const btnSolid = style({ box: padY(px).padX(1_5), paint: radius(sm).bg(primary).fg(primary-foreground), text: fontFamily(body).fontWeight(medium).fontSize(xs).leading(none) })
  const checkMark = style({ paint: fg(primary), text: fontSize(sm).fontWeight(semibold).leading(none) })
</scope>

<row inline use:themeSelect data-theme-picker="1">
  style(host)

  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    label="Choose theme"

  >
    style([
      trigger,
      when({
        hover: style({ paint: bg(theme-picker-option-hover-bg) }),
        focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
        group-inline: style({ box: display(none) }),
      }),
    ])

    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" hidden="true">
      style([ style(triggerIcon), style({ box: shrink(0) }) ])

      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>

  <button
    type="button"
    data-picker-trigger="mode"
    data-mode-only-trigger="1"
    label="Color mode"
    hidden

  >
    style([
      modeTrigger,
      when({
        hover: style({ paint: bg(theme-picker-option-hover-bg) }),
        focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
      }),
    ])

    <span data-mode-icon="light" hidden="true">
      style([ style(glyph), style({ box: shrink(0) }) ])
☀</span>
    <span data-mode-icon="dark" hidden hidden="true">
      style([ style(glyph), style({ box: shrink(0) }) ])
☾</span>
    <span data-mode-icon="system" hidden hidden="true">
      style([ style(glyph), style({ box: shrink(0) }) ])
◐</span>
  </button>

  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom-end"
    role="listbox"
    label="Theme"

  >
    style([
      panel,
      compound({ inline: true }, style({ box: position(static).w(full).maxW(none).pad(0).overflowX(hidden), paint: shadow(none).bg(transparent).borderWidth(0) })),
    ])

    <column gap="2" data-picker-header>
      style(header)

      <column gap="1">
        style({ box: minW(0) })

        <span>
          style(headerEyebrow)
Active</span>
        <row align="center" gap="2" nowrap>
          style({ box: minW(0) })

          <span data-selected-label>
            style([ style(headerTitle), style({ box: grow(1).minW(0) }) ])
—</span>
          <span data-selected-tier>
            style([ style(headerMeta), style({ box: shrink(0) }) ])
</span>
        </row>
      </column>

      <!-- Filter tabs: selected = bold + fill + primary underline (readable on every host theme) -->
      <row gap="0" role="tablist" label="Filter themes by mode" data-mode-bar data-mode-segment nowrap>
        style([ style(modeSeg), style({ box: minW(0) }) ])

        <button type="button" data-mode="light" role="tab" selected="true" data-active="true" title="Show light themes">
          style([ style({ box: grow(1).minW(0) }), 
          modeSegBtn,
          when({
            hover: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg) }),
            focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
            selected: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg).borderB(primary), text: fontWeight(semibold) }),
          }),
        ])

          Light
        </button>
        <button type="button" data-mode="dark" role="tab" selected="false" title="Show dark themes">
          style([ style({ box: grow(1).minW(0) }), 
          modeSegBtn,
          when({
            hover: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg) }),
            focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
            selected: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg).borderB(primary), text: fontWeight(semibold) }),
          }),
        ])

          Dark
        </button>
        <button type="button" data-mode="system" role="tab" selected="false" title="Show dual-mode themes">
          style([ style({ box: grow(1).minW(0) }), 
          modeSegBtn,
          when({
            hover: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg) }),
            focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
            selected: style({ paint: fg(theme-picker-foreground).bg(theme-picker-option-hover-bg).borderB(primary), text: fontWeight(semibold) }),
          }),
        ])

          System
        </button>
      </row>
    </column>

    <span data-catalog-label>
      style(catalogHead)
Light themes</span>

    <column gap="2" data-theme-grid>
      style([ style(panelList), style({ box: minW(0) }) ])

      <button type="button" role="option" value="sail">
        style([
        option,
        when({
          hover: style({ paint: border(border) }),
          focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) }),
          selected: style({ paint: border(primary) }),
          forced-colors: when({
            selected: style({ paint: border(CanvasText) }),
          }),
        }),
      ])

        <column gap="0">
          style([ style(optionCol), style({ box: minW(0) }) ])

          <!-- Full-bleed live specimen: heading face + chromatic strips + controls -->
          <column theme="sail" data-mode="light" hidden="true">
            style(swatch)

            <span>
              style(overlay)
</span>
            <column gap="2" justify="between">
              style([ style(specimen), style({ box: minW(0) }) ])

              <row align="center" gap="2" nowrap>
                style({ box: minW(0).w(full) })

                <span data-swatch-title>
                  style([ style(title), style({ box: grow(1).minW(0) }) ])
Sail</span>
                <span data-selected-check hidden hidden="true">
                  style([ style(checkMark), style({ box: shrink(0) }) ])
✓</span>
              </row>
              <!-- primary · secondary · accent — equal strips, no surface-matching noise -->
              <row align="center" gap="1" nowrap hidden="true">
                style({ box: minW(0).w(full) })

                <span>
                  style([ style(stripPrimary), style({ box: grow(1).minW(0) }) ])
</span>
                <span>
                  style([ style(stripSecondary), style({ box: grow(1).minW(0) }) ])
</span>
                <span>
                  style([ style(stripAccent), style({ box: grow(1).minW(0) }) ])
</span>
              </row>
              <!-- Body-face controls: outline + solid (fills the bottom without fake pangrams) -->
              <row align="center" gap="1" nowrap>
                style({ box: minW(0) })

                <row inline nowrap>
                  style([ style(btnGhost), style({ box: shrink(0) }) ])
Cancel</row>
                <row inline nowrap>
                  style([ style(btnSolid), style({ box: shrink(0) }) ])
Continue</row>
              </row>
            </column>
          </column>
        </column>
      </button>
    </column>
  </div>
</row>
