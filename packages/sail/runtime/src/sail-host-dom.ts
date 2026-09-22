/**
 * Framework-only SailHost ↔ DOM bridge.
 *
 * Not part of the Pure-portable surface: names DOM lib types. Import from
 * `@ship/sail-runtime/internal` (or this module) — never from product Pure code.
 */
import type { SailHost } from './sail-host'

/** View SailHost as the underlying web element (identity on web). */
export function unwrapSailHost(host: SailHost): HTMLElement {
  return host as unknown as HTMLElement
}
