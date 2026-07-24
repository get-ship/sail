<!--
  <theme-picker> — layout keywords + structure sx for swatch mini-cards → skins.
  CVA/group-inline: hide trigger + static cols=4 panel; panel geometry on sx; forced-colors on option.

  Paint: panel/trigger/option from component.theme-picker tokens (cssNames
  theme-picker, theme-picker-foreground, theme-picker-trigger-foreground,
  theme-picker-option-foreground, theme-picker-option-hover-bg). Swatch mini-cards
  still read card/background globals intentionally (theme previews).
-->
<scope>
  props: {
    value = ''
    inline = false
  }
</scope>
<row inline use:themeSelect sx="group">
  <button class="sail-theme-trigger" type="button" popovertarget="sail-theme-panel" aria-label="Choose theme" sx="size: control-h-sm p: 0 color: theme-picker-trigger-foreground bg: transparent border border: border rounded: full cursor: pointer hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset group-inline:hidden">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" sx="size: icon-size-md shrink: 0 block">
      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>
  <grid cols="2" gap="2" class="sail-theme-panel" id="sail-theme-panel" popover use:anchor placement="bottom-end" role="listbox" aria-label="Theme" sx="fixed left: (--anchor-left) top: (--anchor-top) m: 0 p: 1_5 w: panel-max-w-md max-w: panel-max-w-md color: theme-picker-foreground bg: theme-picker border border: border rounded: md shadow: lg group-inline:static group-inline:w: full group-inline:max-w: none group-inline:shadow: none group-inline:bg: transparent group-inline:border: 0 group-inline:p: 0">
    <button class="sail-theme-option" type="button" role="option" value="ship-light" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="ship-light" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">SHIP Light</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>SHIP Light</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="ship-dark" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="ship-dark" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">SHIP Dark</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>SHIP Dark</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="paper" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="paper" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Paper</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Paper</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="halo" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="halo" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Halo</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Halo</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="console" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="console" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Console</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Console</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="contrast" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="contrast" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">High Contrast</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>High Contrast</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="glass" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="glass" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Glass</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Glass</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-light" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="neutral-light" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Neutral Light</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Neutral Light</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-dark" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="neutral-dark" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Neutral Dark</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Neutral Dark</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="brut" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="brut" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Brutalist</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Brutalist</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="8bit" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="8bit" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">8-Bit</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>8-Bit</span>
      </column>
    </button>
    <button class="sail-theme-option" type="button" role="option" value="synthwave" sx="min-w: 0 p: 1_5 bg: transparent border border: transparent rounded: md cursor: pointer font-family: body font-size: xs color: theme-picker-option-foreground text: start hover:bg: theme-picker-option-hover-bg focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: ring-offset selected:border: primary selected:color: theme-picker-foreground selected:font-weight: medium forced-colors:selected:border: CanvasText">
      <column gap="1" sx="min-w: 0 w: full">
        <center class="sail-theme-swatch" data-theme-preview="synthwave" aria-hidden="true" sx="relative w: full h: 24 rounded: md overflow: hidden isolate bg: background shadow: inset-ring">
          <span class="tc-overlay" sx="absolute inset: 0 z: 1 pointer-events: none opacity: (--surface-overlay-opacity) mix-blend: (--surface-overlay-blend) bg: (--surface-overlay)"></span>
          <column class="tc-card" gap="1" sx="relative z: 2 w: full mx: 2 py: 2 px: 2_5 bg: card color: card-foreground border border: border rounded: md shadow: card-rest">
            <span class="tc-eyebrow" sx="font: (--label-font) font: (--label-weight) tracking: (--label-tracking) color: (--icon-size-3xs) leading: none color: muted-foreground">Theme</span>
            <span class="tc-heading" sx="font-family: heading font: (--heading-weight) font-size: sm leading: none color: card-foreground">Synthwave</span>
            <row class="tc-typo" align="baseline" gap="1">
              <span class="tc-body" sx="font-family: body color: (--icon-size-3xs) leading: none color: card-foreground opacity: 85">The quick brown fox</span>
              <span class="tc-mono" sx="font-family: mono color: (--icon-size-3xs) px: 1 py: px rounded: sm bg: hover-bg color: muted-foreground">build: ok</span>
            </row>
            <row class="tc-controls" align="center" gap="1">
              <row inline nowrap class="tc-cta" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: primary color: primary-foreground">Action</row>
              <row inline nowrap class="tc-ghost" sx="font-weight: medium color: (--icon-size-3xs) leading: none py: px px: 1_5 rounded: sm bg: transparent color: muted-foreground border border: border">Cancel</row>
              <row inline nowrap class="tc-chip" sx="font-weight: semibold color: (--icon-size-3xs) leading: none py: px px: 1 rounded: full bg: muted color: accent shadow: inset-ring">New</row>
            </row>
          </column>
        </center>
        <span>Synthwave</span>
      </column>
    </button>
  </grid>
</row>
