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
  const sheets = [suiBaseSheet, suiTonesSheet].filter(
    (s): s is CSSStyleSheet => s != null,
  )
  if (sheets.length === 0) return
  try {
    root.adoptedStyleSheets = [...sheets, ...(root.adoptedStyleSheets ?? [])]
  } catch {
    /* unsupported: silently fall back to inline <style> blocks */
  }
}
