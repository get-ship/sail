// Compile-error formatting — the folded `sui-compile-error` (culled to a
// helper, build-spec §Error surface). Pure presentation over a structured
// diagnostic: the offending source line + a caret at the column, plus the
// raw message. No state, no events, no DOM construction here — the engine's
// template branch renders the returned strings via `sui-code` + `sui-alert`.

/** A structured compile diagnostic — the `{kind:'err'}` shape the injected
 *  `compile()` returns (no exception across the `prop:compile` seam). */
export interface CompileDiagnostic {
  message: string
  line: number | null
  col: number | null
  category: string | null
}

export interface FormattedCompileError {
  /** The full diagnostic message, untruncated (copyable for self-correction). */
  message: string
  /** `line:col` location, or `''` when the diagnostic carries no position. */
  location: string
  category: string
  /** The offending source line, or `''` when the line can't be resolved. */
  sourceLine: string
  /** A caret-on-its-own-line frame: the source line above a `^` at `col`. */
  caretFrame: string
}

/** A stable identity for a diagnostic, used to dedup `sui-render:error`
 *  (the `lastEmitted` guard) so an unchanged error isn't emitted twice. */
export function errorKey(d: CompileDiagnostic): string {
  return `${d.category ?? ''}|${d.line ?? ''}:${d.col ?? ''}|${d.message}`
}

/**
 * Build the caret/gutter view of a compile diagnostic against the raw source
 * the engine already owns. Resolves the offending 1-based `line`, then frames
 * it above a caret positioned at the 1-based `col`.
 */
export function formatCompileError(source: string, d: CompileDiagnostic): FormattedCompileError {
  const location = d.line != null ? `${d.line}:${d.col ?? 0}` : ''
  const lines = source.split('\n')
  const idx = d.line != null ? d.line - 1 : -1
  const sourceLine = idx >= 0 && idx < lines.length ? (lines[idx] ?? '') : ''

  let caretFrame = sourceLine
  if (sourceLine && d.col != null && d.col >= 1) {
    // A caret under the column. Tabs in the source render as one column each
    // for the purpose of the caret offset (the common, legible default).
    const pad = ' '.repeat(Math.max(0, d.col - 1))
    caretFrame = `${sourceLine}\n${pad}^`
  }

  return {
    message: d.message,
    location,
    category: d.category ?? 'compile',
    sourceLine,
    caretFrame,
  }
}
