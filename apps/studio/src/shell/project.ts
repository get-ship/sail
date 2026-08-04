/** Project root resolution + seed paths for Studio explorer. */
import { hostSailProject } from '@ship/sail-studio-shell'

export function projectPaths(proj: string): string[] {
  const root = proj.replace(/\/$/, '')
  // Seed includes non-.sail so the explorer shows a full-project shape before ListFiles.
  const paths = [`${root}/routes/page.sail`, `${root}/package.json`]
  if (root === 'apps/studio' || root.endsWith('/apps/studio')) {
    paths.push(
      'apps/studio/test/fixtures/sample.sail',
      'apps/studio/test/fixtures/tier-s.sail',
      'apps/studio/test/fixtures/tier-i.sail',
      'apps/studio/test/fixtures/tier-c.sail',
      'apps/studio/src/edit-explorer/edit-explorer-vm.ts',
    )
  }
  return paths
}

/**
 * Ops target (workspace-relative).
 * `?project=apps/…` → host sail-project meta → apps/studio dogfood.
 * Meta is host-stamped; apps read via studio-shell, not document.querySelector.
 */
export function resolveProjectRoot(): string {
  // DOMDEC: no ambient location.search — host meta only.
  const meta = hostSailProject()
  if (meta) return meta
  return 'apps/studio'
}
