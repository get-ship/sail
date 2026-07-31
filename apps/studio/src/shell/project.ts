/** Project root resolution + seed paths for Studio explorer. */

export function projectPaths(proj: string): string[] {
  const root = proj.replace(/\/$/, '')
  const paths = [`${root}/routes/page.sail`]
  if (root === 'apps/studio' || root.endsWith('/apps/studio')) {
    paths.push(
      'apps/studio/test/fixtures/sample.sail',
      'apps/studio/test/fixtures/tier-s.sail',
      'apps/studio/test/fixtures/tier-i.sail',
      'apps/studio/test/fixtures/tier-c.sail',
    )
  }
  return paths
}

/**
 * Ops target (workspace-relative).
 * `?project=apps/…` → sail-project meta → apps/studio dogfood.
 */
export function resolveProjectRoot(): string {
  if (typeof document === 'undefined') return 'apps/studio'
  try {
    const q = new URLSearchParams(location.search).get('project')?.trim()
    if (q && !q.includes('..') && (q.startsWith('apps/') || q.startsWith('/apps/'))) {
      return q.replace(/^\//, '').replace(/\/$/, '')
    }
  } catch {
    /* ignore */
  }
  const meta =
    document.querySelector('meta[name="sail-project"]')?.getAttribute('content')?.trim() ||
    ''
  if (meta) return meta.replace(/\/$/, '')
  return 'apps/studio'
}
