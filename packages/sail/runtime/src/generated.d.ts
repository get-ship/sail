// Committed fallback declarations for `./generated`.
//
// The real `./generated.ts` is an auto-generated artifact (from the Rust wire
// types; no committed gen-types xtask — regenerate via the runtime package
// typecheck path when present) and is gitignored (`**/src/generated.ts`).
// When that file is present it wins module
// resolution over this `.d.ts` (`.ts` is resolved before `.d.ts`), so the
// deck build is never affected by this stub. When it is ABSENT — a fresh
// checkout, CI typechecking sail standalone, or a consumer pointing
// `@ship/*` at sail source instead of hand-mirrored shims — these
// declarations let the runtime typecheck without the codegen step.
//
// Keep in sync with the generated output. This is a faithful mirror of the
// types the sail runtime imports plus their transitive dependencies; if the
// Rust wire types grow a new field the runtime depends on, update here too
// (the per-package `typecheck` script will flag a missing/incompatible
// type as a hard error, so drift cannot pass silently).

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | Partial<{ [key in string]: JsonValue }>

/**
 * Route identity for the shell rail / launcher.
 */
export type ManifestRail = { label: string | null; icon: string | null }

/**
 * Per-route shell display metadata. Maps to `<ship-rail>` / `<ship-launcher>`.
 */
export type RailMeta = { label: string; icon: string }

export type ResolveError = { message: string }

export type ResolvedRoute = {
  source: string
  params: Partial<{ [key in string]: string }>
  source_path: string | null
}

export type RouteListEntry = {
  name: string
  type: string
  source: string
  rail?: RailMeta | null
}

export type RouteManifest = {
  name: string
  type: string
  elementName?: string | null
  bundleUrl?: string | null
  claims?: string[] | null
  entry?: string | null
}

export type RoutePayload = {
  name: string
  sub?: string | null
  html: string
  css: string
  scope: JsonValue
  moduleUrl: string
  params: Partial<{ [key in string]: string }>
  clients: string[]
}
