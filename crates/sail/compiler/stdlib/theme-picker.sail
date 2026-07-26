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
  const triggerIcon = style.size(icon-size-md).display(block)
  // Popover HOST must not set `display` (layout keywords do). UA closed state is
  // display:none; author display:grid forces the panel always-visible and can
  // crash heavy pages (header picker + /components tile = two full panels).
  const panel = style.position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(1_5).w(panel-max-w-md).maxW(panel-max-w-md).fg(theme-picker-foreground).bg(theme-picker).borderWidth(1).border(border).radius(md).shadow(lg).z(popover)
  const panelGrid = style.w(full)
  const option = style.minW(0).pad(1_5).bg(transparent).borderWidth(1).border(transparent).radius(md).cursor(pointer).fontFamily(body).fontSize(xs).fg(theme-picker-option-foreground).textAlign(start)
  const optionCol = style.minW(0).w(full)
  // Swatch mini-cards: paint MUST use theme CSS vars (background/card/…) so
  // [data-theme-preview='…'] scopes from themes.css re-skin the preview.
  // h(24)=space-24 (96px) is the intentional swatch height.
  // opacity only accepts token steps (…80,90…) — not bare 85.
  const swatch = style.position(relative).w(full).h(24).minH(24).radius(md).overflow(hidden).isolate().bg(background).shadow(inset-ring)
  const overlay = style.position(absolute).inset(0).z(1).pointerEvents(none).opacity(bind(surface-overlay-opacity)).mixBlend(bind(surface-overlay-blend)).bg(bind(surface-overlay))
  const card = style.position(relative).z(2).w(full).mx(2).padY(2).padX(2_5).bg(card).fg(card-foreground).borderWidth(1).border(border).radius(md).shadow(card-rest)
  const eyebrow = style.fontFamily(bind(label-font)).fontWeight(bind(label-weight)).tracking(bind(label-tracking)).fontSize(xs).leading(none).fg(muted-foreground)
  const heading = style.fontFamily(heading).fontWeight(bind(heading-weight)).fontSize(sm).leading(none).fg(card-foreground)
  const bodySample = style.fontFamily(body).fontSize(xs).leading(none).fg(card-foreground).opacity(80)
  const monoSample = style.fontFamily(mono).fontSize(xs).padX(1).padY(px).radius(sm).bg(hover-bg).fg(muted-foreground)
  const cta = style.fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(primary).fg(primary-foreground)
  const ghost = style.fontWeight(medium).fontSize(xs).leading(none).padY(px).padX(1_5).radius(sm).bg(transparent).fg(muted-foreground).borderWidth(1).border(border)
  const chip = style.fontWeight(semibold).fontSize(xs).leading(none).padY(px).padX(1).radius(full).bg(muted).fg(accent).shadow(inset-ring)
</scope>
<row inline use:themeSelect sx={host}>
  <button type="button" popovertarget="sail-theme-panel" aria-label="Choose theme" sx={[
    trigger,
    when({
      hover: style.bg(theme-picker-option-hover-bg),
      focus: style.outline(solid).outline(2).outline(ring).outlineOffset(ring-offset),
      group-inline: style.display(none),
    }),
  ]}>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" shrink="0" sx={triggerIcon}>
      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>
  <div id="sail-theme-panel" popover use:anchor placement="bottom-end" role="listbox" aria-label="Theme" sx={[
    panel,
    when({
      group-inline: style.position(static).w(full).maxW(none).shadow(none).bg(transparent).borderWidth(0).pad(0),
    }),
  ]}>
    <grid cols="2" gap="2" sx={panelGrid}>
    <button type="button" role="option" value="ship-light" sx={[
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
        <center data-theme-preview="ship-light" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>SHIP Light</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>SHIP Light</span>
      </column>
    </button>
    <button type="button" role="option" value="ship-dark" sx={[
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
        <center data-theme-preview="ship-dark" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>SHIP Dark</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>SHIP Dark</span>
      </column>
    </button>
    <button type="button" role="option" value="paper" sx={[
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
        <center data-theme-preview="paper" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Paper</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Paper</span>
      </column>
    </button>
    <button type="button" role="option" value="halo" sx={[
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
        <center data-theme-preview="halo" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Halo</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Halo</span>
      </column>
    </button>
    <button type="button" role="option" value="console" sx={[
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
        <center data-theme-preview="console" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Console</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Console</span>
      </column>
    </button>
    <button type="button" role="option" value="contrast" sx={[
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
        <center data-theme-preview="contrast" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>High Contrast</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>High Contrast</span>
      </column>
    </button>
    <button type="button" role="option" value="glass" sx={[
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
        <center data-theme-preview="glass" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Glass</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Glass</span>
      </column>
    </button>
    <button type="button" role="option" value="neutral-light" sx={[
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
        <center data-theme-preview="neutral-light" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Neutral Light</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Neutral Light</span>
      </column>
    </button>
    <button type="button" role="option" value="neutral-dark" sx={[
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
        <center data-theme-preview="neutral-dark" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Neutral Dark</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Neutral Dark</span>
      </column>
    </button>
    <button type="button" role="option" value="brut" sx={[
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
        <center data-theme-preview="brut" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Brutalist</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Brutalist</span>
      </column>
    </button>
    <button type="button" role="option" value="8bit" sx={[
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
        <center data-theme-preview="8bit" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>8-Bit</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>8-Bit</span>
      </column>
    </button>
    <button type="button" role="option" value="synthwave" sx={[
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
        <center data-theme-preview="synthwave" aria-hidden="true" sx={swatch}>
          <span sx={overlay}></span>
          <column gap="1" sx={card}>
            <span sx={eyebrow}>Theme</span>
            <span sx={heading}>Synthwave</span>
            <row align="baseline" gap="1">
              <span sx={bodySample}>The quick brown fox</span>
              <span sx={monoSample}>build: ok</span>
            </row>
            <row align="center" gap="1">
              <row inline nowrap sx={cta}>Action</row>
              <row inline nowrap sx={ghost}>Cancel</row>
              <row inline nowrap sx={chip}>New</row>
            </row>
          </column>
        </center>
        <span>Synthwave</span>
      </column>
    </button>
    </grid>
  </div>
</row>
