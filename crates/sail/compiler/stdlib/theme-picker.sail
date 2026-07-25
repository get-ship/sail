<!--
  <theme-picker> — Style product for trigger/panel/option/swatch paint.
  Paint: component.theme-picker tokens. Swatch mini-cards read card/background
  globals intentionally (theme previews). group-inline matrices via when.
-->
<scope>
  props: {
    value = ''
    inline = false
  }

  const host = style.group()
  const trigger = style.size(control-h-sm).pad(0).fg(theme-picker-trigger-foreground).bg(transparent).borderWidth(1).border(border).radius(full).cursor(pointer)
  const triggerIcon = style.size(icon-size-md).shrink(0).display(block)
  const panel = style.position(fixed).left(bind(anchorLeft)).top(bind(anchorTop)).m(0).pad(1_5).w(panel-max-w-md).maxW(panel-max-w-md).fg(theme-picker-foreground).bg(theme-picker).borderWidth(1).border(border).radius(md).shadow(lg)
  const option = style.minW(0).pad(1_5).bg(transparent).borderWidth(1).border(transparent).radius(md).cursor(pointer).fontFamily(body).fontSize(xs).fg(theme-picker-option-foreground).textAlign(start)
  const optionCol = style.minW(0).w(full)
  const swatch = style.position(relative).w(full).h(24).radius(md).overflow(hidden).isolate().bg(background).shadow(inset-ring)
  const overlay = style.position(absolute).inset(0).z(1).pointerEvents(none).opacity(bind(surfaceOverlayOpacity)).mixBlend(bind(surfaceOverlayBlend)).bg(bind(surfaceOverlay))
  const card = style.position(relative).z(2).w(full).mx(2).padY(2).padX(2_5).bg(card).fg(card-foreground).borderWidth(1).border(border).radius(md).shadow(card-rest)
  const eyebrow = style.fontFamily(bind(labelFont)).fontWeight(bind(labelWeight)).tracking(bind(labelTracking)).fontSize(xs).leading(none).fg(muted-foreground)
  const heading = style.fontFamily(heading).fontWeight(bind(headingWeight)).fontSize(sm).leading(none).fg(card-foreground)
  const bodySample = style.fontFamily(body).fontSize(xs).leading(none).fg(card-foreground).opacity(85)
  const monoSample = style.fontFamily(mono).fontSize(xs).padX(1).padY(px).radius(sm).bg(hover-bg).fg(muted-foreground)
  const cta = style.fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(primary).fg(primary-foreground)
  const ghost = style.fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(transparent).fg(muted-foreground).borderWidth(1).border(border)
  const chip = style.fontWeight(semibold).fontSize(xs).leading(none).padY(px).padX(1).radius(full).bg(muted).fg(accent).shadow(inset-ring)
</scope>
<row inline use:themeSelect sx={host}>
  <button class="sail-theme-trigger" type="button" popovertarget="sail-theme-panel" aria-label="Choose theme" sx={[
    trigger,
    when({
      hover: style.bg(theme-picker-option-hover-bg),
      focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
      group-inline: style.display(none),
    }),
  ]}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" sx={triggerIcon}>
      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>
  <grid cols="2" gap="2" class="sail-theme-panel" id="sail-theme-panel" popover use:anchor placement="bottom-end" role="listbox" aria-label="Theme" sx={[
    panel,
    when({
      group-inline: style.position(static).w(full).maxW(none).shadow(none).bg(transparent).borderWidth(0).pad(0),
    }),
  ]}>
    <button class="sail-theme-option" type="button" role="option" value="ship-light" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="ship-light" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>SHIP Light</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>SHIP Light</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="ship-dark" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="ship-dark" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>SHIP Dark</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>SHIP Dark</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="paper" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="paper" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Paper</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Paper</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="halo" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="halo" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Halo</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Halo</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="console" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="console" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Console</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Console</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="contrast" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="contrast" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>High Contrast</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>High Contrast</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="glass" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="glass" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Glass</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Glass</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-light" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="neutral-light" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Neutral Light</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Neutral Light</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-dark" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="neutral-dark" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Neutral Dark</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Neutral Dark</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="brut" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="brut" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Brutalist</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Brutalist</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="8bit" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="8bit" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>8-Bit</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>8-Bit</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="synthwave" sx={[
      option,
      when({
        hover: style.bg(theme-picker-option-hover-bg),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
        selected: style.border(primary).fg(theme-picker-foreground).fontWeight(medium),
        forced-colors: when({
          selected: style.border(CanvasText),
        }),
      }),
    ]}>
      <column gap="1" sx={optionCol}>
        <center class="sail-theme-swatch" data-theme-preview="synthwave" aria-hidden="true" sx={swatch}>
          <span class="tc-overlay" sx={overlay}></span>
          <column class="tc-card" gap="1" sx={card}>
            <span class="tc-eyebrow" sx={eyebrow}>Theme</span>
            <span class="tc-heading" sx={heading}>Synthwave</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx={bodySample}>The quick brown fox</span>
              <span class="tc-mono" sx={monoSample}>build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx={cta}>Action</row>
              <row inline nowrap class="tc-ghost" sx={ghost}>Cancel</row>
              <row inline nowrap class="tc-chip" sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Synthwave</span>
      </column>
    </button>
  </grid>
</row>
