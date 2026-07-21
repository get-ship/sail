<!--
  <theme-picker> — the theme switcher: a trigger that opens a panel of theme
  cards, each previewing the surface it selects.

    <theme-picker></theme-picker>

  The whole control is STATIC MARKUP plus one action. The panel is a native
  `[popover]` — top layer, Escape and outside-click close, no positioning code —
  and each card previews its theme by setting `data-theme` on the swatch itself,
  so the colors come from the real stylesheet rather than a hand-copied hex.

  Selection routes through `applyTheme` (see use:themeSelect), which sets
  `data-theme`, toggles `.dark`, and announces `ship:theme:change`.
-->

<scope>
  props: {
    value = ''
    inline = false
  }
</scope>

<div use:themeSelect sx="inline-flex">
  <button class="sail-theme-trigger" type="button" popovertarget="sail-theme-panel" aria-label="Choose theme">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <circle cx="13.5" cy="6.5" r="1.5"></circle>
      <circle cx="17.5" cy="10.5" r="1.5"></circle>
      <circle cx="8.5" cy="7.5" r="1.5"></circle>
      <circle cx="6.5" cy="12.5" r="1.5"></circle>
      <path d="M12 2a10 10 0 0 0 0 20 2 2 0 0 0 2-2v-1a2 2 0 0 1 2-2h1a4 4 0 0 0 4-4 10 10 0 0 0-9-11Z"></path>
    </svg>
  </button>

  <div class="sail-theme-panel" id="sail-theme-panel" popover use:anchor placement="bottom-end" role="listbox" aria-label="Theme">
    <button class="sail-theme-option" type="button" role="option" value="ship-light">
      <span class="sail-theme-swatch" data-theme-preview="ship-light" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">SHIP Light</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      SHIP Light
    </button>
    <button class="sail-theme-option" type="button" role="option" value="ship-dark">
      <span class="sail-theme-swatch" data-theme-preview="ship-dark" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">SHIP Dark</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      SHIP Dark
    </button>
    <button class="sail-theme-option" type="button" role="option" value="paper">
      <span class="sail-theme-swatch" data-theme-preview="paper" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Paper</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Paper
    </button>
    <button class="sail-theme-option" type="button" role="option" value="halo">
      <span class="sail-theme-swatch" data-theme-preview="halo" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Halo</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Halo
    </button>
    <button class="sail-theme-option" type="button" role="option" value="console">
      <span class="sail-theme-swatch" data-theme-preview="console" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Console</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Console
    </button>
    <button class="sail-theme-option" type="button" role="option" value="contrast">
      <span class="sail-theme-swatch" data-theme-preview="contrast" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">High Contrast</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      High Contrast
    </button>
    <button class="sail-theme-option" type="button" role="option" value="glass">
      <span class="sail-theme-swatch" data-theme-preview="glass" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Glass</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Glass
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-light">
      <span class="sail-theme-swatch" data-theme-preview="neutral-light" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Neutral Light</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Neutral Light
    </button>
    <button class="sail-theme-option" type="button" role="option" value="neutral-dark">
      <span class="sail-theme-swatch" data-theme-preview="neutral-dark" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Neutral Dark</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Neutral Dark
    </button>
    <button class="sail-theme-option" type="button" role="option" value="brut">
      <span class="sail-theme-swatch" data-theme-preview="brut" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Brutalist</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Brutalist
    </button>
    <button class="sail-theme-option" type="button" role="option" value="8bit">
      <span class="sail-theme-swatch" data-theme-preview="8bit" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">8-Bit</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      8-Bit
    </button>
    <button class="sail-theme-option" type="button" role="option" value="synthwave">
      <span class="sail-theme-swatch" data-theme-preview="synthwave" aria-hidden="true"><span class="tc-overlay"></span><span class="tc-card"><span class="tc-eyebrow">Theme</span><span class="tc-heading">Synthwave</span><span class="tc-typo"><span class="tc-body">The quick brown fox</span><span class="tc-mono">build: ok</span></span><span class="tc-controls"><span class="tc-cta">Action</span><span class="tc-ghost">Cancel</span><span class="tc-chip">New</span></span></span></span>
      Synthwave
    </button>
  </div>
</div>
