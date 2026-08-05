// Takumi static snapshot helper — RC scaffold only.
//
// Full path is crates/sail/render (SSR project → layout → takumi PNG). Wiring
// that into a zero-deps JS import needs a native/NAPI or CLI bridge that does
// not exist as a stable public package yet. Do not pretend browser screenshots
// are the same contract.

export type SnapshotResult = {
  /** PNG bytes when a renderer is available. */
  png: Uint8Array | null
  /** Honest status for agents / CI. */
  status: 'unavailable' | 'ok'
  reason?: string
}

/**
 * Snapshot a static `.sail` render via Takumi when a host bridge is provided.
 * Without a bridge this returns `unavailable` — call sites must not treat
 * absence as a green visual assertion.
 */
export async function snapshotSail(
  _source: string,
  opts?: {
    /** Optional host: path to a prebuilt PNG, or async renderer. */
    render?: (source: string) => Promise<Uint8Array>
  },
): Promise<SnapshotResult> {
  if (opts?.render) {
    const png = await opts.render(_source)
    return { png, status: 'ok' }
  }
  return {
    png: null,
    status: 'unavailable',
    reason:
      'Takumi snapshot bridge not wired in @ship/sail-test RC slice. Use crates/sail/render or a host-provided render() callback. Do not substitute browser screenshots as the static contract.',
  }
}
