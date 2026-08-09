// demo-clock-api — teaching fixture for /docs/app/reactivity's live-data snippet.
// Lives under src/compositions so controllers may value-import it
// (sail-controller-import-convention). Mirrors the shape of a host-injected
// `api.demo.clock()` client without the bare `ship` package call surface.

export const api = {
  demo: {
    // Promise form is enough for the docs compile gate + query() keying;
    // a real host client may return LiveSource for SSE push.
    clock: (_args: Record<string, unknown> = {}): Promise<{ now: string }> =>
      Promise.resolve({ now: new Date().toISOString() }),
  },
}
