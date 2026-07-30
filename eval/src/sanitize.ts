/**
 * Sanitize trajectories / artifacts before any commit-facing write.
 * Strips absolute machine paths so RESULTS.md and promoted fixtures stay clean.
 * URL scheme://… strings are preserved (not treated as filesystem paths).
 */

/**
 * Absolute filesystem paths (POSIX multi-segment or Windows drive).
 * Broader than home/tmp only — covers /workspaces, /root, /mnt, /data, /nix, etc.
 * Requires at least two path segments (`/a/b`).
 * Applied only after URL protection so path segments of https://… are not mangled.
 */
const ABS_PATH =
  /(?:\/(?:[\w.+@-]+\/)+[\w.+@/-]+)|(?:[A-Za-z]:\\(?:[^\s"'`\\]+\\)*[^\s"'`]+)/g

/** scheme://host/… — protected before ABS_PATH so /api/v1 inside URLs survives. */
const URL_RE = /[a-zA-Z][a-zA-Z0-9+.-]*:\/\/[^\s"'`<>]+/g

/**
 * Internal token so workdir-relative tails are not re-matched as absolute paths.
 * Uses `|` as separator (not `/`) so ABS_PATH cannot swallow the relative tail.
 */
const WORKDIR_TOKEN = '@@WORKDIR@@'
const URL_TOKEN = (i: number) => `@@URL${i}@@`

/** Replace absolute filesystem paths with a stable placeholder. */
export function sanitizePath(p: string, workdir?: string): string {
  if (workdir && (p === workdir || p.startsWith(workdir + '/') || p.startsWith(workdir + '\\'))) {
    const rel = p.slice(workdir.length).replace(/^[/\\]+/, '')
    return rel || '.'
  }
  // Bare path (not free text): no URL protect needed for pure paths.
  return p.replace(ABS_PATH, '<path>')
}

export function sanitizeText(text: string, workdir?: string): string {
  // 1. Shield URLs so multi-segment path redaction cannot eat /api/v1 etc.
  const urls: string[] = []
  let out = text.replace(URL_RE, (m) => {
    const i = urls.length
    urls.push(m)
    return URL_TOKEN(i)
  })

  // 2. Workdir (+ optional suffix) → protected token
  if (workdir) {
    const escaped = workdir.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    out = out.replace(
      new RegExp(escaped + '(?:[/\\\\][^\\s"\'`]*)?', 'g'),
      (match) => {
        const rest = match.slice(workdir.length).replace(/^[/\\]+/, '').replace(/\\/g, '/')
        return rest ? `${WORKDIR_TOKEN}|${rest}` : WORKDIR_TOKEN
      },
    )
  }

  // 3. Redact remaining absolute FS paths
  out = out.replace(ABS_PATH, '<path>')

  // 4. Restore workdir placeholders
  out = out.replace(
    new RegExp(WORKDIR_TOKEN + '(?:\\|([^\\s"\']*))?', 'g'),
    (_m, rest: string | undefined) => (rest ? `<workdir>/${rest}` : '<workdir>'),
  )

  // 5. Restore URLs
  out = out.replace(/@@URL(\d+)@@/g, (_m, idx: string) => urls[Number(idx)] ?? '')
  return out
}

export function sanitizeJson<T>(value: T, workdir?: string): T {
  return JSON.parse(sanitizeText(JSON.stringify(value), workdir)) as T
}

/** Shared note sanitizer for RESULTS.md table cells. */
export function sanitizeNote(s: string, maxLen = 120): string {
  return sanitizeText(s).replace(/\|/g, '/').slice(0, maxLen)
}
