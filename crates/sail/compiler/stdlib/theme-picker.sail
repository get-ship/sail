<!--
  <theme-picker> — catalog + optional mode-only.

  Full popover:
  - Light | Dark | System = filter tabs only (never paint the site)
  - One theme per row: live [data-theme-preview] specimen (theme fonts +
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

  const host = style.group()

  const trigger = style.size(control-h-sm).pad(0).fg(theme-picker-trigger-foreground).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer)
  const modeTrigger = style.size(control-h-sm).minW(control-h-sm).pad(0).fg(theme-picker-trigger-foreground).bg(transparent).borderWidth(border-width).border(border).radius(full).cursor(pointer)
  const triggerIcon = style.size(icon-size-md).display(block)
  const glyph = style.fontSize(sm).leading(none).fg(theme-picker-trigger-foreground)

  const panel = style.position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(0).w(panel-max-w-sm).maxW(panel-w-toast).minW(0).maxH(panel-vh-md).overflowX(hidden).overflowY(auto).overscroll(contain).fg(theme-picker-foreground).bg(theme-picker).borderWidth(border-width).border(border).radius(lg).shadow(lg).z(popover)

  const header = style.w(full).minW(0).padX(3).padT(3).padB(0)
  const headerEyebrow = style.fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform).fg(muted-foreground).leading(none)
  const headerTitle = style.fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(base).leading(tight).fg(theme-picker-foreground).truncate()
  const headerMeta = style.fontSize(xs).fg(muted-foreground).leading(none)

  // Filter tabs — ALWAYS readable on popover chrome.
  // Selected: strong fill using hover-bg + primary underline (never primary fill — breaks on brut).
  const modeSeg = style.w(full).minW(0).gap(0).padX(1).borderBWidth(1).borderB(border)
  const modeSegBtn = style.minW(0).padY(2).padX(2).fontSize(xs).fontWeight(medium).fg(muted-foreground).bg(transparent).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).cursor(pointer).textAlign(center).whitespace(nowrap).select(none).tapHighlight(none)

  const catalogHead = style.w(full).minW(0).padX(3).padT(2).padB(1).fontSize(xs).fontWeight(semibold).tracking(wide).textCase(label-transform).fg(muted-foreground)

  const panelList = style.w(full).minW(0).pad(1_5)
  const option = style.w(full).minW(0).pad(0).bg(transparent).borderWidth(border-width).border(transparent).radius(md).cursor(pointer).overflow(hidden).textAlign(start)
  const optionCol = style.w(full).minW(0)

  // Live specimen — full-bleed via data-theme-preview (theme fonts + tokens).
  // Same height as before (space-20); content rebalanced, not resized.
  const swatch = style.position(relative).w(full).minW(0).h(20).minH(20).radius(md).overflow(hidden).isolate().bg(background).fg(foreground)
  const overlay = style.position(absolute).inset(0).z(1).pointerEvents(none).opacity(bind(surface-overlay-opacity)).mixBlend(bind(surface-overlay-blend)).bg(bind(surface-overlay))
  const specimen = style.position(relative).z(2).w(full).h(full).minW(0).pad(2_5)
  // Name once — heading face of the previewed theme
  const title = style.fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(sm).leading(tight).fg(foreground).truncate()
  // Chromatic trio only: primary · secondary · accent (no card/fg that match the surface)
  const stripPrimary = style.h(3).radius(sm).minW(0).bg(primary)
  const stripSecondary = style.h(3).radius(sm).minW(0).bg(secondary)
  const stripAccent = style.h(3).radius(sm).minW(0).bg(accent)
  // Real control chrome fills the lower row (body face from the theme)
  const btnGhost = style.fontFamily(body).fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(transparent).fg(foreground).borderWidth(border-width).border(border)
  const btnSolid = style.fontFamily(body).fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(primary).fg(primary-foreground)
  const checkMark = style.fontSize(sm).fg(primary).fontWeight(semibold).leading(none)
</scope>

<row inline use:themeSelect sx={host} data-theme-picker="1">
  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    aria-label="Choose theme"
    sx={[
      trigger,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        group-inline: style.display(none),
      }),
    ]}
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" shrink="0" sx={triggerIcon}>
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
    aria-label="Color mode"
    hidden
    sx={[
      modeTrigger,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
      }),
    ]}
  >
    <span data-mode-icon="light" aria-hidden="true" shrink="0" sx={glyph}>☀</span>
    <span data-mode-icon="dark" hidden aria-hidden="true" shrink="0" sx={glyph}>☾</span>
    <span data-mode-icon="system" hidden aria-hidden="true" shrink="0" sx={glyph}>◐</span>
  </button>

  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom-end"
    role="listbox"
    aria-label="Theme"
    sx={[
      panel,
      when({
        group-inline: style.position(static).w(full).maxW(none).shadow(none).bg(transparent).borderWidth(0).pad(0).overflowX(hidden),
      }),
    ]}
  >
    <column gap="2" data-picker-header sx={header}>
      <column gap="1" min-w="0">
        <span sx={headerEyebrow}>Active</span>
        <row align="center" gap="2" nowrap min-w="0">
          <span data-selected-label grow min-w="0" sx={headerTitle}>—</span>
          <span data-selected-tier shrink="0" sx={headerMeta}></span>
        </row>
      </column>

      <!-- Filter tabs: selected = bold + fill + primary underline (readable on every host theme) -->
      <row gap="0" role="tablist" aria-label="Filter themes by mode" data-mode-bar data-mode-segment nowrap min-w="0" sx={modeSeg}>
        <button type="button" data-mode="light" role="tab" aria-selected="true" data-active="true" title="Show light themes" grow min-w="0" sx={[
          modeSegBtn,
          when({
            hover: style.fg(theme-picker-foreground).bg(theme-picker-option-hover-bg),
            focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
            selected: style.fg(theme-picker-foreground).fontWeight(semibold).bg(theme-picker-option-hover-bg).borderB(primary),
          }),
        ]}>
          Light
        </button>
        <button type="button" data-mode="dark" role="tab" aria-selected="false" title="Show dark themes" grow min-w="0" sx={[
          modeSegBtn,
          when({
            hover: style.fg(theme-picker-foreground).bg(theme-picker-option-hover-bg),
            focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
            selected: style.fg(theme-picker-foreground).fontWeight(semibold).bg(theme-picker-option-hover-bg).borderB(primary),
          }),
        ]}>
          Dark
        </button>
        <button type="button" data-mode="system" role="tab" aria-selected="false" title="Show dual-mode themes" grow min-w="0" sx={[
          modeSegBtn,
          when({
            hover: style.fg(theme-picker-foreground).bg(theme-picker-option-hover-bg),
            focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
            selected: style.fg(theme-picker-foreground).fontWeight(semibold).bg(theme-picker-option-hover-bg).borderB(primary),
          }),
        ]}>
          System
        </button>
      </row>
    </column>

    <span data-catalog-label sx={catalogHead}>Light themes</span>

    <column gap="2" data-theme-grid min-w="0" sx={panelList}>
      <button type="button" role="option" value="sail" sx={[
        option,
        when({
          hover: style.border(border),
          focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
          selected: style.border(primary),
          forced-colors: when({
            selected: style.border(CanvasText),
          }),
        }),
      ]}>
        <column gap="0" min-w="0" sx={optionCol}>
          <!-- Full-bleed live specimen: heading face + chromatic strips + controls -->
          <column data-theme-preview="sail" data-mode="light" aria-hidden="true" sx={swatch}>
            <span sx={overlay}></span>
            <column gap="2" justify="between" min-w="0" sx={specimen}>
              <row align="center" gap="2" nowrap min-w="0" w="full">
                <span data-swatch-title grow min-w="0" sx={title}>Sail</span>
                <span data-selected-check hidden aria-hidden="true" shrink="0" sx={checkMark}>✓</span>
              </row>
              <!-- primary · secondary · accent — equal strips, no surface-matching noise -->
              <row align="center" gap="1" nowrap min-w="0" w="full" aria-hidden="true">
                <span grow min-w="0" sx={stripPrimary}></span>
                <span grow min-w="0" sx={stripSecondary}></span>
                <span grow min-w="0" sx={stripAccent}></span>
              </row>
              <!-- Body-face controls: outline + solid (fills the bottom without fake pangrams) -->
              <row align="center" gap="1" nowrap min-w="0">
                <row inline nowrap shrink="0" sx={btnGhost}>Cancel</row>
                <row inline nowrap shrink="0" sx={btnSolid}>Continue</row>
              </row>
            </column>
          </column>
        </column>
      </button>
    </column>
  </div>
</row>
