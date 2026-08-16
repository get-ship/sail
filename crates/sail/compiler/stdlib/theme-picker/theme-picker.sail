<!--
  <theme-picker> — catalog + optional mode-only.
  Default: compact trigger, panel below (placement=bottom).
  inline: theme list stays in the page (horizontal catalog, no trigger).
  modeOnly: Light/Dark/System bar only.
  variant: icon (compact trigger) · menu-row (full-width option rows) ·
  segmented (mode bar as a connected pill). Swatches deferred.
  Behavior: use:themeSelect.
-->
<scope>
  props: {
    value = ''
    inline = false
    modeOnly = false
    // icon = compact trigger (default). menu-row = full-width option rows.
    // segmented = mode bar as a connected pill rail. Swatches deferred.
    variant: 'icon' | 'menu-row' | 'segmented' = 'icon'
    onThemeChange: (detail: { theme: string; mode: string; preference: string }) => void
  }

  style host = {group()}

  style trigger = {size(control-h-sm).pad(0).fg(foreground).bg(transparent).border({ color: border, width: hairline }).radius(full).cursor(pointer)}
  style modeTrigger = {size({ min-w: control-h-sm }).pad(0).fg(foreground).bg(transparent).border({ color: border, width: hairline }).radius(full).cursor(pointer)}
  style triggerIcon = {size(icon-size-md).display(block)}
  style glyph = {fg(foreground).type({ size: sm, leading: none })}

  style panel = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(0).margin(0).size({ w: panel-max-w-sm, max-w: panel-w-toast, min-w: 0, max-h: panel-vh-md }).overflow({ x: hidden, y: auto, overscroll: contain }).fg(popover-foreground).bg(popover).border({ color: border, width: hairline }).radius(lg).elevation(lg)}
  // Inline = in-flow horizontal catalog. group-inline also hides the trigger.
  style panelInline = {position({ scheme: static, inset: auto, l: auto, t: auto }).pad(0).margin(0).size({ w: full, max-w: none, min-w: 0, max-h: none }).overflow({ x: auto, y: hidden }).fg(popover-foreground).bg(popover).border({ color: border, width: 1 }).radius(lg).elevation(sm)}
  style panelModeOnly = {position({ scheme: static, inset: auto, l: auto, t: auto }).pad(0).margin(0).size({ w: auto, max-w: none, min-w: 0, max-h: none }).overflow({ all: hidden }).elevation(none).bg(transparent).border({ width: 0 })}
  style panelMenuRow = {size({ w: full, max-w: none, min-w: 0 })}

  style header = {size({ w: full, min-w: 0 }).pad({ x: 3, t: 3, b: 0 })}
  style headerEyebrow = {fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: label-transform, leading: none })}
  style headerTitle = {fg(popover-foreground).type({ family: heading, weight: bind(heading-weight), size: base, leading: tight, wrap: truncate })}
  style headerMeta = {fg(muted-foreground).type({ size: xs, leading: none })}

  // Filter tabs — always readable on popover chrome. Selected uses hover-bg +
  // primary underline (never primary fill — breaks on brut).
  style modeSeg = {size({ w: full, min-w: 0 }).pad({ x: 1 }).border({ width: 1, edge: b, color: border })}
  style modeSegRail = {pad(1).bg(muted).border({ color: border, width: 1 }).radius(md)}
  style modeSegBtn = {grow(1).size({ min-w: 0 }).pad({ y: 2, x: 2 }).fg(muted-foreground).bg(transparent).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).cursor(pointer).select(none).tapHighlight(none).type({ size: xs, weight: medium, align: center, wrap: nowrap }), when: {hover: fg(popover-foreground).bg(hover-bg), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset }), selected: fg(popover-foreground).bg(hover-bg).border({ color: primary, edge: b }).type({ weight: semibold })}}
  style modeSegBtnPill = {border({ width: 0 }).radius(sm), when: {selected: bg(background).fg(foreground).border({ width: 0 }).type({ weight: semibold })}}

  style catalogHead = {size({ w: full, min-w: 0 }).pad({ x: 3, t: 2, b: 1 }).fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: label-transform })}

  style panelList = {size({ w: full, min-w: 0 }).pad(1_5)}
  style panelListInline = {dir(row)}
  style option = {size({ w: full, min-w: 0 }).pad(0).overflow({ all: hidden }).bg(transparent).border({ color: transparent, width: hairline }).radius(md).cursor(pointer).type({ align: start })}
  style optionInline = {size({ w: auto, min-w: 36, max-w: 48 })}
  style optionRow = {size({ w: full, min-w: 0 }).radius(sm)}
  style optionCol = {size({ w: full, min-w: 0 })}

  // Live specimen — full-bleed via theme= (theme fonts + tokens).
  style swatch = {position(relative).size({ w: full, min-w: 0, h: 20, min-h: 20 }).overflow({ all: hidden }).radius(md).bg(background).fg(foreground).isolate()}
  style swatchRow = {size({ w: full, min-w: 0, h: auto, min-h: 0 }).bg(transparent).radius(none)}
  style overlay = {position({ scheme: absolute, inset: 0, z: 1 }).filter({ opacity: bind(surface-overlay-opacity), blend: bind(surface-overlay-blend) }).bg(bind(surface-overlay)).pointerEvents(none)}
  style specimen = {position({ scheme: relative, z: 2 }).size({ w: full, h: full, min-w: 0 }).pad(2_5)}
  style specimenRow = {pad({ y: 2, x: 2 })}
  style hideOnMenuRow = {display(none)}
  style title = {fg(foreground).type({ family: heading, weight: bind(heading-weight), size: sm, leading: tight, wrap: truncate })}
  // Chromatic trio: primary · secondary · accent
  style stripPrimary = {size({ h: 3, min-w: 0 }).radius(sm).bg(primary)}
  style stripSecondary = {size({ h: 3, min-w: 0 }).radius(sm).bg(secondary)}
  style stripAccent = {size({ h: 3, min-w: 0 }).radius(sm).bg(accent)}
  style btnGhost = {pad({ y: px, x: 1_5 }).radius(sm).bg(transparent).fg(foreground).border({ color: border, width: hairline }).type({ family: body, weight: medium, size: xs, leading: none })}
  style btnSolid = {pad({ y: px, x: 1_5 }).radius(sm).bg(primary).fg(primary-foreground).type({ family: body, weight: medium, size: xs, leading: none })}
  style checkMark = {fg(primary).type({ size: sm, weight: semibold, leading: none })}

  style focusRing = {when: {focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset })}}
  style triggerHover = {when: {hover: bg(hover-bg)}}
</scope>

<row inline use:themeSelect data-theme-picker="1" style(host, {match: { variant: { 'menu-row': size({ w: full }) } }})>
  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    label="Choose theme"
    style(trigger, triggerHover, focusRing, {when: {group-inline: display(none)}})
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(triggerIcon, {shrink(0)})>
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
    <span data-mode-icon="light" hidden="true" style(glyph, {shrink(0)})>☀</span>
    <span data-mode-icon="dark" hidden="true" style(glyph, {shrink(0)})>☾</span>
    <span data-mode-icon="system" hidden="true" style(glyph, {shrink(0)})>◐</span>
  </button>

  <!-- listbox role stamped by use:themeSelect (closed multi-part anatomy) -->
  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom"
    label="Theme"
    data-theme-listbox
    style(panel, {match: { inline: { true: panelInline }, modeOnly: { true: panelModeOnly }, variant: { 'menu-row': panelMenuRow } }})
  >
    <column gap="2" data-picker-header style(header)>
      <column gap="1" data-picker-title style({size({ min-w: 0 })})>
        <span style(headerEyebrow)>Active</span>
        <row align="center" gap="2" nowrap style({size({ min-w: 0 })})>
          <span data-selected-label style(headerTitle, {grow(1).size({ min-w: 0 })})>—</span>
          <span data-selected-tier style(headerMeta, {shrink(0)})></span>
        </row>
      </column>

      <!-- tablist/tab/option roles stamped by use:themeSelect -->
      <row
        gap="0"
        label="Filter themes by mode"
        data-mode-bar
        data-mode-segment
        nowrap
        style(modeSeg, {size({ min-w: 0 }), match: { variant: { segmented: modeSegRail }, modeOnly: { true: modeSegRail } }})
      >
        <button type="button" mode="light" selected="true" data-active="true" title="Light" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill }, modeOnly: { true: modeSegBtnPill } }})>
          Light
        </button>
        <button type="button" mode="dark" selected="false" title="Dark" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill }, modeOnly: { true: modeSegBtnPill } }})>
          Dark
        </button>
        <button type="button" mode="system" selected="false" title="System" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill }, modeOnly: { true: modeSegBtnPill } }})>
          System
        </button>
      </row>
    </column>

    <span data-catalog-label style(catalogHead)>Light themes</span>

    <column gap="2" data-theme-grid style(panelList, {size({ min-w: 0 }), when: {group-inline: panelListInline}})>
      <button
        type="button"
        value="sail"
        style(option, {when: {hover: border({ color: border }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset }), selected: border({ color: primary }), forced-colors: {when: {selected: border({ color: CanvasText })}}, group-inline: optionInline}, match: { variant: { 'menu-row': optionRow } }})
      >
        <column gap="0" style(optionCol, {size({ min-w: 0 })})>
          <column theme="sail" mode="light" hidden="true" style(swatch, {match: { variant: { 'menu-row': swatchRow } }})>
            <span style(overlay, {match: { variant: { 'menu-row': hideOnMenuRow } }})></span>
            <column gap="2" justify="between" style(specimen, {size({ min-w: 0 }), match: { variant: { 'menu-row': specimenRow } }})>
              <row align="center" gap="2" nowrap style({size({ min-w: 0, w: full })})>
                <span data-swatch-title style(title, {grow(1).size({ min-w: 0 })})>Sail</span>
                <span data-selected-check hidden="true" style(checkMark, {shrink(0)})>✓</span>
              </row>
              <row align="center" gap="1" nowrap hidden="true" style({size({ min-w: 0, w: full }), match: { variant: { 'menu-row': hideOnMenuRow } }})>
                <span style(stripPrimary, {grow(1).size({ min-w: 0 })})></span>
                <span style(stripSecondary, {grow(1).size({ min-w: 0 })})></span>
                <span style(stripAccent, {grow(1).size({ min-w: 0 })})></span>
              </row>
              <row align="center" gap="1" nowrap style({size({ min-w: 0 }), match: { variant: { 'menu-row': hideOnMenuRow } }})>
                <row inline nowrap style(btnGhost, {shrink(0)})>Cancel</row>
                <row inline nowrap style(btnSolid, {shrink(0)})>Continue</row>
              </row>
            </column>
          </column>
        </column>
      </button>
    </column>
  </div>
</row>
