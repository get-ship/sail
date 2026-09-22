//! Pure ambient-authority classification for `sail.config.json` `"network"` /
//! `"unconfined"` — the SAME fail-closed rules as
//! `sail_compiler::check::policy::classify` ambient arms
//! (`Policy::network_for` / `Policy::unconfined_for`).
//!
//! No filesystem. Walk-up resolution lives in `./ambient-resolve` (Node). The
//! SES runner (`./route-runner`) consumes a pre-resolved {@link AmbientAuthority}
//! so the browser surface never pulls `node:fs`.
//!
//! Knobs are unsafe-shaped: a grant voids the default deny at both the compile
//! lint and the runtime endowment gates.

/** Filename the walk-up stops on — must stay byte-identical to
 *  `sail_lint::config::CONFIG_FILE`. */
export const SAIL_CONFIG_FILE = 'sail.config.json'

/** Keys the compiler's config parser recognizes. Unknown top-level keys make
 *  the Rust classifier Reject (fail closed). Ambient fields fail closed here
 *  the same way even though we don't re-implement the styling-tier classifier. */
const RECOGNIZED_KEYS: Record<string, true> = {
  preset: true,
  sx: true,
  variants: true,
  raw_containers: true,
  stylesheets: true,
  network: true,
  unconfined: true,
}

/**
 * The ambient-authority half of a package's resolved `sail.config.json` —
 * what the SES host needs to endow (or refuse). Orthogonal to styling tier.
 */
export interface AmbientAuthority {
  /** Per-origin fetch/WebSocket/EventSource allowlist. Empty = no grant. */
  network: string[]
  /** Whole-hog opt-out of SES confinement. Default false. */
  unconfined: boolean
  /** Absolute path of the config that produced this resolution, if any. */
  configPath: string | null
}

/** Deny-by-default ambient authority — no config, Rejected, or absent keys. */
export const AMBIENT_DENY_ALL: AmbientAuthority = {
  network: [],
  unconfined: false,
  configPath: null,
}

/** Pure classification of a parsed config body → ambient fields.
 *  Mirrors `classify` ambient arms in `policy.rs`: wrong-typed network /
 *  unconfined, unknown keys, or non-object → fail closed. */
export function classifyAmbient(raw: unknown, configPath: string | null = null): AmbientAuthority {
  if (raw === null || typeof raw !== 'object' || Array.isArray(raw)) {
    return { network: [], unconfined: false, configPath }
  }

  for (const key of Object.keys(raw)) {
    if (!RECOGNIZED_KEYS[key]) {
      return { network: [], unconfined: false, configPath }
    }
  }

  const obj = raw as Record<string, unknown>

  let network: string[] = []
  if ('network' in obj) {
    const net = obj.network
    if (!Array.isArray(net) || !net.every((x) => typeof x === 'string')) {
      return { network: [], unconfined: false, configPath }
    }
    network = net.slice() as string[]
  }

  let unconfined = false
  if ('unconfined' in obj) {
    const uc = obj.unconfined
    if (typeof uc !== 'boolean') {
      return { network: [], unconfined: false, configPath }
    }
    unconfined = uc
  }

  return { network, unconfined, configPath }
}

/** Classify JSON text. Unparseable → fail closed (Denied). */
export function classifyAmbientJson(
  text: string,
  configPath: string | null = null,
): AmbientAuthority {
  try {
    return classifyAmbient(JSON.parse(text) as unknown, configPath)
  } catch {
    return { network: [], unconfined: false, configPath }
  }
}
