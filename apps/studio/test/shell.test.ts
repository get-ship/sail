import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  activityFromPath,
  appRouteBase,
  hrefForActivity,
  pathForActivity,
  railRowsForPath,
} from '../src/shell/shell-page-vm'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const CHROME = join(REPO, 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail')
const SHELL_VM = join(REPO, 'apps/studio/src/shell/shell-page-vm.ts')
const ROUTES_GEN = join(REPO, 'apps/studio/routes.gen.ts')

describe('Studio routes + chrome (STUDIO-1)', () => {
  const chrome = readFileSync(CHROME, 'utf8')
  const shellVm = readFileSync(SHELL_VM, 'utf8')
  const routesGen = readFileSync(ROUTES_GEN, 'utf8')
  const layout = readFileSync(join(REPO, 'apps/studio/routes/_layout.sail'), 'utf8')

  it('routes.gen exposes leaf surfaces + layout chrome', () => {
    expect(routesGen).toMatch(/pattern: '\/edit'/)
    expect(routesGen).toMatch(/pattern: '\/checks'/)
    expect(routesGen).toMatch(/pattern: '\/design'/)
    expect(routesGen).toMatch(/pattern: '\/tasks'/)
    expect(routesGen).toMatch(/tag: 'sail-studio-chrome'/)
    expect(routesGen).toMatch(/layoutModules/)
    expect(layout).toMatch(/<sail-studio-chrome/)
    expect(existsSync(join(REPO, 'apps/studio/routes/edit/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/checks/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/design/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/tasks/page.sail'))).toBe(true)
  })

  it('leaf routes mount surfaces; no activity-signal switching in chrome', () => {
    for (const [leaf, tag] of [
      ['edit', 'edit-surface'],
      ['checks', 'checks-surface'],
      ['design', 'design-surface'],
      ['tasks', 'tasks-surface'],
    ] as const) {
      const page = readFileSync(join(REPO, `apps/studio/routes/${leaf}/page.sail`), 'utf8')
      expect(page).toMatch(new RegExp(`<${tag}\\b`))
    }
    // Signal switching deleted from shell
    expect(shellVm).not.toMatch(/setActivity/)
    expect(shellVm).not.toMatch(/isEdit|isChecks|isDesign|isTasks/)
    expect(shellVm).not.toMatch(/onRailClick/)
    expect(chrome).not.toMatch(/onRailClick|on:click=\{onRailClick\}/)
    // Rail = real links
    expect(chrome).toMatch(/href=\{item\.href\}/)
    expect(chrome).toMatch(/data-studio-rail/)
    expect(chrome).toMatch(/<tooltip\b/)
    expect(shellVm).toMatch(/hrefForActivity|activityFromPath/)
  })

  it('activityFromPath + hrefForActivity cover all surfaces', () => {
    expect(activityFromPath('/edit')).toBe('edit')
    expect(activityFromPath('/')).toBe('edit')
    expect(activityFromPath('/checks')).toBe('checks')
    expect(activityFromPath('/design')).toBe('design')
    expect(activityFromPath('/tasks')).toBe('tasks')
    expect(hrefForActivity('edit')).toMatch(/\/edit$/)
    expect(hrefForActivity('checks')).toMatch(/\/checks$/)
    expect(pathForActivity('/edit')).toBe('/edit')
    // appRouteBase is empty outside browser meta (overlay sets deck-app-route)
    expect(appRouteBase()).toBe('')
  })

  it('rail active flips when path changes (SPA leaf navigation model)', () => {
    // Pure rows: each path marks exactly one activity active
    const edit = railRowsForPath('/edit')
    expect(edit.find((r) => r.id === 'edit')?.active).toBe(true)
    expect(edit.filter((r) => r.active)).toHaveLength(1)

    const checks = railRowsForPath('/checks')
    expect(checks.find((r) => r.id === 'checks')?.active).toBe(true)
    expect(checks.find((r) => r.id === 'edit')?.active).toBe(false)

    const design = railRowsForPath('/design')
    expect(design.find((r) => r.id === 'design')?.active).toBe(true)
    expect(design.find((r) => r.id === 'checks')?.active).toBe(false)

    const tasks = railRowsForPath('/tasks')
    expect(tasks.find((r) => r.id === 'tasks')?.active).toBe(true)
    expect(tasks.every((r) => r.href.includes(r.id === 'edit' ? '/edit' : `/${r.id}`))).toBe(
      true,
    )

    // Chrome + shell wire live route.pathname + effect re-sync (not popstate-only)
    expect(shellVm).toMatch(/pathname\?:/)
    expect(shellVm).toMatch(/effect\s*\(/)
    expect(shellVm).toMatch(/syncRail/)
    expect(chrome).toMatch(/createShellPage\(\{\s*pathname:\s*routePath/)
    expect(chrome).toMatch(/route\?\.pathname|r\.pathname/)
  })

  it('batch new task uses deck navigate, not raw pushState', () => {
    expect(shellVm).toMatch(/from ['"]@ship\/deck\/router['"]/)
    expect(shellVm).toMatch(/navigate\(hrefForActivity\('tasks'\)\)/)
    expect(shellVm).not.toMatch(/history\.pushState/)
    expect(shellVm).not.toMatch(/PopStateEvent/)
  })

  it('chrome has activity bar + status; no shell doc-tabs, viewport, esc badge', () => {
    expect(chrome).toMatch(/data-sail-studio-shell/)
    expect(chrome).toMatch(/data-studio-rail/)
    expect(chrome).toMatch(/data-studio-status/)
    expect(chrome).toMatch(/slot name="route"/)
    // Explicit removals
    expect(chrome).not.toMatch(/toggle-group|onViewport|viewport/)
    expect(chrome).not.toMatch(/data-hotkey-hints|>esc</)
    expect(chrome).not.toMatch(/data-studio-tabs/)
    expect(chrome).not.toMatch(/selectionLabel/)
    expect(chrome).not.toMatch(/data-badge|hasBadge/)
    expect(shellVm).not.toMatch(/setViewport|onViewport/)
    expect(shellVm).not.toMatch(/ViewportId/)
    // Health tones bound (soft badge tone from ops/watch health signals)
    expect(chrome).toMatch(/tone=\{opsHealthTone\(\)\}/)
    expect(chrome).toMatch(/tone=\{watchHealthTone\(\)\}/)
  })

  it('status bar is slim with health segments', () => {
    expect(chrome).toMatch(/data-studio-status/)
    expect(chrome).toMatch(/data-health="ops"/)
    expect(chrome).toMatch(/data-health="watch"/)
    expect(chrome).toMatch(/data-studio-status[\s\S]*\bnowrap\b/)
    expect(chrome).toMatch(/data-studio-status[\s\S]*minW\(0\)|data-studio-status[\s\S]*min-w="0"/)
    expect(chrome).toMatch(/data-studio-status[\s\S]*truncate\(\)/)
    expect(chrome).toMatch(/data-studio-status[\s\S]*h\(6\)/)
  })

  it('rail uses VSCode left-accent active pattern (links, not variant flip)', () => {
    expect(chrome).toMatch(/aria-current=\{item\.active/)
    expect(chrome).toMatch(/data-active=\{item\.active/)
    expect(chrome).toMatch(/borderLWidth\(2\)|borderL\(/)
    expect(chrome).not.toMatch(/variant=\{item\.active \? 'solid'/)
  })

  it('activity rail has Exit / Back to app (never a dead-end)', () => {
    expect(chrome).toMatch(/data-exit-studio/)
    expect(chrome).toMatch(/Back to app/)
    expect(chrome).toMatch(/onExitStudio|on:click=\{onExitStudio\}/)
    const exitVm = readFileSync(join(REPO, 'apps/studio/src/shell/exit.ts'), 'utf8')
    expect(exitVm).toMatch(/export function exitStudio/)
    expect(exitVm).toMatch(/type: 'close'/)
    expect(exitVm).toMatch(/location\.assign\('\/'\)/)
    expect(shellVm).toMatch(/Escape/)
    expect(shellVm).toMatch(/exitStudio/)
  })

  it('page + shell + edit files stay ≤400 LOC', () => {
    const files = [
      'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail',
      'apps/studio/src/shell/shell-page-vm.ts',
      'apps/studio/src/edit-surface/edit-surface.sail',
      'apps/studio/src/edit-surface/edit-surface-vm.ts',
      'apps/studio/src/edit-explorer/edit-explorer.sail',
      'apps/studio/src/edit-explorer/edit-explorer-vm.ts',
      'apps/studio/src/edit-element-tree/edit-element-tree.sail',
      'apps/studio/src/edit-element-tree/edit-element-tree-vm.ts',
    ]
    for (const rel of files) {
      const src = readFileSync(join(REPO, rel), 'utf8')
      expect(src.split('\n').length, rel).toBeLessThanOrEqual(400)
    }
  })

  it('edit-surface owns editor tabs; empty guidance; no viewport picker', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface.sail'),
      'utf8',
    )
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-vm.ts'),
      'utf8',
    )
    expect(surface).toMatch(/data-edit-surface/)
    expect(surface).toMatch(/data-surface="edit"/)
    expect(surface).toMatch(/data-studio-tabs|data-edit-tabs/)
    expect(surface).toMatch(/data-tab-id/)
    expect(surface).toMatch(/data-tab-close|showClose/)
    expect(surface).toMatch(/Open a file from the Explorer/)
    expect(surface).toMatch(/Inspector/)
    expect(surfaceVm).toMatch(/showClose/)
    expect(surface).toMatch(/data-view-panel="preview"/)
    expect(surface).toMatch(/data-view-panel="source"/)
    expect(surface).toMatch(/data-live-preview/)
    expect(surface).toMatch(/data-audit-snapshot/)
    expect(surface).toMatch(/data-source-editor/)
    expect(surface).not.toMatch(/toggle-group|onViewport|viewport picker/i)
    expect(surfaceVm).toMatch(/closeDocTab|openShellTabDocument/)
    expect(surfaceVm).toMatch(/showDebug = signal\(false\)/)
    expect(surfaceVm).toMatch(/showAudit = signal\(false\)/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
  })

  it('edit-explorer panel grammar + empty state', () => {
    const explorer = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer.sail'),
      'utf8',
    )
    const tree = readFileSync(
      join(REPO, 'apps/studio/src/edit-element-tree/edit-element-tree.sail'),
      'utf8',
    )
    expect(explorer).toMatch(/data-edit-explorer/)
    expect(explorer).toMatch(/data-file-tree/)
    expect(explorer).toMatch(/Explorer/)
    expect(explorer).toMatch(/textCase\(uppercase\)/)
    expect(explorer).toMatch(/<collapsible\b/)
    expect(explorer).toMatch(/role="tree"/)
    expect(explorer).toMatch(/<edit-element-tree\b/)
    expect(explorer).toMatch(/No files/)
    expect(tree).toMatch(/data-element-tree/)
    expect(tree).toMatch(/data-depth/)
    expect(tree).toMatch(/layerBadge|data-layer-badge/)
    expect(explorer).not.toMatch(/<div\b/)
    expect(tree).not.toMatch(/<div\b/)
  })

  it('edit surface VMs do not dual-load ops/openDocument on mount', () => {
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-vm.ts'),
      'utf8',
    )
    const treeVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-element-tree/edit-element-tree-vm.ts'),
      'utf8',
    )
    const explorerVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer-vm.ts'),
      'utf8',
    )
    for (const vm of [surfaceVm, treeVm]) {
      expect(vm).not.toMatch(/createOpsClient/)
      expect(vm).not.toMatch(/setOpsClient/)
    }
    // surface openDocument only via tab open helper
    expect(explorerVm).not.toMatch(/createOpsClient/)
    expect(explorerVm).not.toMatch(/setOpsClient/)
    expect(explorerVm).toMatch(/openDocument\s*\(/)
    const mountBody =
      explorerVm.match(/mountAmbient\s*\(\s*\(\)\s*=>\s*\{([\s\S]*?)\n  \}\)/)?.[1] ?? ''
    expect(mountBody.length).toBeGreaterThan(0)
    expect(mountBody).not.toMatch(/openDocument/)
  })

  it('chrome boots ops + seed; surfaces not dual-mounted in chrome', () => {
    expect(chrome).not.toMatch(/<edit-surface\b/)
    expect(chrome).not.toMatch(/<checks-surface\b/)
    expect(chrome).not.toMatch(/<design-surface\b/)
    expect(chrome).not.toMatch(/<tasks-surface\b/)
    expect(shellVm).toMatch(/from ['"]@ship\/studio\/edit['"]/)
    expect(shellVm).toMatch(/openDocument|createOpsClient|setOpsClient/)
    expect(shellVm).toMatch(/seedFilesFromPaths|bindEditSelection/)
    // Live preview on for non-studio targets; dogfood skips nested iframe.
    expect(shellVm).toMatch(/loadPreview/)
    expect(shellVm).not.toMatch(/refreshPreview/)
    expect(shellVm).toMatch(/loadFindings|setChecksOpsClient/)
    expect(shellVm).toMatch(/loadThemes|loadIconGlyphs/)
    expect(shellVm).toMatch(/loadThreads|newTaskFromBatch/)
  })

  // STUDIO-2 R1/R4/R5/B1 + batched sail check → test/studio-2-repairs.test.ts

  it('checks-surface ban-clean + workbench markers', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/checks-surface/checks-surface.sail'),
      'utf8',
    )
    expect(surface).toMatch(/data-checks-surface/)
    expect(surface).toMatch(/data-surface="checks"/)
    expect(surface).toMatch(/data-findings-list/)
    expect(surface).toMatch(/data-severity-group/)
    expect(surface).toMatch(/data-review-fixes/)
    expect(surface).toMatch(/data-review-dialog/)
    expect(surface).toMatch(/Review fixes/)
    expect(surface).toMatch(/applyFixesLabel|Apply/)
    expect(surface).toMatch(/data-both-viewports|showBothViewports/)
    expect(surface).toMatch(/data-deeplink-url|openSourceUrl/)
    expect(surface).toMatch(/data-error-count|data-warning-count/)
    expect(surface).toMatch(/emptyTitle|emptyBody/)
    expect(surface).toMatch(/No findings|project is clean|emptyTitle/)
    // Error strip + retry (not empty-state-as-error only)
    expect(surface).toMatch(/data-checks-error/)
    expect(surface).toMatch(/data-retry-checks|onRetryChecks|Retry/)
    expect(surface).toMatch(/<collapsible\b/)
    expect(surface).toMatch(/<dialog\b/)
    expect(surface).not.toMatch(/commit intent/i)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<empty-state\b/)
    expect(surface).toMatch(/<scroll-area\b/)
  })

  it('design-surface ban-clean + workbench markers', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/design-surface/design-surface.sail'),
      'utf8',
    )
    const inspector = readFileSync(
      join(REPO, 'apps/studio/src/design-inspector/design-inspector.sail'),
      'utf8',
    )
    const combined = `${surface}\n${inspector}`
    expect(surface).toMatch(/data-design-surface/)
    expect(surface).toMatch(/data-surface="design"/)
    expect(surface).toMatch(/data-group-rail|data-decisions-rail/)
    expect(surface).toMatch(/data-token-list|data-token-editor/)
    expect(surface).toMatch(/<design-inspector\b/)
    expect(inspector).toMatch(/data-design-inspector/)
    expect(inspector).toMatch(/data-specimen-live|data-specimen-canvas/)
    expect(combined).not.toMatch(/<div\b/)
    expect(combined).not.toMatch(/<span\b/)
    expect(combined).toMatch(/<empty-state\b/)
    expect(combined).toMatch(/<scroll-area\b/)
  })

  it('tasks-surface: status dots, compact rows, ban-clean', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/tasks-surface/tasks-surface.sail'),
      'utf8',
    )
    expect(surface).toMatch(/data-tasks-surface/)
    expect(surface).toMatch(/data-surface="tasks"/)
    expect(surface).toMatch(/data-threads-list/)
    expect(surface).toMatch(/data-tasks-table/)
    expect(surface).toMatch(/data-task-detail/)
    expect(surface).toMatch(/data-task-conversation/)
    expect(surface).toMatch(/data-anchor-card/)
    expect(surface).toMatch(/data-commit-intent/)
    expect(surface).toMatch(/Save decision/)
    expect(surface).toMatch(/data-filter-actor/)
    expect(surface).toMatch(/data-status-badge|data-status-dot/)
    expect(surface).toMatch(/tone=\{row\.statusTone\}/)
    expect(surface).toMatch(/data-retry-tasks|onRetryTasks|Retry/)
    expect(surface).toMatch(/data-accept-live|data-rebind-selection/)
    expect(surface).toMatch(/attachCountLabel/)
    expect(surface).not.toMatch(/commit intent/i)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<empty-state\b/)
    expect(surface).toMatch(/<dialog\b/)
    expect(surface).toMatch(/<scroll-area\b/)
  })

  it('viewport picker absent from studio app sources', () => {
    const roots = [
      'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail',
      'apps/studio/src/shell/shell-page-vm.ts',
      'apps/studio/src/edit-surface/edit-surface.sail',
      'apps/studio/routes/page.sail',
      'apps/studio/routes/edit/page.sail',
    ]
    for (const rel of roots) {
      const src = readFileSync(join(REPO, rel), 'utf8')
      expect(src, rel).not.toMatch(/toggle-group[\s\S]*viewport|onViewport|Preview viewport/)
    }
  })

  // Batched sail check → test/studio-2-repairs.test.ts (STUDIO-2 R6)

  it('sail.config renders client-side (SPA workbench)', () => {
    const cfg = readFileSync(join(REPO, 'apps/studio/sail.config.jsonc'), 'utf8')
    expect(cfg).toMatch(/"render"\s*:\s*"csr"/)
  })
})

// Unit 7 close-tab promote → test/shell-tabs.test.ts