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

/// Wrap framework CSS in the `sui` cascade layer. Author CSS is UNLAYERED,
/// which beats any layer — so users override the design system by writing
/// normal CSS (or a theme), never fighting specificity or reaching for
/// !important. One rule governs all precedence: your styles are above ours.
/// (`:where()` is kept inside for the framework's OWN internal cascade; it's
/// invisible to the author-override story, which is pure layer order.)
export const layered = (css: string): string => `@layer sui {\n${css}}\n`

export const suiTonesSheet: CSSStyleSheet | null = makeSheet(layered(TONES_CSS))
export const suiSizesSheet: CSSStyleSheet | null = makeSheet(layered(SIZES_CSS))
export const suiBaseSheet: CSSStyleSheet | null = makeSheet(layered(BASE_CSS))
export const suiTypographySheet: CSSStyleSheet | null = makeSheet(layered(TYPOGRAPHY_CSS))

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
// The CONTROL skins (button/input/checkbox/radio surfaces + their baseline)
// live in the light-DOM document contracts, which do NOT cross a shadow
// boundary. A shadow component that nests a framework-styled control (a
// `<button ghost>` dismiss, an `<input>`) would render it visually bare
// unless its shadow root ALSO carries the control skins. So build a sheet of
// {element-keyed tone/size channels + control contracts + button/input/toggle
// skins} and adopt it alongside the base sheets. Element-keyed (not :host-
// keyed) so a nested `<button tone=primary>` resolves --tone on itself.
// Lazy: the skin constants are defined further down the module. (Surfaced by
// the sui-alert re-port pilot — bare controls in every shadow L2 otherwise.)
let controlsSheetMemo: CSSStyleSheet | null | undefined
function suiControlsSheet(): CSSStyleSheet | null {
  if (controlsSheetMemo === undefined) {
    controlsSheetMemo = makeSheet(layered(CONTROL_SKINS_CSS))
  }
  return controlsSheetMemo
}

export function adoptSuiSheets(root: unknown): void {
  if (typeof ShadowRoot === 'undefined') return
  if (!(root instanceof ShadowRoot)) return
  const sheets = [
    suiBaseSheet,
    suiTonesSheet,
    suiSizesSheet,
    suiTypographySheet,
    suiControlsSheet(),
  ].filter((s): s is CSSStyleSheet => s != null)
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
// ~18% of all baseline CSS was these, hand-repeated). GLOBAL: a SUI document
// is SUI's canvas, so these apply to every control in the light tree, not
// just component subtrees — "the web, with default styles." Interop with a
// foreign page is a DIFFERENT mode (emit a web component; its shadow root
// insulates it for free). :where() keeps specificity at zero so any explicit
// component/sx rule wins ties.
const CONTROL_CONTRACTS_CSS =
  // Control baseline: what a control IS before any skin.
  ':where(button, input, textarea, select) { font: inherit; font-family: var(--font-body); letter-spacing: inherit; color: inherit; }\n' +
  ':where(button) { line-height: 1; user-select: none; white-space: nowrap; cursor: pointer; }\n' +
  ':where(input, textarea, select) { appearance: none; }\n' +
  // Focus: one ring, everywhere, token-driven — never re-authored.
  ':where(button, input, textarea, select, a, [tabindex]):focus-visible { outline: var(--ring-width, 2px) solid var(--ring); outline-offset: var(--ring-offset, 2px); }\n' +
  '@media (forced-colors: active) { :where(button, input, textarea, select, a, [tabindex]):focus-visible { outline: 2px solid CanvasText; } }\n' +
  // Disabled: the treatment, once.
  ':where(button, input, textarea, select):disabled { cursor: not-allowed; opacity: 0.5; }\n' +
  // Interactive transition defaults from the motion tokens.
  ':where(button, input, textarea, select, a, [tabindex]) { transition: background-color var(--duration-fast) var(--ease-out), border-color var(--duration-fast) var(--ease-out), box-shadow var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out); }\n'

// L1 <input> / <textarea> / <select> — the field surface. A bordered control
// on the background, sized by the size channel and sharing the button's
// radius/shadow language, so a plain <input> IS the design system's field.
// select/textarea share it (native chrome already dropped via appearance:none
// in the baseline). Every rule at :where() zero specificity.
const TEXT_FIELD = "input:not([type='checkbox'], [type='radio'])"
const INPUT_SKIN_CSS =
  `:where(${TEXT_FIELD}, textarea, select) { display: block; min-height: var(--size-h, var(--control-h-md)); padding: var(--space-1_5) var(--size-px, var(--space-3)); font-size: var(--size-fs, var(--text-base)); color: var(--foreground); background: var(--background); border: 1px solid var(--border); border-radius: var(--radius-md); box-shadow: var(--shadow-xs); }\n` +
  `:where(${TEXT_FIELD}, textarea, select):hover:not(:disabled) { border-color: var(--border-strong); }\n` +
  `:where(${TEXT_FIELD}, textarea, select)::placeholder { color: var(--muted-foreground); }\n` +
  ':where(textarea) { min-height: calc(var(--control-h-md) * 2); resize: vertical; }\n'

// L1 checkbox / radio — appearance:none native inputs the framework renders.
// Square (checkbox) or round (radio); checked fills with the tone. The
// checkmark is a background SVG (baked white — reads on every tone fill, the
// same contrast --tone-contrast resolves to); the radio dot is an inset
// box-shadow ring. Theme-reactive via --tone/--border/--background.
// (SWITCH is intentionally not here: a sliding thumb needs a pseudo-element,
// which doesn't render on a replaced <input> — it wants a wrapper, so it
// lands as a structural element / L2, not a bare-input contract. LANG-GAP.)
const CHECK_MARK =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='none' stroke='white' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round' d='M3.5 8.5l3 3 6-6'/%3E%3C/svg%3E\")"
const TOGGLE_CONTROLS_CSS =
  // Shared box: sized to the font, bordered, on the background.
  ":where(input[type='checkbox'], input[type='radio']) { appearance: none; flex: none; width: 1.15em; height: 1.15em; min-height: 0; padding: 0; border: 1.5px solid var(--border-strong); background: var(--background); box-shadow: var(--shadow-xs); cursor: pointer; vertical-align: -0.2em; }\n" +
  ":where(input[type='checkbox']) { border-radius: var(--radius-sm); }\n" +
  ":where(input[type='radio']) { border-radius: 50%; }\n" +
  // Checked: fill with the tone (default → primary), stamp the mark.
  ":where(input[type='checkbox']:checked, input[type='radio']:checked) { background-color: var(--tone, var(--primary)); border-color: var(--tone, var(--primary)); }\n" +
  `:where(input[type='checkbox']:checked) { background-image: ${CHECK_MARK}; background-position: center; background-size: 0.9em; background-repeat: no-repeat; }\n` +
  ":where(input[type='radio']:checked) { box-shadow: inset 0 0 0 0.22em var(--background); }\n"

// L1 <button> — the first language element skin. A bare <button> anywhere in
// a SUI document IS the design system's button ("the web with default styles"):
// tone/size arrive through the attribute-keyed channel contracts above,
// variant surfaces mirror the hand-polished baseline, and every rule sits at
// :where() zero specificity so sx atomics and component variants win ties.
// Fallbacks carry the no-attribute defaults (tone=default, size=md).
const BUTTON_SKIN_CSS =
  ':where(button) { display: inline-flex; align-items: center; justify-content: center; gap: var(--size-gap, var(--space-2)); min-height: var(--size-h, var(--control-h-md)); padding: 0 var(--size-px, var(--space-3)); font-size: var(--size-fs, var(--text-base)); font-weight: var(--font-weight-medium); border-radius: var(--radius-md); border: 1px solid transparent; background: transparent; isolation: isolate; position: relative; }\n' +
  // default / no-variant: neutral bordered surface.
  ":where(button:not([variant]), button[variant='default']) { background: var(--background); border-color: var(--border); color: var(--foreground); box-shadow: var(--shadow-xs); }\n" +
  ":where(button:not([variant]), button[variant='default']):hover:not(:disabled) { background: var(--muted); border-color: var(--border-strong); }\n" +
  // solid: the tone fill. Hover shifts the fill TOWARD the foreground —
  // color-mix makes this mode-aware for free (foreground is dark in light
  // mode → darker; light in dark mode → lighter), so the button stays its
  // own tone instead of collapsing to a neutral grey, and --tone-contrast
  // text stays readable.
  ":where(button[variant='solid']) { background: var(--tone, var(--foreground)); border-color: var(--tone, var(--foreground)); color: var(--tone-contrast, var(--background)); box-shadow: var(--shadow-xs); }\n" +
  ":where(button[variant='solid']):hover:not(:disabled) { background: color-mix(in oklch, var(--tone, var(--foreground)), var(--foreground) 15%); border-color: color-mix(in oklch, var(--tone, var(--foreground)), var(--foreground) 15%); box-shadow: var(--shadow-sm); }\n" +
  // soft: the tinted surface. Hover deepens the tint by mixing in more of
  // the pure tone (stays in the tone family; --tone-fg text stays readable).
  ":where(button[variant='soft']) { background: var(--tone-bg, var(--hover-bg)); border-color: var(--tone-border, var(--border)); color: var(--tone-fg, var(--foreground)); }\n" +
  ":where(button[variant='soft']):hover:not(:disabled) { background: color-mix(in oklch, var(--tone-bg, var(--hover-bg)), var(--tone, var(--foreground)) 18%); border-color: var(--tone, var(--tone-border)); }\n" +
  // outline / ghost. Hover is a low-alpha wash of the TONE (not a neutral
  // grey), so it reads as the control's own colour in both modes.
  ":where(button[variant='outline']) { border-color: var(--tone, var(--foreground)); color: var(--tone-fg, var(--foreground)); }\n" +
  ":where(button[variant='outline'], button[variant='ghost']):hover:not(:disabled) { background: color-mix(in srgb, var(--tone, var(--foreground)) 10%, transparent); }\n" +
  ":where(button[variant='ghost']) { color: var(--tone-fg, var(--foreground)); }\n" +
  // Interaction motion (the baseline lift/press — reduced-motion kills it
  // via the global media rule).
  ':where(button):hover:not(:disabled) { transform: translateY(-1px); }\n' +
  ':where(button):active:not(:disabled) { transform: translateY(1px) scale(0.98); }\n' +
  // Disabled surface (the contract handles cursor/opacity; ghost/outline go
  // transparent like the baseline).
  ":where(button[variant='ghost'], button[variant='outline']):disabled { background: transparent; }\n"

// The CONTROL half of the contracts — element-keyed channels + control
// baseline + button/input/toggle skins, WITHOUT the light host rules
// (display:block, slot, [hidden]) which are the light-DOM component's job.
// This is what shadow roots adopt (via adoptSuiSheets) so their nested
// controls are skinned; the sui-alert pilot proved the boundary blocks the
// document contracts otherwise. Exported so a unit test guards the invariant
// (adoptedStyleSheets isn't constructable under jsdom, so the effect itself
// is browser-only).
export const CONTROL_SKINS_CSS =
  hostAttrs(TONES_CSS) +
  hostAttrs(SIZES_CSS) +
  CONTROL_CONTRACTS_CSS +
  BUTTON_SKIN_CSS +
  INPUT_SKIN_CSS +
  TOGGLE_CONTROLS_CSS

export const LIGHT_CONTRACTS_CSS =
  hostAttrs(TONES_CSS) +
  hostAttrs(SIZES_CSS) +
  CONTROL_CONTRACTS_CSS +
  BUTTON_SKIN_CSS +
  INPUT_SKIN_CSS +
  TOGGLE_CONTROLS_CSS +
  // Host defaults for light components (the :host base rules, element-keyed).
  '[data-component] { display: block; color: var(--foreground); }\n' +
  '[data-component][hidden] { display: none !important; }\n' +
  // <slot> stays in the light tree as the projection wrapper — transparent.
  '[data-component] slot { display: contents; }\n' +
  // Reduced motion kills transitions/animation for component subtrees AND the
  // globally-styled controls (their lift/press/transition are document-wide now).
  '@media (prefers-reduced-motion: reduce) { [data-component], [data-component] *, :where(button, input, textarea, select, a, [tabindex]) { transition: none !important; animation: none !important; } }\n'

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
  // A host that SERVES the contracts asset (deck links
  // /api/assets/sui.css, marked data-sui-contracts) wins — injection is
  // the migration-window fallback for JS-only hosts and dies when the
  // last one converts.
  if (document.querySelector('link[data-sui-contracts], style[data-sui-contracts]')) {
    return
  }
  const style = document.createElement('style')
  style.id = id
  style.textContent = layered(LIGHT_CONTRACTS_CSS)
  document.head.prepend(style)
}
