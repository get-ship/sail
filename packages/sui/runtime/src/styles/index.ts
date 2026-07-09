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
  // Tables.
  'table { border-collapse: collapse; width: 100%; font-size: var(--text-sm); }\n' +
  'th, td { padding: var(--space-2) var(--space-3); text-align: left; border-bottom: 1px solid var(--border); }\n' +
  'th { font-weight: var(--font-weight-semibold, 600); color: var(--foreground); }\n' +
  'td { color: var(--muted-foreground); }\n' +
  'caption { padding: var(--space-2); font-size: var(--text-sm); color: var(--muted-foreground); text-align: left; }\n' +
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
  ":where(span[dot][pulse]) { animation: sui-status-dot-pulse 2.5s var(--ease-out) infinite; }\n" +
  '@keyframes sui-status-dot-pulse { 0%, 100% { opacity: 1; transform: scale(1); } 50% { opacity: 0.82; transform: scale(0.92); } }\n'

// L1 <span skeleton> — the shimmer-placeholder surface. `skeleton` scopes
// the skin the same way `dot` does above. shape/size drive width/height/
// radius; the shimmer gradient/keyframe and the circle-lg 48px are
// off-vocabulary for sx (no `bg:linear-gradient()`, no animation primitive,
// no 48px token — see the former wrapper's own LANG-GAP note), so they land
// here instead.
const SKELETON_SKIN_CSS =
  ":where(span[skeleton]) { display: inline-block; vertical-align: middle; background: linear-gradient(90deg, var(--muted) 0%, var(--tone-muted-bg) 50%, var(--muted) 100%); background-size: 200% 100%; animation: sui-skeleton-shimmer 1.5s ease-in-out infinite; border-radius: var(--radius-sm); }\n" +
  ":where(span[skeleton][shape='block']) { display: block; width: 100%; min-height: 80px; }\n" +
  ":where(span[skeleton][shape='circle']) { border-radius: var(--radius-full); }\n" +
  ":where(span[skeleton][shape='text'], span[skeleton][shape='line']) { width: 100%; }\n" +
  ":where(span[skeleton][shape='text'][size='sm'], span[skeleton][shape='line'][size='sm']) { height: var(--space-2); }\n" +
  ":where(span[skeleton][shape='text'][size='md'], span[skeleton][shape='line'][size='md']) { height: var(--space-3); }\n" +
  ":where(span[skeleton][shape='text'][size='lg'], span[skeleton][shape='line'][size='lg']) { height: var(--space-4); }\n" +
  ":where(span[skeleton][shape='circle'][size='sm']) { width: var(--icon-size-2xl); height: var(--icon-size-2xl); }\n" +
  ":where(span[skeleton][shape='circle'][size='md']) { width: var(--space-7); height: var(--space-7); }\n" +
  ":where(span[skeleton][shape='circle'][size='lg']) { width: 48px; height: 48px; }\n" +
  '@keyframes sui-skeleton-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }\n'

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
  ":where(progress:not([value]))::-webkit-progress-bar { background-image: linear-gradient(90deg, transparent 0%, var(--tone-fg, var(--primary)) 50%, transparent 100%); background-repeat: no-repeat; background-size: 40% 100%; animation: sui-progress-indeterminate 1.2s ease-in-out infinite; }\n" +
  ":where(progress:not([value])) { background-image: linear-gradient(90deg, transparent 0%, var(--tone-fg, var(--primary)) 50%, transparent 100%); background-repeat: no-repeat; background-size: 40% 100%; animation: sui-progress-indeterminate 1.2s ease-in-out infinite; }\n" +
  "@keyframes sui-progress-indeterminate { 0% { background-position: -40% 0; } 100% { background-position: 140% 0; } }\n" +
  "@media (prefers-reduced-motion: reduce) { :where(progress), :where(progress)::-webkit-progress-value { animation: none !important; transition: none !important; } }\n"

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
  '@media (prefers-reduced-motion: reduce) { :where(dialog)::backdrop { transition: none; } }\n' +
  // Scroll-lock — pure CSS, no JS. showModal() sets the [open] attribute, so an
  // open modal anywhere freezes the page behind it.
  ':where(html):has(dialog[open]) { overflow: hidden; }\n'

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
  TOGGLE_CONTROLS_CSS +
  HR_SKIN_CSS +
  STATUS_DOT_SKIN_CSS +
  SKELETON_SKIN_CSS +
  PROGRESS_SKIN_CSS

export const LIGHT_CONTRACTS_CSS =
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
  DIALOG_SKIN_CSS +
  POPOVER_SKIN_CSS +
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
