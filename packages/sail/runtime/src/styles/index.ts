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
  ":host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background);              --tone-bg: var(--tone-muted-bg);       --tone-border: var(--tone-muted-border);       --tone-ring: var(--tone-muted-ring); }\n" +
  // Light-DOM mirror for bare L1 field controls — <input tone=primary>/[invalid]
  // set the border+ring channels the field skin reads (`:host()` never matches a
  // light-DOM element). Border+ring only (what a field tints); parity w/ sail-input.
  ":where(input, select, textarea)[tone='primary']     { --tone-border: var(--tone-primary-border);     --tone-ring: var(--tone-primary-ring); }\n" +
  ":where(input, select, textarea)[tone='info']        { --tone-border: var(--tone-info-border);        --tone-ring: var(--tone-info-ring); }\n" +
  ":where(input, select, textarea)[tone='success']     { --tone-border: var(--tone-success-border);     --tone-ring: var(--tone-success-ring); }\n" +
  ":where(input, select, textarea)[tone='warning']     { --tone-border: var(--tone-warning-border);     --tone-ring: var(--tone-warning-ring); }\n" +
  ":where(input, select, textarea)[tone='accent']      { --tone-border: var(--tone-accent-border);      --tone-ring: var(--tone-accent-ring); }\n" +
  ":where(input, select, textarea)[tone='muted']       { --tone-border: var(--tone-muted-border);       --tone-ring: var(--tone-muted-ring); }\n" +
  ":where(input[invalid], input[aria-invalid='true'], select[invalid], select[aria-invalid='true'], textarea[invalid], textarea[aria-invalid='true']) { --tone-border: var(--destructive); --tone-ring: var(--tone-destructive-ring); }\n"

const SIZES_CSS =
  // The FULL size contract, framework-owned — the exact analogue of tone. A
  // component that declares a `size` prop gets the dimensional channels for free
  // (`--size-h` control height, `--size-px` inline padding, `--size-fs` font
  // size, `--size-gap` inner gap). No per-component <theme prop="size"> block:
  // one global scale keeps every control coherent, and we own the compiler.
  ":host([size='xs']) { --size-h: var(--control-h-2xs); --size-px: var(--space-2);   --size-fs: var(--text-xs);   --size-gap: var(--space-1); }\n" +
  ":host([size='sm']) { --size-h: var(--control-h-sm);  --size-px: var(--space-2_5); --size-fs: var(--text-sm);   --size-gap: var(--space-1_5); }\n" +
  ":host([size='md']) { --size-h: var(--control-h-md);  --size-px: var(--space-3);   --size-fs: var(--text-base); --size-gap: var(--space-2); }\n" +
  ":host([size='lg']) { --size-h: var(--control-h-lg);  --size-px: var(--space-4);   --size-fs: var(--text-md);   --size-gap: var(--space-2); }\n" +
  // Light-DOM mirror — bare L1 native controls (input/select/textarea/button)
  // carry `size` in light DOM, where `:host()` never matches. Same scale, scoped
  // to the controls so it only sets the vars their skins consume.
  ":where(input, select, textarea, button)[size='xs'] { --size-h: var(--control-h-2xs); --size-px: var(--space-2);   --size-fs: var(--text-xs);   --size-gap: var(--space-1); }\n" +
  ":where(input, select, textarea, button)[size='sm'] { --size-h: var(--control-h-sm);  --size-px: var(--space-2_5); --size-fs: var(--text-sm);   --size-gap: var(--space-1_5); }\n" +
  ":where(input, select, textarea, button)[size='md'] { --size-h: var(--control-h-md);  --size-px: var(--space-3);   --size-fs: var(--text-base); --size-gap: var(--space-2); }\n" +
  ":where(input, select, textarea, button)[size='lg'] { --size-h: var(--control-h-lg);  --size-px: var(--space-4);   --size-fs: var(--text-md);   --size-gap: var(--space-2); }\n"

// R3: shared host-resets adopted into every shadow component. Rules
// here are duplicated literally in nearly every <style> block; hoisting
// them lets the sail-compiler strip the per-component copies (see
// `crates/sail-compiler/src/emit/styles.rs`). Keep this list strictly to
// rules whose author-form text the compiler can recognize and remove.
// The box model EVERYTHING in this file is authored against. Every --control-h-*
// token, every min-height, every padding measures the BORDER box: the size you
// ask for is the size you get. Without it the framework silently inherits
// whatever box model the host page happens to have — a [size=md] input stops
// being --control-h-md, and a <textarea rows=4> stops honoring `rows` (the
// min-height floor swallows the rows whole). Specificity 0 and inside @layer sui,
// so an author still wins. Shared by BOTH surfaces below: shadow components adopt
// it via BASE_CSS, light-DOM pages get it via LIGHT_CONTRACTS_CSS. One definition.
const BOX_MODEL_CSS = '*, *::before, *::after { box-sizing: border-box; }\n'

const BASE_CSS =
  BOX_MODEL_CSS +
  // Framework-owned host defaults. Custom elements default to `display:inline`
  // (almost never wanted) and every component wants the foreground color — so
  // the framework decides both. A component needing a different display (inline
  // -flex, grid) overrides in its own styles, which win in the cascade. This is
  // what lets most <host> blocks disappear entirely.
  ':host { display: block; color: var(--foreground); }\n' +
  ':host([hidden]) { display: none !important; }\n' +
  // transition kill stays a blanket !important — hover/interaction micro-
  // transitions carry no component-authored reduced-motion intent to lose.
  // The animation kill is NOT !important and is wrapped in :where() (zero
  // specificity): a shadow component that owns its own reduced-motion
  // behavior (a slowdown, a pause) has real selector specificity inside
  // its own adopted stylesheet and wins the cascade unforced, instead of
  // being defeated by this default. Anything that has NOT opted in still
  // gets stopped here — this is the floor, not a universal override.
  '@media (prefers-reduced-motion: reduce) { :where(*) { transition: none !important; animation: none; } }\n'

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
  // No `color` here: body already hands paragraphs the foreground through
  // inheritance, and an explicit p ink rule beats inheritance in EVERY
  // ink-scoped container (a filled band, a terminal panel) — dark-on-dark.
  'p  { margin: 0; font-size: var(--text-md); line-height: 1.55; }\n' +
  'small { font-size: var(--text-sm); color: var(--muted-foreground); }\n' +
  // A link that declares a variant is a button link — the button skin owns
  // its ink; the global link colour must not outrank it (bare `a` beats the
  // skin's :where(0) inside the same layer).
  'a:not([variant]) { color: var(--primary); text-decoration: none; }\n' +
  'a:hover { text-decoration: underline; }\n' +
  'strong { font-weight: var(--heading-weight, 700); }\n' +
  'code { font-family: var(--mono-font, monospace); font-size: 0.9em; }\n' +
  // Inline code must READ as code, not merely be re-fonted: a muted chip sets it
  // apart from the prose around it (which is what shadcn and Mantine both do, and
  // what a bare font swap does not). Scoped away from `pre > code`, which would
  // otherwise stack a chip inside a block that already carries its own surface.
  ':not(pre) > code { padding: 0.15em 0.35em; background: var(--muted); border-radius: var(--radius-sm); }\n'

// SEMANTIC HTML DEFAULTS — "the web, with default styles". Every semantic
// element we own gets a token-based default so a plain <ul>/<table>/<img>/
// <blockquote> looks designed with no sx and no <style>. This is the L1
// promise applied to the whole element set: reserved by the compiler,
// styled by the framework, zero bundle cost until used. Media/lists/table/
// quote/inline — all token-driven, theme-reactive, @layer sui.
const SEMANTIC_ELEMENTS_CSS =
  // Media — never overflow the container.
  'img, video, canvas, svg { max-width: 100%; }\n' +
  'img, video { height: auto; }\n' +
  'figure { margin: 0; }\n' +
  'figcaption { margin-top: var(--space-2); font-size: var(--text-sm); color: var(--muted-foreground); }\n' +
  // Lists.
  'ul, ol { margin: 0; padding-left: var(--space-6); }\n' +
  'li { margin: var(--space-1) 0; }\n' +
  'dl { margin: 0; }\n' +
  'dt { font-weight: var(--font-weight-semibold, 600); color: var(--foreground); }\n' +
  'dd { margin: 0 0 var(--space-2) 0; color: var(--muted-foreground); }\n' +
  // Quotes.
  'blockquote { margin: 0; padding-left: var(--space-4); border-left: 2px solid var(--border-strong); color: var(--muted-foreground); }\n' +
  // Tables live in TABLE_SKIN_CSS — they're a control surface, not prose.
  // Inline emphasis.
  'mark { background: var(--tone-warning-bg, var(--hover-bg)); color: inherit; padding: 0 0.2em; border-radius: var(--radius-sm); }\n' +
  'kbd { font-family: var(--mono-font, monospace); font-size: 0.85em; padding: 0.1em 0.4em; border: 1px solid var(--border); border-bottom-width: 2px; border-radius: var(--radius-sm); background: var(--muted); color: var(--foreground); }\n' +
  'abbr[title] { text-decoration: underline dotted; cursor: help; }\n' +
  'sub, sup { font-size: 0.75em; line-height: 0; }\n' +
  // Code block.
  'pre { margin: 0; padding: var(--space-4); overflow-x: auto; background: var(--muted); border-radius: var(--radius-md); font-family: var(--mono-font, monospace); font-size: var(--text-sm); line-height: 1.5; }\n' +
  'pre code { font-size: inherit; }\n' +
  // Rule.
  'hr { border: 0; border-top: 1px solid var(--border); margin: var(--space-4) 0; }\n' +
  // Fluid media embeds.
  'iframe { max-width: 100%; border: 0; }\n'

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
export const suiTypographySheet: CSSStyleSheet | null = makeSheet(
  layered(TYPOGRAPHY_CSS + SEMANTIC_ELEMENTS_CSS),
)

/**
 * Adopt every shared sheet into a shadow root (no-op for light DOM /
 * environments without `CSSStyleSheet`).
 *
 * Called once per shadow component instance from compiler-emitted code
 * (see `crates/sail-compiler/src/emit/js.rs`). Centralising the call
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
// the sail-alert re-port pilot — bare controls in every shadow L2 otherwise.)
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
// light L2 <sail-chip> identically, and dynamic `tone={x()}` restyles both.
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
  ':where(button, a[variant]) { line-height: 1; user-select: none; white-space: nowrap; cursor: pointer; }\n' +
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
// Text-entry fields only — NOT the toggles (checkbox/radio), the slider
// (range), or the color swatch, which carry their own skins and must not
// inherit the bordered text-field box. (file/number/search/date DO want the
// box — they read as fields — so they stay in.)
const TEXT_FIELD =
  "input:not([type='checkbox'], [type='radio'], [type='range'], [type='color'])"
const INPUT_SKIN_CSS =
  `:where(${TEXT_FIELD}, textarea, select) { display: block; min-height: var(--size-h, var(--control-h-md)); padding: var(--space-1_5) var(--size-px, var(--space-3)); font-size: var(--size-fs, var(--text-base)); color: var(--foreground); background: var(--background); border: 1px solid var(--tone-border, var(--border)); border-radius: var(--radius-md); box-shadow: var(--shadow-xs); }\n` +
  // Hover strengthens the border only when the field is NOT toned/invalid
  // (a toned border already reads); otherwise it keeps its tone.
  `:where(${TEXT_FIELD}, textarea, select):hover:not(:disabled):not([tone]):not([invalid]):not([aria-invalid='true']) { border-color: var(--border-strong); }\n` +
  `:where(${TEXT_FIELD}, textarea, select)::placeholder { color: var(--muted-foreground); }\n` +
  ':where(textarea) { min-height: calc(var(--control-h-md) * 2); resize: vertical; }\n' +
  // <select> — appearance:none dropped the native arrow, so draw a chevron.
  // A background SVG (neutral gray so it reads in both themes — a data-URI
  // can't inherit currentColor); padding-right clears the glyph.
  ":where(select) { padding-right: var(--space-8); background-image: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='16' height='16' viewBox='0 0 16 16' fill='none' stroke='%23888888' stroke-width='1.75' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M4 6l4 4 4-4'/%3E%3C/svg%3E\"); background-repeat: no-repeat; background-position: right var(--space-3) center; background-size: 1rem; }\n" +
  // Native option list — reset so the popup isn't OS-default black-on-white.
  ':where(select) option { color: var(--foreground); background: var(--background); }\n' +
  // <input type=file> — keeps the field box, but flex-centers its parts and
  // restyles the selector button as a small secondary button. Standard pseudo
  // + legacy WebKit alias (separate rules — a grouped selector is dropped).
  ":where(input[type='file']) { display: flex; align-items: center; padding: var(--space-1) var(--space-3); cursor: pointer; }\n" +
  ":where(input[type='file'])::file-selector-button { margin-right: var(--space-3); padding: var(--space-1) var(--space-3); font: inherit; font-weight: var(--font-weight-medium); color: var(--foreground); background: var(--background); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: var(--shadow-xs); cursor: pointer; }\n" +
  ":where(input[type='file'])::-webkit-file-upload-button { margin-right: var(--space-3); padding: var(--space-1) var(--space-3); font: inherit; font-weight: var(--font-weight-medium); color: var(--foreground); background: var(--background); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: var(--shadow-xs); cursor: pointer; }\n" +
  ":where(input[type='file'])::file-selector-button:hover { background: var(--muted); border-color: var(--border-strong); }\n" +
  // <input type=color> — a rounded swatch button (not a text field).
  ":where(input[type='color']) { appearance: none; -webkit-appearance: none; -moz-appearance: none; width: var(--space-9); height: var(--control-h-md); padding: var(--space-1); border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--background); box-shadow: var(--shadow-xs); cursor: pointer; }\n" +
  ":where(input[type='color'])::-webkit-color-swatch-wrapper { padding: 0; }\n" +
  ":where(input[type='color'])::-webkit-color-swatch { border: 0; border-radius: var(--radius-sm); }\n" +
  ":where(input[type='color'])::-moz-color-swatch { border: 0; border-radius: var(--radius-sm); }\n" +
  // The last bits of native input chrome — the date/time picker indicator and
  // the search clear button. These already sit in the text-field box; just make
  // them cursor-consistent and dim-until-hover so they don't shout.
  ":where(input)::-webkit-calendar-picker-indicator { cursor: pointer; opacity: 0.55; }\n" +
  ":where(input)::-webkit-calendar-picker-indicator:hover { opacity: 1; }\n" +
  ":where(input[type='search'])::-webkit-search-cancel-button { cursor: pointer; }\n" +
  // <optgroup> label in the native option list.
  ":where(optgroup) { font-weight: var(--font-weight-semibold); color: var(--muted-foreground); }\n"

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
// The tri-state (indeterminate) glyph — a dash. Tree/table row-selection
// drive this via `el.indeterminate = true` in JS; the contract must paint it.
const DASH_MARK =
  "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='none' stroke='white' stroke-width='2.5' stroke-linecap='round' d='M4 8h8'/%3E%3C/svg%3E\")"
const TOGGLE_CONTROLS_CSS =
  // Shared box: sized to the font, bordered, on the background. The box is
  // 1.15em square, so the `[size=...]` channel's --size-fs (shared with
  // button/input) scales it too — same channel, same fallback (md/text-base).
  ":where(input[type='checkbox'], input[type='radio']) { appearance: none; flex: none; width: 1.15em; height: 1.15em; min-height: 0; padding: 0; font-size: var(--size-fs, var(--text-base)); border: 1.5px solid var(--border-strong); background: var(--background); box-shadow: var(--shadow-xs); cursor: pointer; vertical-align: -0.2em; }\n" +
  ":where(input[type='checkbox']) { border-radius: var(--radius-sm); }\n" +
  ":where(input[type='radio']) { border-radius: 50%; }\n" +
  // Checked: fill with the tone (default → primary), stamp the mark.
  ":where(input[type='checkbox']:checked, input[type='radio']:checked) { background-color: var(--tone, var(--primary)); border-color: var(--tone, var(--primary)); }\n" +
  `:where(input[type='checkbox']:checked) { background-image: ${CHECK_MARK}; background-position: center; background-size: 0.9em; background-repeat: no-repeat; }\n` +
  ":where(input[type='radio']:checked) { box-shadow: inset 0 0 0 0.22em var(--background); }\n" +
  // Indeterminate (tri-state) checkbox — tone fill + dash. `:indeterminate`
  // is a live DOM state (set in JS), not an attribute, so it only reaches
  // here; tree/table select-all glyphs rely on it.
  ":where(input[type='checkbox']:indeterminate) { background-color: var(--tone, var(--primary)); border-color: var(--tone, var(--primary)); }\n" +
  `:where(input[type='checkbox']:indeterminate) { background-image: ${DASH_MARK}; background-position: center; background-size: 0.9em; background-repeat: no-repeat; }\n` +
  // Invalid: destructive border override (an [invalid] attribute, mirroring
  // the field/select convention elsewhere in the framework — native
  // `aria-invalid="true"` is honored too so plain HTML forms get it for free).
  ":where(input[type='checkbox'][invalid], input[type='radio'][invalid], input[type='checkbox'][aria-invalid='true'], input[type='radio'][aria-invalid='true']) { border-color: var(--destructive); }\n" +
  ":where(input[type='checkbox'][invalid]:checked, input[type='radio'][invalid]:checked, input[type='checkbox'][aria-invalid='true']:checked, input[type='radio'][aria-invalid='true']:checked) { background-color: var(--destructive); border-color: var(--destructive); }\n"

// L1 <button> — the first language element skin. A bare <button> anywhere in
// a SUI document IS the design system's button ("the web with default styles"):
// tone/size arrive through the attribute-keyed channel contracts above,
// variant surfaces mirror the hand-polished baseline, and every rule sits at
// :where() zero specificity so sx atomics and component variants win ties.
// Fallbacks carry the no-attribute defaults (tone=default, size=md).
const BUTTON_SKIN_CSS =
  ':where(a[variant]) { text-decoration: none; }\n' +
  ':where(button, a[variant]) { display: inline-flex; align-items: center; justify-content: center; gap: var(--size-gap, var(--space-2)); min-height: var(--size-h, var(--control-h-md)); padding: 0 var(--size-px, var(--space-3)); font-size: var(--size-fs, var(--text-base)); font-weight: var(--font-weight-medium); border-radius: var(--radius-md); border: 1px solid transparent; background: transparent; isolation: isolate; position: relative; }\n' +
  // default / no-variant: neutral bordered surface.
  ":where(button:not([variant]), button[variant='default'], a[variant='default']) { background: var(--background); border-color: var(--border); color: var(--foreground); box-shadow: var(--shadow-xs); }\n" +
  ":where(button:not([variant]), button[variant='default'], a[variant='default']):hover:not(:disabled) { background: var(--muted); border-color: var(--border-strong); }\n" +
  // solid: the tone fill. Hover shifts the fill TOWARD the foreground —
  // color-mix makes this mode-aware for free (foreground is dark in light
  // mode → darker; light in dark mode → lighter), so the button stays its
  // own tone instead of collapsing to a neutral grey, and --tone-contrast
  // text stays readable.
  ":where(button[variant='solid'], a[variant='solid']) { background: var(--tone, var(--foreground)); border-color: var(--tone, var(--foreground)); color: var(--tone-contrast, var(--background)); box-shadow: var(--shadow-xs); }\n" +
  ":where(button[variant='solid'], a[variant='solid']):hover:not(:disabled) { background: color-mix(in oklch, var(--tone, var(--foreground)), var(--foreground) 15%); border-color: color-mix(in oklch, var(--tone, var(--foreground)), var(--foreground) 15%); box-shadow: var(--shadow-sm); }\n" +
  // soft: the tinted surface. Hover deepens the tint by mixing in more of
  // the pure tone (stays in the tone family; --tone-fg text stays readable).
  ":where(button[variant='soft'], a[variant='soft']) { background: var(--tone-bg, var(--hover-bg)); border-color: var(--tone-border, var(--border)); color: var(--tone-fg, var(--foreground)); }\n" +
  ":where(button[variant='soft'], a[variant='soft']):hover:not(:disabled) { background: color-mix(in oklch, var(--tone-bg, var(--hover-bg)), var(--tone, var(--foreground)) 18%); border-color: var(--tone, var(--tone-border)); }\n" +
  // outline / ghost. Hover is a low-alpha wash of the TONE (not a neutral
  // grey), so it reads as the control's own colour in both modes.
  ":where(button[variant='outline'], a[variant='outline']) { border-color: var(--tone, var(--foreground)); color: var(--tone-fg, var(--foreground)); }\n" +
  ":where(button[variant='outline'], button[variant='ghost'], a[variant='outline'], a[variant='ghost']):hover:not(:disabled) { background: color-mix(in srgb, var(--tone, var(--foreground)) 10%, transparent); }\n" +
  ":where(button[variant='ghost'], a[variant='ghost']) { color: var(--tone-fg, var(--foreground)); }\n" +
  // Interaction motion (the baseline lift/press — reduced-motion kills it
  // via the global media rule).
  ':where(button, a[variant]):hover:not(:disabled) { transform: translateY(-1px); }\n' +
  ':where(button, a[variant]):active:not(:disabled) { transform: translateY(1px) scale(0.98); }\n' +
  // Disabled surface (the contract handles cursor/opacity; ghost/outline go
  // transparent like the baseline).
  ":where(button[variant='ghost'], button[variant='outline']):disabled { background: transparent; }\n"

// L1 <hr> — the separator surface. Horizontal by default (a thin tone-tinted
// rule); `orientation="vertical"` and `variant="dashed"` are attribute-keyed
// like every other channel here. tone arrives for free via the generic
// `[tone=...]` channel above (--tone-bg), so `<hr tone="primary">` just works.
const HR_SKIN_CSS =
  ":where(hr) { border: 0; margin: var(--space-2) 0; padding: 0; flex: 0 0 auto; display: block; width: 100%; height: 1px; background: var(--tone-bg, var(--border-subtle)); }\n" +
  ":where(hr[variant='dashed']) { background: transparent; border-top: 1px dashed var(--tone-bg, var(--border-subtle)); height: 0; }\n" +
  ":where(hr[orientation='vertical']) { display: inline-block; align-self: stretch; vertical-align: middle; width: 1px; height: auto; min-height: 1em; margin: 0 var(--space-2); background: var(--border); }\n" +
  ":where(hr[orientation='vertical'][variant='dashed']) { background: transparent; border-left: 1px dashed var(--border); width: 0; }\n"

// L1 <span dot> — the status-dot surface. `dot` scopes the skin so a plain
// <span size="..."> elsewhere isn't affected (its size scale means something
// different than the control size channel above). tone arrives for free via
// the generic `[tone=...]` channel (--tone-fg/--tone-border/--tone-ring).
const STATUS_DOT_SKIN_CSS =
  ":where(span[dot]) { display: inline-block; vertical-align: middle; width: var(--icon-size-3xs); height: var(--icon-size-3xs); border-radius: var(--radius-full); background: var(--tone-fg); box-shadow: 0 0 0 1.5px var(--tone-border), 0 0 0 4.5px var(--tone-ring); }\n" +
  ":where(span[dot][size='sm']) { width: var(--space-1_5); height: var(--space-1_5); box-shadow: 0 0 0 1px var(--tone-border), 0 0 0 3.5px var(--tone-ring); }\n" +
  ":where(span[dot][pulse]) { animation: sail-status-dot-pulse 2.5s var(--ease-out) infinite; }\n" +
  '@keyframes sail-status-dot-pulse { 0%, 100% { opacity: 1; transform: scale(1); } 50% { opacity: 0.82; transform: scale(0.92); } }\n'

// L1 <span skeleton> — the shimmer-placeholder surface. `skeleton` scopes
// the skin the same way `dot` does above. shape/size drive width/height/
// radius; the shimmer gradient/keyframe and the circle-lg 48px are
// off-vocabulary for sx (no `bg:linear-gradient()`, no animation primitive,
// no 48px token — see the former wrapper's own LANG-GAP note), so they land
// here instead.
const SKELETON_SKIN_CSS =
  ":where(span[skeleton]) { display: inline-block; vertical-align: middle; background: linear-gradient(90deg, var(--muted) 0%, var(--tone-muted-bg) 50%, var(--muted) 100%); background-size: 200% 100%; animation: sail-skeleton-shimmer 1.5s ease-in-out infinite; border-radius: var(--radius-sm); }\n" +
  ":where(span[skeleton][shape='block']) { display: block; width: 100%; min-height: 80px; }\n" +
  ":where(span[skeleton][shape='circle']) { border-radius: var(--radius-full); }\n" +
  ":where(span[skeleton][shape='text'], span[skeleton][shape='line']) { width: 100%; }\n" +
  ":where(span[skeleton][shape='text'][size='sm'], span[skeleton][shape='line'][size='sm']) { height: var(--space-2); }\n" +
  ":where(span[skeleton][shape='text'][size='md'], span[skeleton][shape='line'][size='md']) { height: var(--space-3); }\n" +
  ":where(span[skeleton][shape='text'][size='lg'], span[skeleton][shape='line'][size='lg']) { height: var(--space-4); }\n" +
  ":where(span[skeleton][shape='circle'][size='sm']) { width: var(--icon-size-2xl); height: var(--icon-size-2xl); }\n" +
  ":where(span[skeleton][shape='circle'][size='md']) { width: var(--space-7); height: var(--space-7); }\n" +
  ":where(span[skeleton][shape='circle'][size='lg']) { width: 48px; height: 48px; }\n" +
  '@keyframes sail-skeleton-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }\n'

// L1 drop zone — `<div dropzone use:dropzone>` around a native file input.
// The div is the drop surface + click-to-browse trigger (wired by the action);
// the input is visually hidden but keeps form participation + the picker. A div
// (not a label) so the compiler's form-label skin doesn't fight this one; the
// container carries no compiler skin, so a clean zero-specificity `:where()`
// wins and authors can still override.
const DROPZONE_SKIN_CSS =
  ":where([dropzone]) { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: var(--space-1); min-height: calc(var(--control-h-lg) * 2); padding: var(--space-6); text-align: center; border: 1.5px dashed var(--border-strong); border-radius: var(--radius-lg); background: var(--muted); color: var(--muted-foreground); font-size: var(--text-sm); cursor: pointer; transition: border-color var(--duration-fast) var(--ease-out), background-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out); }\n" +
  ":where([dropzone]:hover) { border-color: var(--primary); color: var(--foreground); }\n" +
  ":where([dropzone][data-drag]) { border-color: var(--primary); border-style: solid; background: var(--tone-primary-bg, var(--hover-bg)); color: var(--foreground); }\n" +
  // Prompt line (medium) vs the hint / reflected filename (small, muted).
  ":where([dropzone] > span:first-of-type) { font-weight: var(--font-weight-medium); color: inherit; }\n" +
  ":where([dropzone] strong) { color: var(--primary); font-weight: var(--font-weight-medium); }\n" +
  ":where([dropzone] [data-dz-files]) { font-size: var(--text-xs); font-weight: var(--font-weight-normal); color: var(--muted-foreground); }\n" +
  // The nested file input — visually hidden, present for form value + picker.
  ":where([dropzone] input[type='file']) { position: absolute; width: 1px; height: 1px; padding: 0; margin: 0; border: 0; opacity: 0; overflow: hidden; clip: rect(0 0 0 0); pointer-events: none; }\n"

// L1 <fieldset>/<legend> — the form group. Native defaults (3D groove border,
// odd padding, min-content width) are the web's ugliest built-in; this reframes
// it as a clean bordered section with the legend riding the top edge. `min-
// inline-size:0` kills the legacy min-content sizing so it flexes normally.
const FIELDSET_SKIN_CSS =
  ":where(fieldset) { min-inline-size: 0; margin: 0; padding: var(--space-4); border: 1px solid var(--border); border-radius: var(--radius-md); background: transparent; }\n" +
  ":where(legend) { padding: 0 var(--space-2); font-size: var(--text-sm); font-weight: var(--font-weight-medium); color: var(--foreground); }\n" +
  // <output> — a live form result; tabular figures so recomputes don't jitter.
  ":where(output) { font-variant-numeric: tabular-nums; }\n"

// L1 <input type=range> — the slider. appearance:none drops the OS control;
// the skin rebuilds track + thumb from tokens. WebKit and Firefox expose the
// track/thumb through different pseudo-elements (and, once again, a grouped
// selector is dropped whole by either), so every rule is per-engine. Firefox
// fills the traversed portion for free via ::-moz-range-progress; the WebKit
// filled-track needs a value-driven gradient (JS), left as an enhancement.
// Parity target: feat/next sail-slider — a muted rail with an inset border, a
// tone fill with a top highlight, and a WHITE (--surface-1) thumb ringed in the
// tone with a shadow that lifts+scales on hover/active. Size vars drive sm/md/lg.
const RANGE_SKIN_CSS =
  ":where(input[type='range']) { appearance: none; -webkit-appearance: none; -moz-appearance: none; width: 100%; height: var(--space-5); background: transparent; cursor: pointer; --rng-track: var(--space-1_5); --rng-thumb: var(--space-4); }\n" +
  ":where(input[type='range'][size='sm']) { --rng-track: var(--space-1); --rng-thumb: var(--space-3); }\n" +
  ":where(input[type='range'][size='lg']) { --rng-track: var(--space-2); --rng-thumb: var(--space-5); }\n" +
  // Track — muted rail with a subtle inset border (matches sail-slider .track).
  ":where(input[type='range'])::-webkit-slider-runnable-track { height: var(--rng-track); border-radius: var(--radius-full); background: linear-gradient(to right, var(--tone-fg, var(--primary)) 0 var(--rng-pct, 0%), var(--muted) var(--rng-pct, 0%) 100%); box-shadow: inset 0 0 0 1px var(--border); }\n" +
  ":where(input[type='range'])::-moz-range-track { height: var(--rng-track); border-radius: var(--radius-full); background: var(--muted); box-shadow: inset 0 0 0 1px var(--border); }\n" +
  // Fill (Firefox) — tone with a top highlight (matches sail-slider .fill).
  ":where(input[type='range'])::-moz-range-progress { height: var(--rng-track); border-radius: var(--radius-full); background: var(--tone-fg, var(--primary)); box-shadow: inset 0 1px 0 var(--surface-2); }\n" +
  // Thumb — white surface disc, tone ring, shadow; centered on the thinner track.
  ":where(input[type='range'])::-webkit-slider-thumb { appearance: none; -webkit-appearance: none; box-sizing: border-box; width: var(--rng-thumb); height: var(--rng-thumb); margin-top: calc((var(--rng-track) - var(--rng-thumb)) / 2); border-radius: 50%; background: var(--surface-1); border: 2px solid var(--tone-fg, var(--primary)); box-shadow: var(--shadow-sm); transition: box-shadow var(--duration-fast) var(--ease-out), transform var(--duration-fast) var(--ease-out); }\n" +
  ":where(input[type='range'])::-moz-range-thumb { box-sizing: border-box; width: var(--rng-thumb); height: var(--rng-thumb); border-radius: 50%; background: var(--surface-1); border: 2px solid var(--tone-fg, var(--primary)); box-shadow: var(--shadow-sm); transition: box-shadow var(--duration-fast) var(--ease-out), transform var(--duration-fast) var(--ease-out); }\n" +
  // Hover lift + press scale (matches sail-slider .thumb:hover/:active).
  ":where(input[type='range']:hover)::-webkit-slider-thumb { box-shadow: var(--shadow-md); transform: scale(1.05); }\n" +
  ":where(input[type='range']:hover)::-moz-range-thumb { box-shadow: var(--shadow-md); transform: scale(1.05); }\n" +
  ":where(input[type='range']:active)::-webkit-slider-thumb { transform: scale(0.95); }\n" +
  ":where(input[type='range']:active)::-moz-range-thumb { transform: scale(0.95); }\n" +
  // Focus — ring on the thumb (the control's own :focus-visible outline would
  // wrap the full-width invisible box).
  ":where(input[type='range']):focus-visible::-webkit-slider-thumb { outline: var(--ring-width, 2px) solid var(--ring); outline-offset: 2px; }\n" +
  ":where(input[type='range']):focus-visible::-moz-range-thumb { outline: var(--ring-width, 2px) solid var(--ring); outline-offset: 2px; }\n" +
  ":where(input[type='range']):disabled { opacity: 0.5; cursor: not-allowed; }\n" +
  "@media (prefers-reduced-motion: reduce) { :where(input[type='range'])::-webkit-slider-thumb, :where(input[type='range'])::-moz-range-thumb { transition: none; } }\n"

// L1 <progress> — the determinate/indeterminate progress bar. A bare native
// <progress value=.6> IS the design system's progress: the platform gives the
// value semantics + a11y for free; this skin just replaces the OS chrome with
// the token vocabulary. tone arrives via the generic `[tone=...]` channel
// (--tone-fg), so `<progress tone=success>` tints the fill. WebKit and Firefox
// expose the fill through DIFFERENT pseudo-elements, and a rule that groups
// both selectors is dropped whole by either engine — so they're separate.
const PROGRESS_SKIN_CSS =
  ":where(progress) { appearance: none; -webkit-appearance: none; -moz-appearance: none; display: inline-block; vertical-align: middle; width: 100%; height: var(--progress-h, var(--space-2)); border: 0; border-radius: var(--radius-full); overflow: hidden; background: var(--muted); color: var(--tone-fg, var(--primary)); }\n" +
  ":where(progress[size='sm']) { height: var(--space-1_5); }\n" +
  ":where(progress[size='lg']) { height: var(--space-3); }\n" +
  // Track (WebKit) — Firefox uses the element background set above.
  ":where(progress)::-webkit-progress-bar { background: var(--muted); border-radius: var(--radius-full); }\n" +
  // Determinate fill — one rule per engine (a shared selector is invalid to both).
  ":where(progress)::-webkit-progress-value { background: var(--tone-fg, var(--primary)); border-radius: var(--radius-full); transition: width var(--duration-medium) var(--ease-out); }\n" +
  ":where(progress)::-moz-progress-bar { background: var(--tone-fg, var(--primary)); border-radius: var(--radius-full); }\n" +
  // Indeterminate (no [value]) — a sliding sheen, since appearance:none drops
  // the native barber-pole. Animate the WebKit track and the element itself
  // (the Firefox path) so both engines show motion.
  ":where(progress:not([value]))::-webkit-progress-bar { background-image: linear-gradient(90deg, transparent 0%, var(--tone-fg, var(--primary)) 50%, transparent 100%); background-repeat: no-repeat; background-size: 40% 100%; animation: sail-progress-indeterminate 1.2s ease-in-out infinite; }\n" +
  ":where(progress:not([value])) { background-image: linear-gradient(90deg, transparent 0%, var(--tone-fg, var(--primary)) 50%, transparent 100%); background-repeat: no-repeat; background-size: 40% 100%; animation: sail-progress-indeterminate 1.2s ease-in-out infinite; }\n" +
  "@keyframes sail-progress-indeterminate { 0% { background-position: -40% 0; } 100% { background-position: 140% 0; } }\n" +
  "@media (prefers-reduced-motion: reduce) { :where(progress), :where(progress)::-webkit-progress-value { animation: none !important; transition: none !important; } }\n"

// L1 <meter> — the gauge (a value inside a range: disk usage, score, load).
// Distinct from <progress>: the fill color reflects WHERE the value falls
// relative to low/high/optimum, not just how full it is. The platform owns
// that banding; the skin maps the three bands to the tone tokens. WebKit
// exposes the bands as pseudo-ELEMENTS; Firefox as pseudo-CLASSES on a single
// ::-moz-meter-bar — so, again, one rule per engine.
const METER_SKIN_CSS =
  ":where(meter) { appearance: none; -webkit-appearance: none; -moz-appearance: none; display: inline-block; vertical-align: middle; width: 100%; height: var(--meter-h, var(--space-2)); border: 0; border-radius: var(--radius-full); overflow: hidden; background: var(--muted); }\n" +
  ":where(meter[size='sm']) { height: var(--space-1_5); }\n" +
  ":where(meter[size='lg']) { height: var(--space-3); }\n" +
  // WebKit — track + one pseudo-element per band.
  ":where(meter)::-webkit-meter-bar { background: var(--muted); border: 0; border-radius: var(--radius-full); }\n" +
  ":where(meter)::-webkit-meter-optimum-value { background: var(--success); border-radius: var(--radius-full); }\n" +
  ":where(meter)::-webkit-meter-suboptimum-value { background: var(--warning); border-radius: var(--radius-full); }\n" +
  ":where(meter)::-webkit-meter-even-less-good-value { background: var(--destructive); border-radius: var(--radius-full); }\n" +
  // Firefox — one fill pseudo-element, colored by a band pseudo-class on the
  // meter. (Kept out of :where() so the vendor selector isn't folded away.)
  "meter::-moz-meter-bar { border-radius: var(--radius-full); }\n" +
  "meter:-moz-meter-optimum::-moz-meter-bar { background: var(--success); }\n" +
  "meter:-moz-meter-sub-optimum::-moz-meter-bar { background: var(--warning); }\n" +
  "meter:-moz-meter-sub-sub-optimum::-moz-meter-bar { background: var(--destructive); }\n"

// L1 <details>/<summary> — the native disclosure / accordion. Zero JS: the
// platform owns the open/close toggle and the a11y. The skin frames it, hides
// the OS triangle, draws a rotating chevron, tints the open header, and animates
// open/close natively. Parity target: the feat/next sail-accordion-item (semibold
// text-sm trigger, --muted open tint, down-chevron rotating 180°, medium-duration
// height animation). Group several with a wrapper to build an accordion.
// Down-chevron (matches sail-accordion-item's M4 6l4 4 4-4), masked so it recolors
// with the theme via `background`. Backtick avoids escaping the url("…") quotes.
const CHEVRON_DOWN = `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23000' stroke-width='1.75' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M4 6l4 4 4-4'/%3E%3C/svg%3E")`
const DETAILS_SKIN_CSS =
  // interpolate-size lets ::details-content animate block-size to `auto`.
  ":where(details) { border: 1px solid var(--border); border-radius: var(--radius-lg); background: var(--surface-1, var(--background)); interpolate-size: allow-keywords; }\n" +
  // Stacked adjacent <details> MERGE into one accordion container (the baseline
  // sail-accordion look): the shared 1px border becomes the row divider, and only
  // the group's outer corners round. :has()/adjacent-sibling — no wrapper element.
  ":where(details:has(+ details)) { border-bottom-left-radius: 0; border-bottom-right-radius: 0; }\n" +
  ":where(details + details) { border-top: 0; border-top-left-radius: 0; border-top-right-radius: 0; }\n" +
  ":where(details + details) > summary { border-radius: 0; }\n" +
  ":where(summary) { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-3) var(--space-4); font-size: var(--text-sm); font-weight: var(--font-weight-semibold); color: var(--foreground); cursor: pointer; list-style: none; user-select: none; -webkit-user-select: none; border-radius: var(--radius-lg); transition: background var(--duration-fast) var(--ease-out); }\n" +
  // Hide the platform marker (standard + WebKit) — the chevron replaces it.
  ":where(summary)::-webkit-details-marker { display: none; }\n" +
  ":where(summary)::marker { content: ''; }\n" +
  // Chevron at the trailing edge; rotates 180° and brightens to --foreground on open.
  ":where(summary)::after { content: ''; margin-inline-start: auto; flex: 0 0 auto; width: 16px; height: 16px; background: var(--muted-foreground); -webkit-mask: " + CHEVRON_DOWN + " center / 16px no-repeat; mask: " + CHEVRON_DOWN + " center / 16px no-repeat; transition: transform var(--duration-medium) var(--ease-out), background var(--duration-fast) var(--ease-out); }\n" +
  ":where(details[open]) > summary::after { transform: rotate(180deg); background: var(--foreground); }\n" +
  ":where(summary:hover) { background: var(--hover-bg); }\n" +
  ":where(summary:focus-visible) { outline: var(--focus-ring); outline-offset: calc(-1 * var(--focus-ring-offset, 2px)); }\n" +
  // Open row header tints with --muted so the active section reads at a glance.
  ":where(details[open]) > summary { background: var(--muted); border-bottom: 1px solid var(--border-subtle); border-radius: var(--radius-lg) var(--radius-lg) 0 0; }\n" +
  // Native open/close height animation (progressive enhancement: Chrome 129+;
  // engines without ::details-content simply snap — content still shows).
  ":where(details)::details-content { block-size: 0; overflow: hidden; transition: block-size var(--duration-medium) var(--ease-out), content-visibility var(--duration-medium) allow-discrete; }\n" +
  ":where(details[open])::details-content { block-size: auto; }\n" +
  // Body — the content after <summary>. Margin (not padding) so stacked blocks
  // collapse to a single inter-gap instead of doubling.
  ":where(details) > summary ~ * { margin: var(--space-3) var(--space-4); }\n" +
  "@media (prefers-reduced-motion: reduce) { :where(summary)::after, :where(details)::details-content { transition: none; } }\n"

// L1 <dialog> — the modal surface. A bare native <dialog> IS the design
// system's modal ("the web with default styles"): showModal() puts it in the
// top layer and the platform gives us ::backdrop, ESC, focus-trap, and
// focus-return for free — no sentinels, no scroll-lock JS beyond a :has()
// page rule, no custom element. This is the skin: panel, backdrop, size
// matrix, and the header/body/footer structure. size + backdrop are
// attribute-keyed like every other channel here.
const DIALOG_SKIN_CSS =
  // The panel — centered in the top layer, a flex column so header/footer pin
  // and the body scrolls. Closed state is the UA default (display:none), so
  // the layout only lands on [open] and there's no closed-state artifact.
  ':where(dialog) { position: fixed; inset: 0; margin: auto; padding: 0; width: calc(100vw - var(--space-7)); max-width: var(--panel-max-w-md); max-height: min(calc(100vh - var(--space-7)), var(--panel-vh-md)); overflow: hidden; border: 1px solid var(--border); border-radius: var(--radius-xl); background: var(--surface-1); color: var(--card-foreground); box-shadow: var(--shadow-xl); font-family: var(--font-body); }\n' +
  ':where(dialog[open]) { display: flex; flex-direction: column; }\n' +
  // Backdrop — dim by default; blur / solid are attribute-keyed. Fades in.
  ':where(dialog)::backdrop { background: var(--backdrop); opacity: 0; transition: opacity var(--duration-medium) var(--ease-out); }\n' +
  ':where(dialog[open])::backdrop { opacity: 1; }\n' +
  ":where(dialog[backdrop='blur'])::backdrop { backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px); }\n" +
  ":where(dialog[backdrop='solid'])::backdrop { background: var(--background); }\n" +
  // Size matrix — max-width steps; full goes near-viewport.
  ":where(dialog[size='sm']) { max-width: var(--panel-max-w-sm); }\n" +
  ":where(dialog[size='lg']) { max-width: var(--panel-max-w-lg); }\n" +
  ":where(dialog[size='full']) { max-width: var(--panel-vh-lg); max-height: var(--panel-vh-lg); }\n" +
  // Structure — semantic children the author writes: <header> and <footer>
  // pin, a <section> (the body) scrolls. Loose content (no <section>) rides
  // the panel's own max-height.
  ':where(dialog) > header { flex: 0 0 auto; padding: var(--space-4_5) var(--space-6); border-bottom: 1px solid var(--border-subtle); font-size: var(--text-lg); font-weight: var(--font-weight-semibold); color: var(--foreground); }\n' +
  ':where(dialog) > section { flex: 1 1 auto; min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: var(--space-4) var(--space-6); font-size: var(--text-base); line-height: var(--leading-normal); }\n' +
  ':where(dialog) > footer { flex: 0 0 auto; display: flex; gap: var(--space-2); align-items: center; justify-content: flex-end; padding: var(--space-4) var(--space-6); border-top: 1px solid var(--border-subtle); background: var(--tone-muted-border); }\n' +
  // A bare entry inside owning chrome (command palettes, inline editors): the
  // wrapper draws the surface, the input contributes none.
  ":where(input[flush]) { border: 0; outline: 0; background: transparent; padding: 0; margin: 0; font: inherit; color: var(--foreground); }\n" +
  ":where(input[flush])::placeholder { color: var(--muted-foreground); }\n" +
  // A ul serving as a listbox is a widget, not a document list.
  ":where(ul[role='listbox']) { list-style: none; margin: 0; padding: 0; }\n" +
  // A disabled composite (a component that hosts controls) reads muted and
  // stops taking input — one rule, not a per-component reimplementation.
  ":where([data-component][disabled]) { color: var(--muted-foreground); filter: saturate(0.6); pointer-events: none; }\n" +
  // Native range/checkbox/radio take the platform's own theming hook — the
  // tone channel fills it, so a toned control paints without author CSS.
  ":where(input[type='range'], input[type='checkbox'], input[type='radio']) { accent-color: var(--tone, var(--primary)); }\n" +
  '@media (prefers-reduced-motion: reduce) { :where(dialog)::backdrop { transition: none; } }\n' +
  // Scroll-lock — pure CSS, no JS. showModal() sets the [open] attribute, so an
  // open modal anywhere freezes the page behind it.
  ':where(html):has(dialog[open]) { overflow: hidden; }\n' +
  // SHEET / DRAWER. A sheet IS a <dialog>, docked to an edge — same invoker
  // (command="show-modal"), same Escape, same backdrop, same top layer. Only the
  // geometry changes, so it's a VARIANT, not a second overlay element to learn.
  // `size` keeps its meaning: the cross-axis extent.
  ":where(dialog[side]) { inset: 0; width: auto; height: auto; max-width: none; max-height: none; border-radius: 0; }\n" +
  ":where(dialog[side='start']), :where(dialog[side='end']) { width: min(calc(100vw - var(--space-7)), var(--sheet-size, var(--panel-max-w-sm))); height: 100vh; }\n" +
  ":where(dialog[side='start']) { margin: 0 auto 0 0; border-left: 0; }\n" +
  ":where(dialog[side='end']) { margin: 0 0 0 auto; border-right: 0; }\n" +
  ":where(dialog[side='top']), :where(dialog[side='bottom']) { width: 100vw; max-height: min(calc(100vh - var(--space-7)), var(--panel-vh-md)); }\n" +
  ":where(dialog[side='top']) { margin: 0 0 auto 0; border-top: 0; }\n" +
  ":where(dialog[side='bottom']) { margin: auto 0 0 0; border-bottom: 0; }\n" +
  ":where(dialog[side][size='md']) { --sheet-size: var(--panel-max-w-md); }\n" +
  ":where(dialog[side][size='lg']) { --sheet-size: var(--panel-max-w-lg); }\n"

// L1 <table> — the STRUCTURAL primitive, zero JS. Sorting, selection and
// virtualization make a data grid; that's L2, built on this. Variants ride on the
// table itself ([dense] [striped] [hoverable] [bordered]), plus <thead sticky> and
// [numeric] columns that right-align on tabular figures. Header is de-emphasized
// and the body carries the text, which is the prior art (shadcn, Mantine) and the
// inverse of what the old prose-block table did.
const TABLE_SKIN_CSS =
  // Row rules are DATA separators, not control chrome. The borderless themes
  // (halo) set --border and --border-subtle to `transparent` on purpose — right
  // for a field, fatal for a table: with no rules you cannot track a row across
  // columns, which is a legibility floor rather than a matter of taste. So the
  // rules derive from --foreground (always contrasting, in every theme) and stay
  // themeable via --table-rule / --table-rule-subtle.
  ':where(table) { width: 100%; border-collapse: collapse; font-family: var(--font-body); font-size: var(--text-sm); line-height: var(--leading-normal); color: var(--foreground); text-align: left; --table-rule: color-mix(in srgb, var(--foreground) 16%, transparent); --table-rule-subtle: color-mix(in srgb, var(--foreground) 9%, transparent); }\n' +
  ':where(table) > caption { padding: var(--space-2) var(--space-3); color: var(--muted-foreground); text-align: left; }\n' +
  ':where(th) { padding: var(--space-2_5) var(--space-3); font-weight: var(--font-weight-semibold); color: var(--muted-foreground); border-bottom: 1px solid var(--table-rule); white-space: nowrap; }\n' +
  ':where(td) { padding: var(--space-2_5) var(--space-3); border-bottom: 1px solid var(--table-rule-subtle); }\n' +
  ':where(tbody) > tr:last-child > td { border-bottom: 0; }\n' +
  ':where(table[dense]) :is(th, td) { padding: var(--space-1_5) var(--space-2); }\n' +
  ':where(table[striped]) > tbody > tr:nth-child(even) { background: var(--surface-2); }\n' +
  // Hover is an AFFORDANCE — it says "this row does something". Opt-in, so a
  // static table doesn't lie about being interactive.
  ':where(table[hoverable]) > tbody > tr { transition: background-color var(--duration-fast) var(--ease-out); }\n' +
  ':where(table[hoverable]) > tbody > tr:hover { background: var(--surface-2); }\n' +
  ':where(table[bordered]) :is(th, td) { border: 1px solid var(--table-rule-subtle); }\n' +
  ':where(thead[sticky]) th { position: sticky; top: 0; z-index: 1; background: var(--background); }\n' +
  ':where(th[numeric]), :where(td[numeric]) { text-align: right; font-variant-numeric: tabular-nums; }\n'

// L1 [popover] — the floating surface for the menu / dropdown / tooltip family.
// A native popover in the top layer (platform owns light-dismiss + ESC); the
// `use:anchor` action positions it relative to its trigger. Surface only —
// same token vocabulary as the dialog, one notch lighter shadow.
const POPOVER_SKIN_CSS =
  ":where([popover]) { width: max-content; max-width: var(--panel-max-w-sm); padding: var(--space-1); border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--popover); color: var(--popover-foreground); box-shadow: var(--shadow-lg); font-family: var(--font-body); font-size: var(--text-sm); }\n" +
  // Tooltip variant: tighter, muted surface, no interactive affordance.
  ":where([popover][role='tooltip']) { padding: var(--space-1_5) var(--space-2_5); max-width: var(--panel-max-w-xs, 20rem); background: var(--foreground); color: var(--background); border-color: transparent; box-shadow: var(--shadow-md); }\n" +
  // Menu variant: item rows. An item <button> (or [role='menuitem']) sheds the
  // L1 button box and becomes a full-width row — hover/focus highlights, <hr>
  // divides, disabled dims. This is what turns a popover into a dropdown menu.
  ":where([popover][role='menu']) { min-width: var(--menu-min-w, 12rem); }\n" +
  ":where([popover][role='menu']) button, :where([popover][role='menu']) [role='menuitem'] { display: flex; align-items: center; justify-content: flex-start; gap: var(--space-2); width: 100%; min-height: var(--control-h-sm); padding: var(--space-1_5) var(--space-2_5); border: 0; border-radius: var(--radius-sm); background: transparent; box-shadow: none; color: inherit; font: inherit; font-weight: var(--font-weight-normal); text-align: left; white-space: nowrap; cursor: pointer; }\n" +
  ":where([popover][role='menu']) button:hover, :where([popover][role='menu']) [role='menuitem']:hover, :where([popover][role='menu']) button:focus-visible, :where([popover][role='menu']) [role='menuitem']:focus-visible { background: var(--hover-bg); outline: none; }\n" +
  ":where([popover][role='menu']) button:disabled, :where([popover][role='menu']) [role='menuitem'][aria-disabled='true'] { color: var(--muted-foreground); pointer-events: none; }\n" +
  ":where([popover][role='menu']) hr { margin: var(--space-1) calc(-1 * var(--space-1)); border: 0; border-top: 1px solid var(--border-subtle); }\n" +
  ":where([popover])::backdrop { background: transparent; }\n"

// The CONTROL half of the contracts — element-keyed channels + control
// baseline + button/input/toggle skins, WITHOUT the light host rules
// (display:block, slot, [hidden]) which are the light-DOM component's job.
// This is what shadow roots adopt (via adoptSuiSheets) so their nested
// controls are skinned; the sail-alert pilot proved the boundary blocks the
// document contracts otherwise. Exported so a unit test guards the invariant
// (adoptedStyleSheets isn't constructable under jsdom, so the effect itself
// is browser-only).
export const CONTROL_SKINS_CSS =
  hostAttrs(TONES_CSS) +
  hostAttrs(SIZES_CSS) +
  CONTROL_CONTRACTS_CSS +
  BUTTON_SKIN_CSS +
  INPUT_SKIN_CSS +
  TOGGLE_CONTROLS_CSS +
  HR_SKIN_CSS +
  STATUS_DOT_SKIN_CSS +
  SKELETON_SKIN_CSS +
  PROGRESS_SKIN_CSS +
  METER_SKIN_CSS +
  RANGE_SKIN_CSS +
  FIELDSET_SKIN_CSS +
  DROPZONE_SKIN_CSS +
  DETAILS_SKIN_CSS

export const LIGHT_CONTRACTS_CSS =
  BOX_MODEL_CSS +
  // Root typography anchor for light-DOM pages: bare text inherits the body
  // family and ink from here. The type scale sizes <p>/<h1> but never declares
  // a family, so without this anchor the page root falls to the UA serif. Shadow
  // components inherit the family across their boundary and own their ink via
  // [data-component]. Specificity 0 — an author :root/body rule wins.
  ':where(html) { font-family: var(--font-body); color: var(--foreground); line-height: var(--leading-normal); }\n' +
  hostAttrs(TONES_CSS) +
  hostAttrs(SIZES_CSS) +
  // Semantic-element + typography defaults reach light-DOM pages here (they
  // were shadow-only before) — a bare <table>/<ul>/<img>/<h1> on a page is
  // styled by the framework, no .ship-view wrapper needed.
  TYPOGRAPHY_CSS +
  SEMANTIC_ELEMENTS_CSS +
  CONTROL_CONTRACTS_CSS +
  BUTTON_SKIN_CSS +
  INPUT_SKIN_CSS +
  TOGGLE_CONTROLS_CSS +
  HR_SKIN_CSS +
  STATUS_DOT_SKIN_CSS +
  SKELETON_SKIN_CSS +
  PROGRESS_SKIN_CSS +
  METER_SKIN_CSS +
  RANGE_SKIN_CSS +
  FIELDSET_SKIN_CSS +
  DROPZONE_SKIN_CSS +
  DETAILS_SKIN_CSS +
  DIALOG_SKIN_CSS +
  TABLE_SKIN_CSS +
  POPOVER_SKIN_CSS +
  // Host defaults for light components (the :host base rules, element-keyed).
  '[data-component] { display: block; color: var(--foreground); }\n' +
  '[data-component][hidden] { display: none !important; }\n' +
  // <slot> stays in the light tree as the projection wrapper — transparent.
  '[data-component] slot { display: contents; }\n' +
  // Reduced motion kills transitions/animation for component subtrees AND the
  // globally-styled controls (their lift/press/transition are document-wide now).
  // transition stays a blanket !important kill (interaction micro-transitions
  // carry no component-authored intent to lose). animation is NOT !important
  // and the whole selector list sits inside :where() (zero specificity), so a
  // component that owns its own reduced-motion behavior (e.g. a marquee that
  // pauses instead of vanishing) has real specificity in its own skin and
  // wins the cascade unforced — this rule is only the floor default for
  // anything that has not opted in.
  '@media (prefers-reduced-motion: reduce) { :where([data-component], [data-component] *, button, input, textarea, select, a, [tabindex]) { transition: none !important; animation: none; } }\n'

// `ensureLightContracts()` used to live here and inject the contracts from a string
// constant at every component mount — which meant no bundler could drop the 30KB,
// because the code demonstrably read it. It moved behind a seam
// (./contracts-seam) and the injection itself is opt-in (@ship/sail-runtime/contracts).
// Nothing in the default path references LIGHT_CONTRACTS_CSS now, so it shakes out.
