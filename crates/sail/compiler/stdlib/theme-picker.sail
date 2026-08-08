<!--
  <theme-picker> — catalog + optional mode-only.
  Full popover: Light|Dark|System filter tabs (never paint the site); one
  live [theme=…] specimen per row. modeOnly: sun/moon/system cycle.
  Behavior: use:themeSelect.
-->
<scope>
  props: {
    value = ''
    inline = false
    modeOnly = false
    onThemeChange: (detail: { theme: string; mode: string; preference: string }) => void
  }

  style host = { paint: group() }

  style trigger = {
    box: size(control-h-sm).pad(0),
    paint: fg(theme-picker.trigger-fg).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer),
  }
  style modeTrigger = {
    box: size(control-h-sm).minW(control-h-sm).pad(0),
    paint: fg(theme-picker.trigger-fg).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer),
  }
  style triggerIcon = { box: size(icon-size-md).display(block) }
  style glyph = { paint: fg(theme-picker.trigger-fg), text: fontSize(sm).leading(none) }

  style panel = {
    box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(0).w(panel-max-w-sm).maxW(panel-w-toast).minW(0).maxH(panel-vh-md).overflowX(hidden).overflowY(auto),
    paint: overscroll(contain).fg(theme-picker.foreground).bg(theme-picker).borderWidth(border-width).border(border).radius(lg).shadow(lg).z(popover),
  }

  style header = { box: w(full).minW(0).padX(3).padT(3).padB(0) }
  style headerEyebrow = {
    paint: fg(theme-picker.option-fg),
    text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform).leading(none),
  }
  style headerTitle = {
    paint: fg(theme-picker.foreground),
    text: fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(base).leading(tight).truncate(),
  }
  style headerMeta = { paint: fg(theme-picker.option-fg), text: fontSize(xs).leading(none) }

  // Filter tabs — always readable on popover chrome. Selected uses hover-bg +
  // primary underline (never primary fill — breaks on brut).
  style modeSeg = { box: w(full).minW(0).gap(0).padX(1), paint: borderBWidth(1).borderB(border) }
  style modeSegBtn = {
    box: grow(1).minW(0).padY(2).padX(2),
    paint: fg(theme-picker.option-fg).bg(transparent).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).cursor(pointer).select(none).tapHighlight(none),
    text: fontSize(xs).fontWeight(medium).textAlign(center).whitespace(nowrap),
    when: {
      hover: { paint: fg(theme-picker.foreground).bg(hover-bg) },
      focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) },
      selected: { paint: fg(theme-picker.foreground).bg(hover-bg).borderB(primary), text: fontWeight(semibold) },
    },
  }

  style catalogHead = {
    box: w(full).minW(0).padX(3).padT(2).padB(1),
    paint: fg(theme-picker.option-fg),
    text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform),
  }

  style panelList = { box: w(full).minW(0).pad(1_5) }
  style option = {
    box: w(full).minW(0).pad(0).overflow(hidden),
    paint: bg(transparent).borderWidth(border-width).border(transparent).radius(md).cursor(pointer),
    text: textAlign(start),
  }
  style optionCol = { box: w(full).minW(0) }

  // Live specimen — full-bleed via theme= (theme fonts + tokens).
  style swatch = {
    box: position(relative).w(full).minW(0).h(20).minH(20).overflow(hidden),
    paint: radius(md).isolate().bg(background).fg(foreground),
  }
  style overlay = {
    box: position(absolute).inset(0),
    paint: z(1).pointerEvents(none).opacity(bind(surface-overlay-opacity)).mixBlend(bind(surface-overlay-blend)).bg(bind(surface-overlay)),
  }
  style specimen = { box: position(relative).w(full).h(full).minW(0).pad(2_5), paint: z(2) }
  style title = {
    paint: fg(foreground),
    text: fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(sm).leading(tight).truncate(),
  }
  // Chromatic trio: primary · secondary · accent
  style stripPrimary = { box: h(3).minW(0), paint: radius(sm).bg(primary) }
  style stripSecondary = { box: h(3).minW(0), paint: radius(sm).bg(secondary) }
  style stripAccent = { box: h(3).minW(0), paint: radius(sm).bg(accent) }
  style btnGhost = {
    box: padY(px).padX(1_5),
    paint: radius(sm).bg(transparent).fg(foreground).borderWidth(border-width).border(border),
    text: fontFamily(body).fontWeight(medium).fontSize(xs).leading(none),
  }
  style btnSolid = {
    box: padY(px).padX(1_5),
    paint: radius(sm).bg(primary).fg(primary-foreground),
    text: fontFamily(body).fontWeight(medium).fontSize(xs).leading(none),
  }
  style checkMark = { paint: fg(primary), text: fontSize(sm).fontWeight(semibold).leading(none) }

  style focusRing = {
    when: { focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) } },
  }
  style triggerHover = {
    when: { hover: { paint: bg(hover-bg) } },
  }
</scope>

<row inline use:themeSelect data-theme-picker="1" style(host)>
  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    label="Choose theme"
    style(trigger, triggerHover, focusRing, {
      when: { group-inline: { box: display(none) } },
    })
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(triggerIcon, { box: shrink(0) })>
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
    style(modeTrigger, triggerHover, focusRing)
  >
    <span data-mode-icon="light" hidden="true" style(glyph, { box: shrink(0) })>☀</span>
    <span data-mode-icon="dark" hidden="true" style(glyph, { box: shrink(0) })>☾</span>
    <span data-mode-icon="system" hidden="true" style(glyph, { box: shrink(0) })>◐</span>
  </button>

  <!-- listbox role stamped by use:themeSelect (closed multi-part anatomy) -->
  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom-end"
    label="Theme"
    data-theme-listbox
    style(panel, {
      compound: [
        {
          inline: true,
          box: position(static).w(full).maxW(none).pad(0).overflowX(hidden),
          paint: shadow(none).bg(transparent).borderWidth(0),
        },
      ],
    })
  >
    <column gap="2" data-picker-header style(header)>
      <column gap="1" style({ box: minW(0) })>
        <span style(headerEyebrow)>Active</span>
        <row align="center" gap="2" nowrap style({ box: minW(0) })>
          <span data-selected-label style(headerTitle, { box: grow(1).minW(0) })>—</span>
          <span data-selected-tier style(headerMeta, { box: shrink(0) })></span>
        </row>
      </column>

      <!-- tablist/tab/option roles stamped by use:themeSelect -->
      <row
        gap="0"
        label="Filter themes by mode"
        data-mode-bar
        data-mode-segment
        nowrap
        style(modeSeg, { box: minW(0) })
      >
        <button type="button" data-mode="light" selected="true" data-active="true" title="Show light themes" style(modeSegBtn)>
          Light
        </button>
        <button type="button" data-mode="dark" selected="false" title="Show dark themes" style(modeSegBtn)>
          Dark
        </button>
        <button type="button" data-mode="system" selected="false" title="Show dual-mode themes" style(modeSegBtn)>
          System
        </button>
      </row>
    </column>

    <span data-catalog-label style(catalogHead)>Light themes</span>

    <column gap="2" data-theme-grid style(panelList, { box: minW(0) })>
      <button
        type="button"
        value="sail"
        style(option, {
          when: {
            hover: { paint: border(border) },
            focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(ring-offset) },
            selected: { paint: border(primary) },
            forced-colors: { when: { selected: { paint: border(CanvasText) } } },
          },
        })
      >
        <column gap="0" style(optionCol, { box: minW(0) })>
          <column theme="sail" data-mode="light" hidden="true" style(swatch)>
            <span style(overlay)></span>
            <column gap="2" justify="between" style(specimen, { box: minW(0) })>
              <row align="center" gap="2" nowrap style({ box: minW(0).w(full) })>
                <span data-swatch-title style(title, { box: grow(1).minW(0) })>Sail</span>
                <span data-selected-check hidden="true" style(checkMark, { box: shrink(0) })>✓</span>
              </row>
              <row align="center" gap="1" nowrap hidden="true" style({ box: minW(0).w(full) })>
                <span style(stripPrimary, { box: grow(1).minW(0) })></span>
                <span style(stripSecondary, { box: grow(1).minW(0) })></span>
                <span style(stripAccent, { box: grow(1).minW(0) })></span>
              </row>
              <row align="center" gap="1" nowrap style({ box: minW(0) })>
                <row inline nowrap style(btnGhost, { box: shrink(0) })>Cancel</row>
                <row inline nowrap style(btnSolid, { box: shrink(0) })>Continue</row>
              </row>
            </column>
          </column>
        </column>
      </button>
    </column>
  </div>
</row>
