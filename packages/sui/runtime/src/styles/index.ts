const TONES_CSS =
  ":host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }\n" +
  ":host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }\n" +
  ":host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }\n" +
  ":host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }\n" +
  ":host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }\n" +
  ":host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }\n" +
  ":host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }\n" +
  ":host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n"

// R3: shared host-resets adopted into every shadow component. Rules
// here are duplicated literally in nearly every <style> block; hoisting
// them lets the sui-compiler strip the per-component copies (see
// `crates/sui-compiler/src/emit/styles.rs`). Keep this list strictly to
// rules whose author-form text the compiler can recognize and remove.
const BASE_CSS =
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
  const sheets = [suiBaseSheet, suiTonesSheet, suiTypographySheet].filter(
    (s): s is CSSStyleSheet => s != null,
  )
  if (sheets.length === 0) return
  try {
    root.adoptedStyleSheets = [...sheets, ...(root.adoptedStyleSheets ?? [])]
  } catch {
    /* unsupported: silently fall back to inline <style> blocks */
  }
}
