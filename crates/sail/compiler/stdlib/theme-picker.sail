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

  style host = {paint: group()}

  style trigger = {box: size(control-h-sm).space({ pad: 0 }), paint: ink({ color: theme-picker.trigger-fg }).fill({ color: transparent }).border({ color: border, width: hairline }).radius(full).cursor(pointer)}
  style modeTrigger = {box: size({ min-w: control-h-sm }).space({ pad: 0 }), paint: ink({ color: theme-picker.trigger-fg }).fill({ color: transparent }).border({ color: border, width: hairline }).radius(full).cursor(pointer)}
  style triggerIcon = {box: size(icon-size-md).flow({ display: block })}
  style glyph = {paint: ink({ color: theme-picker.trigger-fg }), text: type({ size: sm, leading: none })}

  style panel = {box: place({ position: fixed, inset: auto, left: bind(anchorLeft), top: bind(anchorTop), z: popover }).space({ m: 0, pad: 0 }).size({ w: panel-max-w-sm, max-w: panel-w-toast, min-w: 0, max-h: panel-vh-md }).overflow({ x: hidden, y: auto, overscroll: contain }), paint: ink({ color: theme-picker.foreground }).fill({ color: theme-picker }).border({ color: border, width: hairline }).radius(lg).elevation(lg)}
  style panelInline = {box: place({ position: static, inset: auto }).space({ m: 0, pad: 0 }).size({ w: full, max-w: none, min-w: 0 }).overflow({ x: hidden }), paint: elevation(none).fill({ color: transparent }).border({ width: 0 })}

  style header = {box: size({ w: full, min-w: 0 }).space({ pad-x: 3, pad-t: 3, pad-b: 0 })}
  style headerEyebrow = {paint: ink({ color: theme-picker.option-fg }), text: type({ size: xs, weight: semibold, tracking: wide, case: label-transform, leading: none })}
  style headerTitle = {paint: ink({ color: theme-picker.foreground }), text: type({ family: heading, weight: bind(heading-weight), size: base, leading: tight, wrap: truncate })}
  style headerMeta = {paint: ink({ color: theme-picker.option-fg }), text: type({ size: xs, leading: none })}

  // Filter tabs — always readable on popover chrome. Selected uses hover-bg +
  // primary underline (never primary fill — breaks on brut).
  style modeSeg = {box: size({ w: full, min-w: 0 }).space({ pad-x: 1 }), paint: border({ width: 1, edge: b, color: border })}
  style modeSegBtn = {box: flow({ grow: 1 }).size({ min-w: 0 }).space({ pad-y: 2, pad-x: 2 }), paint: ink({ color: theme-picker.option-fg }).fill({ color: transparent }).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).cursor(pointer).select(none).tapHighlight(none), text: type({ size: xs, weight: medium, align: center, wrap: nowrap }), when: {hover: {paint: ink({ color: theme-picker.foreground }).fill({ color: hover-bg })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset })}, selected: {paint: ink({ color: theme-picker.foreground }).fill({ color: hover-bg }).border({ color: primary, edge: b }), text: type({ weight: semibold })}}}

  style catalogHead = {box: size({ w: full, min-w: 0 }).space({ pad-x: 3, pad-t: 2, pad-b: 1 }), paint: ink({ color: theme-picker.option-fg }), text: type({ size: xs, weight: semibold, tracking: wide, case: label-transform })}

  style panelList = {box: size({ w: full, min-w: 0 }).space({ pad: 1_5 })}
  style option = {box: size({ w: full, min-w: 0 }).space({ pad: 0 }).overflow({ all: hidden }), paint: fill({ color: transparent }).border({ color: transparent, width: hairline }).radius(md).cursor(pointer), text: type({ align: start })}
  style optionCol = {box: size({ w: full, min-w: 0 })}

  // Live specimen — full-bleed via theme= (theme fonts + tokens).
  style swatch = {box: place({ position: relative }).size({ w: full, min-w: 0, h: 20, min-h: 20 }).overflow({ all: hidden }), paint: radius(md).fill({ color: background }).ink(foreground).isolate()}
  style overlay = {box: place({ position: absolute, inset: 0, z: 1 }), paint: filter({ opacity: bind(surface-overlay-opacity), blend: bind(surface-overlay-blend) }).fill({ color: bind(surface-overlay) }).pointerEvents(none)}
  style specimen = {box: place({ position: relative, z: 2 }).size({ w: full, h: full, min-w: 0 }).space({ pad: 2_5 })}
  style title = {paint: ink({ color: foreground }), text: type({ family: heading, weight: bind(heading-weight), size: sm, leading: tight, wrap: truncate })}
  // Chromatic trio: primary · secondary · accent
  style stripPrimary = {box: size({ h: 3, min-w: 0 }), paint: radius(sm).fill({ color: primary })}
  style stripSecondary = {box: size({ h: 3, min-w: 0 }), paint: radius(sm).fill({ color: secondary })}
  style stripAccent = {box: size({ h: 3, min-w: 0 }), paint: radius(sm).fill({ color: accent })}
  style btnGhost = {box: space({ pad-y: px, pad-x: 1_5 }), paint: radius(sm).fill({ color: transparent }).ink(foreground).border({ color: border, width: hairline }), text: type({ family: body, weight: medium, size: xs, leading: none })}
  style btnSolid = {box: space({ pad-y: px, pad-x: 1_5 }), paint: radius(sm).fill({ color: primary }).ink({ color: primary-foreground }), text: type({ family: body, weight: medium, size: xs, leading: none })}
  style checkMark = {paint: ink({ color: primary }), text: type({ size: sm, weight: semibold, leading: none })}

  style focusRing = {when: {focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset })}}}
  style triggerHover = {when: {hover: {paint: fill({ color: hover-bg })}}}
</scope>

<row inline use:themeSelect data-theme-picker="1" style(host)>
  <button
    type="button"
    popovertarget="sail-theme-panel"
    data-picker-trigger="full"
    label="Choose theme"
    style(trigger, triggerHover, focusRing, {when: {group-inline: {box: flow({ display: none })}}})
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(triggerIcon, {box: flow({ shrink: 0 })})>
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
    <span data-mode-icon="light" hidden="true" style(glyph, {box: flow({ shrink: 0 })})>☀</span>
    <span data-mode-icon="dark" hidden="true" style(glyph, {box: flow({ shrink: 0 })})>☾</span>
    <span data-mode-icon="system" hidden="true" style(glyph, {box: flow({ shrink: 0 })})>◐</span>
  </button>

  <!-- listbox role stamped by use:themeSelect (closed multi-part anatomy) -->
  <div
    id="sail-theme-panel"
    popover
    use:anchor
    placement="bottom-end"
    label="Theme"
    data-theme-listbox
    style(panel, {match: {
        inline: {
          true: panelInline,
        },
      }})
  >
    <column gap="2" data-picker-header style(header)>
      <column gap="1" style({box: size({ min-w: 0 })})>
        <span style(headerEyebrow)>Active</span>
        <row align="center" gap="2" nowrap style({box: size({ min-w: 0 })})>
          <span data-selected-label style(headerTitle, {box: flow({ grow: 1 }).size({ min-w: 0 })})>—</span>
          <span data-selected-tier style(headerMeta, {box: flow({ shrink: 0 })})></span>
        </row>
      </column>

      <!-- tablist/tab/option roles stamped by use:themeSelect -->
      <row
        gap="0"
        label="Filter themes by mode"
        data-mode-bar
        data-mode-segment
        nowrap
        style(modeSeg, {box: size({ min-w: 0 })})
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

    <column gap="2" data-theme-grid style(panelList, {box: size({ min-w: 0 })})>
      <button
        type="button"
        value="sail"
        style(option, {when: {hover: {paint: border({ color: border })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: ring-offset })}, selected: {paint: border({ color: primary })}, forced-colors: {when: {selected: {paint: border({ color: CanvasText })}}}}})
      >
        <column gap="0" style(optionCol, {box: size({ min-w: 0 })})>
          <column theme="sail" data-mode="light" hidden="true" style(swatch)>
            <span style(overlay)></span>
            <column gap="2" justify="between" style(specimen, {box: size({ min-w: 0 })})>
              <row align="center" gap="2" nowrap style({box: size({ min-w: 0, w: full })})>
                <span data-swatch-title style(title, {box: flow({ grow: 1 }).size({ min-w: 0 })})>Sail</span>
                <span data-selected-check hidden="true" style(checkMark, {box: flow({ shrink: 0 })})>✓</span>
              </row>
              <row align="center" gap="1" nowrap hidden="true" style({box: size({ min-w: 0, w: full })})>
                <span style(stripPrimary, {box: flow({ grow: 1 }).size({ min-w: 0 })})></span>
                <span style(stripSecondary, {box: flow({ grow: 1 }).size({ min-w: 0 })})></span>
                <span style(stripAccent, {box: flow({ grow: 1 }).size({ min-w: 0 })})></span>
              </row>
              <row align="center" gap="1" nowrap style({box: size({ min-w: 0 })})>
                <row inline nowrap style(btnGhost, {box: flow({ shrink: 0 })})>Cancel</row>
                <row inline nowrap style(btnSolid, {box: flow({ shrink: 0 })})>Continue</row>
              </row>
            </column>
          </column>
        </column>
      </button>
    </column>
  </div>
</row>
