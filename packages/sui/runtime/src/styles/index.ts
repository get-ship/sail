const TONES_CSS =
  // The FULL tone contract, framework-owned. A component that declares a
  // `tone` prop gets all six channels for free — no per-component <theme
  // prop="tone"> block. (Extends the prior 3-channel set to bg/border/ring.)
  ":host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background);              --tone-bg: var(--hover-bg);            --tone-border: var(--border);                 --tone-ring: var(--ring); }\n" +
  ":host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground);     --tone-bg: var(--tone-primary-bg);     --tone-border: var(--tone-primary-border);     --tone-ring: var(--tone-primary-ring); }\n" +
  ":host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground);        --tone-bg: var(--tone-info-bg);        --tone-border: var(--tone-info-border);        --tone-ring: var(--tone-info-ring); }\n" +
  ":host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--tone-success-fg);  --tone-contrast: var(--success-foreground);     --tone-bg: var(--tone-success-bg);     --tone-border: var(--tone-success-border);     --tone-ring: var(--tone-success-ring); }\n" +
  ":host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--tone-warning-fg);  --tone-contrast: var(--warning-foreground);     --tone-bg: var(--tone-warning-bg);     --tone-border: var(--tone-warning-border);     --tone-ring: var(--tone-warning-ring); }\n" +
  ":host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); --tone-bg: var(--tone-destructive-bg); --tone-border: var(--tone-destructive-border); --tone-ring: var(--tone-destructive-ring); }\n" +
  ":host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground);      --tone-bg: var(--tone-accent-bg);      --tone-border: var(--tone-accent-border);      --tone-ring: var(--tone-accent-ring); }\n" +
  ":host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background);              --tone-bg: var(--tone-muted-bg);       --tone-border: var(--tone-muted-border);       --tone-ring: var(--tone-muted-ring); }\n"

const SIZES_CSS =
  // The FULL size contract, framework-owned — the exact analogue of tone. A
  // component that declares a `size` prop gets the dimensional channels for free
  // (`--size-h` control height, `--size-px` inline padding, `--size-fs` font
  // size, `--size-gap` inner gap). No per-component <theme prop="size"> block:
  // one global scale keeps every control coherent, and we own the compiler.
  ":host([size='xs']) { --size-h: var(--control-h-2xs); --size-px: var(--space-2);   --size-fs: var(--text-xs);   --size-gap: var(--space-1); }\n" +
  ":host([size='sm']) { --size-h: var(--control-h-sm);  --size-px: var(--space-2_5); --size-fs: var(--text-sm);   --size-gap: var(--space-1_5); }\n" +
  ":host([size='md']) { --size-h: var(--control-h-md);  --size-px: var(--space-3);   --size-fs: var(--text-base); --size-gap: var(--space-2); }\n" +
  ":host([size='lg']) { --size-h: var(--control-h-lg);  --size-px: var(--space-4);   --size-fs: var(--text-md);   --size-gap: var(--space-2); }\n"

// R3: shared host-resets adopted into every shadow component. Rules
// here are duplicated literally in nearly every <style> block; hoisting
// them lets the sui-compiler strip the per-component copies (see
// `crates/sui-compiler/src/emit/styles.rs`). Keep this list strictly to
// rules whose author-form text the compiler can recognize and remove.
const BASE_CSS =
  // Framework-owned host defaults. Custom elements default to `display:inline`
  // (almost never wanted) and every component wants the foreground color — so
  // the framework decides both. A component needing a different display (inline
  // -flex, grid) overrides in its own styles, which win in the cascade. This is
  // what lets most <host> blocks disappear entirely.
  ':host { display: block; color: var(--foreground); }\n' +
  ':host([hidden]) { display: none !important; }\n' +
  '@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }\n'

// LANGUAGE-LAYER TYPOGRAPHY. Semantic content tags are styled from the type
// scale automatically — the author writes `<h1>`/`<p>` plain and gets taste,
// with no sx and no <style>. This is what makes the `strictest` policy livable:
// text has a home (the tags themselves) instead of needing a styling escape.
// Every value is a token; nothing here is hand-tuned. Adopted into every shadow
// component alongside the resets.
const TYPOGRAPHY_CSS =
  'h1,h2,h3,h4,h5,h6 { margin: 0; font-family: var(--heading-font, inherit); font-weight: var(--heading-weight, 700); line-height: var(--leading-tight, 1.15); color: var(--foreground); }\n' +
  'h1 { font-size: var(--text-4xl); }\n' +
  'h2 { font-size: var(--text-2xl); }\n' +
  'h3 { font-size: var(--text-xl); }\n' +
  'h4 { font-size: var(--text-lg); }\n' +
  'h5 { font-size: var(--text-md); }\n' +
  'h6 { font-size: var(--text-sm); }\n' +
  'p  { margin: 0; font-size: var(--text-md); line-height: 1.55; color: var(--foreground); }\n' +
  'small { font-size: var(--text-sm); color: var(--muted-foreground); }\n' +
  'a { color: var(--primary); text-decoration: none; }\n' +
  'a:hover { text-decoration: underline; }\n' +
  'strong { font-weight: var(--heading-weight, 700); }\n' +
  'code { font-family: var(--mono-font, monospace); font-size: 0.9em; }\n'

function makeSheet(css: string): CSSStyleSheet | null {
  if (typeof CSSStyleSheet === 'undefined') return null
  try {
    const sheet = new CSSStyleSheet()
    sheet.replaceSync(css)
    return sheet
  } catch {
    return null
  }
}

export const suiTonesSheet: CSSStyleSheet | null = makeSheet(TONES_CSS)
export const suiSizesSheet: CSSStyleSheet | null = makeSheet(SIZES_CSS)
export const suiBaseSheet: CSSStyleSheet | null = makeSheet(BASE_CSS)
export const suiTypographySheet: CSSStyleSheet | null = makeSheet(TYPOGRAPHY_CSS)

/**
 * Adopt every shared sheet into a shadow root (no-op for light DOM /
 * environments without `CSSStyleSheet`).
 *
 * Called once per shadow component instance from compiler-emitted code
 * (see `crates/sui-compiler/src/emit/js.rs`). Centralising the call
 * here keeps the per-component adoption snippet to a single function
 * call — a meaningful saving across N×76 components in the bundle.
 *
 * The compiler relies on this importing zero other module-level
 * symbols at the call site. Keep the API surface tiny so the emitter
 * doesn't need to learn new imports per sheet.
 */
export function adoptSuiSheets(root: unknown): void {
  if (typeof ShadowRoot === 'undefined') return
  if (!(root instanceof ShadowRoot)) return
  const sheets = [suiBaseSheet, suiTonesSheet, suiSizesSheet, suiTypographySheet].filter(
    (s): s is CSSStyleSheet => s != null,
  )
  if (sheets.length === 0) return
  try {
    root.adoptedStyleSheets = [...sheets, ...(root.adoptedStyleSheets ?? [])]
  } catch {
    /* unsupported: silently fall back to inline <style> blocks */
  }
}

// ── LIGHT-DOM contracts ─────────────────────────────────────────────────────
// The document-level, attribute-keyed form of the framework contracts. Where
// the shadow path adopts :host-keyed sheets into every root, the light path
// installs ONE sheet keyed on plain attributes — which is also what makes the
// contracts layer-invisible: `[tone='primary']` styles an L1 <button> and a
// light L2 <sui-chip> identically, and dynamic `tone={x()}` restyles both.
const hostAttrs = (css: string): string =>
  // `:host([tone='x'])` → `[tone='x']` — the element IS the host in light DOM.
  css.replace(/:host\(\[([^)]+)\]\)/g, '[$1]')

// The INTERACTIVE-SURFACE contracts — the framework decisions no component
// re-makes (the audit's control-baseline + focus + disabled clusters:
// ~18% of all baseline CSS was these, hand-repeated). Scoped inside
// [data-component] so host pages are untouched; :where() keeps specificity
// at zero so any component rule wins ties.
const CONTROL_CONTRACTS_CSS =
  // Control baseline: what a control IS before any skin.
  '[data-component] :where(button, input, textarea, select) { font: inherit; font-family: var(--font-body); letter-spacing: inherit; color: inherit; }\n' +
  '[data-component] :where(button) { line-height: 1; user-select: none; white-space: nowrap; cursor: pointer; }\n' +
  '[data-component] :where(input, textarea, select) { appearance: none; }\n' +
  // Focus: one ring, everywhere, token-driven — never re-authored.
  '[data-component] :where(button, input, textarea, select, a, [tabindex]):focus-visible { outline: var(--ring-width, 2px) solid var(--ring); outline-offset: var(--ring-offset, 2px); }\n' +
  '@media (forced-colors: active) { [data-component] :where(button, input, textarea, select, a, [tabindex]):focus-visible { outline: 2px solid CanvasText; } }\n' +
  // Disabled: the treatment, once.
  '[data-component] :where(button, input, textarea, select):disabled { cursor: not-allowed; opacity: 0.5; }\n' +
  // Interactive transition defaults from the motion tokens.
  '[data-component] :where(button, a, [tabindex]) { transition: background-color var(--duration-fast) var(--ease-out), border-color var(--duration-fast) var(--ease-out), box-shadow var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out); }\n'

const LIGHT_CONTRACTS_CSS =
  hostAttrs(TONES_CSS) +
  hostAttrs(SIZES_CSS) +
  CONTROL_CONTRACTS_CSS +
  // Host defaults for light components (the :host base rules, element-keyed).
  '[data-component] { display: block; color: var(--foreground); }\n' +
  '[data-component][hidden] { display: none !important; }\n' +
  // <slot> stays in the light tree as the projection wrapper — transparent.
  '[data-component] slot { display: contents; }\n' +
  '@media (prefers-reduced-motion: reduce) { [data-component], [data-component] * { transition: none !important; animation: none !important; } }\n'

let lightContractsInstalled = false

/**
 * Install the framework contracts once at document level for light-DOM
 * components. Injected EARLY (head, before per-component scoped styles) so
 * a component's own rules win ties by source order.
 */
export function ensureLightContracts(): void {
  if (lightContractsInstalled) return
  if (typeof document === 'undefined') return
  lightContractsInstalled = true
  const id = 'sui-light-contracts'
  if (document.getElementById(id)) return
  const style = document.createElement('style')
  style.id = id
  style.textContent = LIGHT_CONTRACTS_CSS
  document.head.prepend(style)
}
