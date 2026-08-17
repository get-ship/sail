<!--
  <theme-picker> — catalog, or `mode` (one icon that cycles light/dark/system).
  Axes: size full|compact · chrome popover|inline · orientation horizontal|vertical.
  chrome=inline is in-flow (wraps / overflows with the surroundings).
  Cards always paint via theme= + mode= — same specimen as the popover.
  `mode-only` and `inline` remain aliases.
  variant: icon · menu-row · segmented. Behavior: use:themeSelect.

  Host is <row> — never <row inline>. [inline] is the product alias so
  group-inline / group-chrome-inline can hide the popover on first paint.
  Mode trigger + inline cards live inside the popover so a closed picker
  is only the trigger. use:themeSelect lifts them when chrome=inline / mode.
-->
<scope>
  props: {
    value = ''
    chrome: 'popover' | 'inline' = 'popover'
    size: 'full' | 'compact' = 'full'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    inline = false
    mode = false
    modeOnly = false
    variant: 'icon' | 'menu-row' | 'segmented' = 'icon'
    onThemeChange: (detail: { theme: string; mode: string; preference: string }) => void
  }

  style host = {group()}
  style hostInline = {size({ w: 100%, max-w: 100%, min-w: 0 })}
  style hostVert = {dir(column)}
  style hostHoriz = {dir(row)}
  style gone = {display(none)}
  style flexOn = {display(flex)}
  style modeOn = {display(inline-flex)}
  style stack = {dir(column)}
  style optionInline = {size({ w: 36, min-w: 28, max-w: 100% })}
  style optionStretch = {size({ w: 100%, min-w: 0 })}
  style swatchCompact = {size({ h: 14, min-h: 14 })}

  style trigger = {size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm }).pad(1_5).fg(foreground).bg(transparent).border({ color: border, width: hairline }).radius(full).cursor(pointer), when: {group-inline: gone, group-chrome-inline: gone, group-mode: gone, group-mode-only: gone}}
  style modeTrigger = {display(none).size({ min-w: control-h-sm, h: control-h-sm }).pad(1_5).fg(foreground).bg(transparent).border({ color: border, width: hairline }).radius(full).cursor(pointer), when: {group-mode: modeOn, group-mode-only: modeOn}}
  style triggerIcon = {size(icon-size-sm).display(block)}
  style glyph = {fg(foreground).type({ size: sm, leading: none })}

  style panel = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(0).margin(0).size({ w: panel-max-w-sm, max-w: 24rem, min-w: 0, max-h: panel-vh-md }).overflow({ x: hidden, y: auto, overscroll: contain }).fg(popover-foreground).bg(popover).border({ color: border, width: hairline }).radius(lg).elevation(lg), when: {group-inline: gone, group-chrome-inline: gone, group-mode: gone, group-mode-only: gone}}
  style panelMenuRow = {size({ w: 100%, max-w: none, min-w: 0 })}

  style header = {size({ w: 100%, min-w: 0 }).pad({ x: 3, t: 3, b: 0 })}
  style headerEyebrow = {fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: label-transform, leading: none })}
  style headerTitle = {fg(popover-foreground).type({ family: heading, weight: bind(heading-weight), size: base, leading: tight, wrap: truncate })}
  style headerMeta = {fg(muted-foreground).type({ size: xs, leading: none })}

  style modeSeg = {size({ w: 100%, min-w: 0 }).pad({ x: 1 }).border({ width: 1, edge: b, color: border })}
  style modeSegRail = {pad(1).bg(muted).border({ color: border, width: 1 }).radius(md)}
  style modeSegBtn = {grow(1).size({ min-w: 0 }).pad({ y: 2, x: 2 }).fg(muted-foreground).bg(transparent).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).cursor(pointer).select(none).tapHighlight(none).type({ size: xs, weight: medium, align: center, wrap: nowrap }), when: {hover: fg(popover-foreground).bg(hover-bg), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset }), selected: fg(popover-foreground).bg(hover-bg).border({ color: primary, edge: b }).type({ weight: semibold })}}
  style modeSegBtnPill = {border({ width: 0 }).radius(sm), when: {selected: bg(background).fg(foreground).border({ width: 0 }).type({ weight: semibold })}}

  style catalogHead = {size({ w: 100%, min-w: 0 }).pad({ x: 3, t: 2, b: 1 }).fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: label-transform })}

  style panelList = {size({ w: 100%, min-w: 0 }).pad(1_5)}
  style option = {size({ w: 100%, min-w: 0 }).pad(0).overflow({ all: hidden }).bg(transparent).border({ color: transparent, width: hairline }).radius(md).cursor(pointer).type({ align: start }), when: {group-chrome-inline: optionInline, group-inline: optionInline, group-orientation-vertical: optionStretch}}
  style optionRow = {size({ w: 100%, min-w: 0 }).radius(sm)}
  style optionCol = {size({ w: 100%, min-w: 0 })}

  style swatch = {position(relative).size({ w: 100%, min-w: 0, h: 20, min-h: 20 }).overflow({ all: hidden }).radius(md).bg(background).fg(foreground).isolate(), when: {group-size-compact: swatchCompact}}
  style swatchRow = {size({ w: 100%, min-w: 0, h: auto, min-h: 0 }).bg(transparent).radius(none)}
  style overlay = {position({ scheme: absolute, inset: 0, z: 1 }).filter({ opacity: bind(surface-overlay-opacity), blend: bind(surface-overlay-blend) }).bg(bind(surface-overlay)).pointerEvents(none)}
  style specimen = {position({ scheme: relative, z: 2 }).size({ w: 100%, h: 100%, min-w: 0 }).pad(2_5)}
  style specimenRow = {pad({ y: 2, x: 2 })}
  style hideOnMenuRow = {display(none)}
  style title = {fg(foreground).type({ family: heading, weight: bind(heading-weight), size: sm, leading: tight, wrap: truncate })}
  style stripPrimary = {size({ h: 3, min-w: 0 }).radius(sm).bg(primary)}
  style stripSecondary = {size({ h: 3, min-w: 0 }).radius(sm).bg(secondary)}
  style stripAccent = {size({ h: 3, min-w: 0 }).radius(sm).bg(accent)}
  style btnGhost = {pad({ y: px, x: 1_5 }).radius(sm).bg(transparent).fg(foreground).border({ color: border, width: hairline }).type({ family: body, weight: medium, size: xs, leading: none })}
  style btnSolid = {pad({ y: px, x: 1_5 }).radius(sm).bg(primary).fg(primary-foreground).type({ family: body, weight: medium, size: xs, leading: none })}
  style checkMark = {fg(primary).type({ size: sm, weight: semibold, leading: none })}
  style hideOnCompact = {display(flex), when: {group-size-compact: gone}}

  style inlineStrip = {display(none).size({ w: 100%, max-w: 100%, min-w: 0 }).overflow({ x: auto, y: auto }), when: {group-inline: flexOn, group-chrome-inline: flexOn, group-orientation-vertical: stack, group-mode: gone, group-mode-only: gone}}

  style focusRing = {when: {focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset })}}
  style triggerHover = {when: {hover: bg(hover-bg)}}
</scope>

<row use:themeSelect data-theme-picker="1" style(host, {match: { variant: { 'menu-row': size({ w: 100% }) }, chrome: { inline: hostInline }, orientation: { horizontal: hostHoriz, vertical: hostVert } }})>
  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    label="Choose theme"
    style(trigger, triggerHover, focusRing)
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(triggerIcon, {shrink(0)})>
      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>

  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom"
    label="Theme"
    data-theme-listbox
    style(panel, {match: { variant: { 'menu-row': panelMenuRow } }})
  >
    <button
      type="button"
      data-picker-trigger="mode"
      data-mode-only-trigger="1"
      label="Color mode"
      style(modeTrigger, triggerHover, focusRing)
    >
      <span data-mode-icon="light" hidden="true" style(glyph, {shrink(0)})>☀</span>
      <span data-mode-icon="dark" hidden="true" style(glyph, {shrink(0)})>☾</span>
      <span data-mode-icon="system" hidden="true" style(glyph, {shrink(0)})>◐</span>
    </button>
    <row data-inline-catalog style(inlineStrip)>
      <button type="button" value="sail" theme="sail" mode="light" data-inline-card style(option, swatch, {when: {hover: border({ color: border }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset }), selected: border({ color: primary }), forced-colors: {when: {selected: border({ color: CanvasText })}}}})>
        <span data-swatch-title style(title)>Sail</span>
        <span data-selected-check hidden="true" style(checkMark)>✓</span>
        <span style(stripPrimary)></span>
        <span style(stripSecondary)></span>
        <span style(stripAccent)></span>
      </button>
    </row>
    <column gap="2" data-picker-header style(header)>
      <div data-picker-title style({size({ min-w: 0 })})>
        <column gap="1">
          <span style(headerEyebrow)>Active</span>
          <row align="center" gap="2" nowrap style({size({ min-w: 0 })})>
            <span data-selected-label style(headerTitle, {grow(1).size({ min-w: 0 })})>—</span>
            <span data-selected-tier style(headerMeta, {shrink(0)})></span>
          </row>
        </column>
      </div>

      <row
        gap="0"
        label="Filter themes by mode"
        data-mode-bar
        data-mode-segment
        nowrap
        style(modeSeg, {size({ min-w: 0 }), match: { variant: { segmented: modeSegRail } }})
      >
        <button type="button" mode="light" selected="true" data-active="true" title="Light" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill } }})>
          Light
        </button>
        <button type="button" mode="dark" selected="false" title="Dark" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill } }})>
          Dark
        </button>
        <button type="button" mode="system" selected="false" title="System" style(modeSegBtn, {match: { variant: { segmented: modeSegBtnPill } }})>
          System
        </button>
      </row>
    </column>

    <span data-catalog-label style(catalogHead)>Light themes</span>

    <column gap="2" data-theme-grid style(panelList, {size({ min-w: 0 })})>
      <button
        type="button"
        value="sail"
        style(option, {when: {hover: border({ color: border }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset }), selected: border({ color: primary }), forced-colors: {when: {selected: border({ color: CanvasText })}}}, match: { variant: { 'menu-row': optionRow } }})
      >
        <column gap="0" style(optionCol, {size({ min-w: 0 })})>
          <column theme="sail" mode="light" hidden="true" style(swatch, {match: { variant: { 'menu-row': swatchRow } }})>
            <span style(overlay, {match: { variant: { 'menu-row': hideOnMenuRow } }})></span>
            <column gap="1" justify="between" style(specimen, {size({ min-w: 0 }), match: { variant: { 'menu-row': specimenRow } }})>
              <row align="center" gap="2" nowrap style({size({ min-w: 0, w: 100% })})>
                <span data-swatch-title style(title, {grow(1).size({ min-w: 0 })})>Sail</span>
                <span data-selected-check hidden="true" style(checkMark, {shrink(0)})>✓</span>
              </row>
              <row align="center" gap="1" nowrap style({size({ min-w: 0, w: 100% }), match: { variant: { 'menu-row': hideOnMenuRow } }})>
                <span style(stripPrimary, {grow(1).size({ min-w: 0 })})></span>
                <span style(stripSecondary, {grow(1).size({ min-w: 0 })})></span>
                <span style(stripAccent, {grow(1).size({ min-w: 0 })})></span>
              </row>
              <row align="center" gap="1" nowrap style(hideOnCompact, {size({ min-w: 0 }), match: { variant: { 'menu-row': hideOnMenuRow } }})>
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
