<!--
  <theme-picker> — the theme switcher: a trigger that opens a panel of theme
  cards, each previewing the surface it selects.

    <theme-picker></theme-picker>

  The whole control is STATIC MARKUP plus one action. The panel is a native
  `[popover]` — top layer, Escape and outside-click close, no positioning code —
  and each card previews its theme by setting `data-theme` on the swatch itself,
  so the colors come from the real stylesheet rather than a hand-copied hex.

  Selection routes through `applyTheme` (see use:themePicker), which sets
  `data-theme`, toggles `.dark`, and announces `ship:theme:change`.
-->

<scope>
  props: {
    value = ''
    inline = false
  }
</scope>

<style>
  .sail-theme-picker { display: inline-flex; }

  /* The trigger is chrome, not content: a compact icon button. The panel does
     the explaining. */
  .sail-theme-trigger {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--control-h-sm);
    height: var(--control-h-sm);
    padding: 0;
    color: var(--foreground);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-full);
    cursor: pointer;
  }
  .sail-theme-trigger:hover { background: var(--hover-bg); }
  .sail-theme-trigger:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }
  .sail-theme-trigger svg { width: 16px; height: 16px; flex: 0 0 auto; }

  .sail-theme-panel {
    margin: 0;
    position: fixed;
    padding: var(--space-1_5);
    width: 380px;
    max-width: calc(100vw - var(--space-4));
    background: var(--popover);
    color: var(--popover-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
  }
  .sail-theme-panel:popover-open {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-1_5);
  }
  /* The `inline` variant's rules live with the rest of this element's skin in
     @ship/tokens element-skins.css (the skin supersedes this style block). */

  .sail-theme-option {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    /* Grid items default to min-width:auto, so a button refuses to shrink below
       its label and blows the track out past the panel. */
    min-width: 0;
    padding: var(--space-1_5);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    font: inherit;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    text-align: start;
  }
  .sail-theme-option:hover { background: var(--hover-bg); }
  .sail-theme-option:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }
  .sail-theme-option[aria-selected='true'] {
    border-color: var(--primary);
    color: var(--foreground);
    font-weight: var(--font-weight-medium);
  }

  /* The preview IS the theme: a miniature UI rendered under
     [data-theme-preview], so the surface, the card, the type scale, the accent
     and the shadows are all painted by the real stylesheet. Three colored bars
     tell you nothing; this tells you what the theme looks like. */
  .sail-theme-swatch {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 96px;
    border-radius: var(--radius-md);
    overflow: hidden;
    isolation: isolate;
    background: var(--background);
    box-shadow: inset 0 0 0 1px color-mix(in oklch, currentColor 14%, transparent);
  }
  .sail-theme-swatch .tc-overlay {
    position: absolute;
    inset: 0;
    z-index: 1;
    pointer-events: none;
    background: var(--surface-overlay);
    mix-blend-mode: var(--surface-overlay-blend);
    opacity: var(--surface-overlay-opacity);
  }
  .sail-theme-swatch .tc-card {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: calc(100% - 18px);
    padding: 7px 9px 8px;
    background: var(--card);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-card-rest, var(--shadow-sm));
  }
  .sail-theme-swatch .tc-eyebrow {
    font-family: var(--label-font);
    font-weight: var(--label-weight);
    letter-spacing: var(--label-tracking);
    font-size: 7px;
    line-height: 1;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .sail-theme-swatch .tc-heading {
    font-family: var(--heading-font);
    font-weight: var(--heading-weight);
    font-size: var(--text-sm);
    line-height: 1.05;
    color: var(--card-foreground);
  }
  .sail-theme-swatch .tc-typo {
    display: flex;
    align-items: baseline;
    gap: 5px;
  }
  .sail-theme-swatch .tc-body {
    font-family: var(--body-font);
    font-size: 8px;
    line-height: 1;
    color: var(--card-foreground);
    opacity: 0.85;
  }
  .sail-theme-swatch .tc-mono {
    font-family: var(--font-mono);
    font-size: 7.5px;
    padding: 1px 3px;
    border-radius: 3px;
    background: var(--hover-bg);
    color: var(--muted-foreground);
  }
  .sail-theme-swatch .tc-controls { display: flex; align-items: center; gap: 4px; }
  .sail-theme-swatch .tc-cta {
    display: inline-flex;
    font-weight: var(--font-weight-medium);
    font-size: 8px;
    line-height: 1;
    padding: 3px 6px;
    border-radius: var(--radius-sm);
    background: var(--primary);
    color: var(--primary-foreground);
  }
  .sail-theme-swatch .tc-ghost {
    display: inline-flex;
    font-weight: var(--font-weight-medium);
    font-size: 8px;
    line-height: 1;
    padding: 3px 6px;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--muted-foreground);
    border: 1px solid var(--border);
  }
  .sail-theme-swatch .tc-chip {
    display: inline-flex;
    font-weight: 600;
    font-size: 7.5px;
    line-height: 1;
    padding: 2px 5px;
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--accent) 18%, transparent);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--accent) 30%, transparent);
  }

  @media (forced-colors: active) {
    .sail-theme-option[aria-selected='true'] { border-color: CanvasText; }
  }
</style>

<div use:themePicker>
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
  </div>
</div>
