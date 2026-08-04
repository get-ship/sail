import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  ACTIVITY_BY_DIGIT,
  activityFromPath,
  appRouteBase,
  hotkeyMod,
  hrefForActivity,
  pathForActivity,
  railLabel,
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
    expect(routesGen).toMatch(/pattern: '\/components'/)
    expect(routesGen).toMatch(/pattern: '\/design'/)
    expect(routesGen).toMatch(/pattern: '\/themes'/)
    expect(routesGen).toMatch(/pattern: '\/tasks'/)
    expect(routesGen).toMatch(/tag: 'sail-studio-chrome'/)
    expect(routesGen).toMatch(/layoutModules/)
    expect(layout).toMatch(/<sail-studio-chrome/)
    expect(existsSync(join(REPO, 'apps/studio/routes/edit/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/checks/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/components/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/design/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/themes/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/tasks/page.sail'))).toBe(true)
    // /design is redirect-only (Themes cutover)
    const designPage = readFileSync(join(REPO, 'apps/studio/routes/design/page.sail'), 'utf8')
    expect(designPage).toMatch(/\/themes/)
    expect(designPage).not.toMatch(/<design-surface\b/)
  })

  it('leaf routes mount surfaces; no activity-signal switching in chrome', () => {
    for (const [leaf, tag] of [
      ['edit', 'edit-surface'],
      ['checks', 'checks-surface'],
      ['components', 'components-surface'],
      ['themes', 'themes-surface'],
      ['tasks', 'tasks-surface'],
    ] as const) {
      const page = readFileSync(join(REPO, `apps/studio/routes/${leaf}/page.sail`), 'utf8')
      expect(page).toMatch(new RegExp(`<${tag}\\b`))
    }
    // Signal switching deleted from shell (rail uses links + click→navigate, not setActivity)
    expect(shellVm).not.toMatch(/setActivity/)
    expect(shellVm).not.toMatch(/isEdit|isChecks|isDesign|isThemes|isTasks/)
    // Rail = real links + optimistic navigate/sync on click
    expect(chrome).toMatch(/href=\{item\.href\}/)
    expect(chrome).toMatch(/data-studio-rail/)
    expect(chrome).toMatch(/on:click=\{onRailClick\}/)
    expect(shellVm).toMatch(/onRailClick/)
    expect(shellVm).toMatch(/navigate\(href\)/)
    expect(chrome).toMatch(/<tooltip\b/)
    expect(shellVm).toMatch(/hrefForActivity|activityFromPath/)
  })

  it('activityFromPath + hrefForActivity cover all surfaces', () => {
    expect(activityFromPath('/edit')).toBe('edit')
    expect(activityFromPath('/')).toBe('edit')
    expect(activityFromPath('/checks')).toBe('checks')
    expect(activityFromPath('/components')).toBe('components')
    expect(activityFromPath('/components/sail-tree')).toBe('components')
    expect(activityFromPath('/design')).toBe('themes')
    expect(activityFromPath('/themes')).toBe('themes')
    expect(activityFromPath('/tasks')).toBe('tasks')
    expect(hrefForActivity('edit')).toMatch(/\/edit$/)
    expect(hrefForActivity('checks')).toMatch(/\/checks$/)
    expect(hrefForActivity('components')).toMatch(/\/components$/)
    expect(hrefForActivity('themes')).toMatch(/\/themes$/)
    expect(pathForActivity('/edit')).toBe('/edit')
    // appRouteBase is empty outside browser meta (overlay sets deck-app-route)
    expect(appRouteBase()).toBe('')
  })

  it('rail active flips when path changes (SPA leaf navigation model)', () => {
    // Pure rows: each path marks exactly one activity active
    const edit = railRowsForPath('/edit')
    expect(edit.find((r) => r.id === 'edit')?.active).toBe(true)
    expect(edit.filter((r) => r.active)).toHaveLength(1)
    expect(edit.find((r) => r.id === 'design')).toBeUndefined()

    const checks = railRowsForPath('/checks')
    expect(checks.find((r) => r.id === 'checks')?.active).toBe(true)
    expect(checks.find((r) => r.id === 'edit')?.active).toBe(false)

    // Legacy /design path highlights Themes (cutover)
    const design = railRowsForPath('/design')
    expect(design.find((r) => r.id === 'themes')?.active).toBe(true)
    expect(design.find((r) => r.id === 'design')).toBeUndefined()

    const themes = railRowsForPath('/themes')
    expect(themes.find((r) => r.id === 'themes')?.active).toBe(true)

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

  it('STUDIO-0 hotkeys: digit map, rail labels, save gate, status markers', () => {
    expect(ACTIVITY_BY_DIGIT).toEqual({
      '1': 'edit',
      '2': 'components',
      '3': 'themes',
      '4': 'tasks',
      '5': 'checks',
    })
    const mod = hotkeyMod()
    expect(mod === '⌘' || mod === 'Ctrl+').toBe(true)
    expect(railLabel('edit', 'Ctrl+')).toBe('Edit Ctrl+1')
    expect(railLabel('components', '⌘')).toBe('Components ⌘2')
    expect(railLabel('themes', 'Ctrl+')).toBe('Themes Ctrl+3')
    expect(railLabel('checks', '⌘')).toBe('Checks ⌘5')
    const themeRows = railRowsForPath('/themes', 'Ctrl+')
    expect(themeRows.find((r) => r.id === 'themes')?.label).toBe('Themes Ctrl+3')
    expect(themeRows.find((r) => r.id === 'tasks')?.label).toBe('Tasks Ctrl+4')
    expect(themeRows.find((r) => r.id === 'components')?.label).toBe('Components Ctrl+2')
    expect(themeRows.find((r) => r.id === 'design')).toBeUndefined()

    // Cmd/Ctrl+S always preventDefault; save only when activePath
    expect(shellVm).toMatch(/e\.key === 's'/)
    expect(shellVm).toMatch(/preventDefault\(\)/)
    expect(shellVm).toMatch(/activePath/)
    expect(shellVm).not.toMatch(/view === ['"]source['"]/)
    expect(shellVm).toMatch(/subscribeEdit/)
    expect(shellVm).toMatch(/sync\(href\)|sync\(p\)|sync\(path/)
    expect(shellVm).toMatch(/ACTIVITY_BY_DIGIT/)
    expect(shellVm).toMatch(/onRailClick/)

    expect(chrome).toMatch(/data-save-status/)
    expect(chrome).toMatch(/data-theme-status/)
    expect(chrome).toMatch(/data-batch-count/)
    expect(chrome).toMatch(/saveStatusLabel|themeStatusLabel/)
  })

  it('batch new task uses deck navigate, not raw pushState', () => {
    expect(shellVm).toMatch(/from ['"]@ship\/deck\/router['"]/)
    expect(shellVm).toMatch(/hrefForActivity\('tasks'\)/)
    expect(shellVm).toMatch(/navigate\(href\)/)
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

  it('L0 shell layout: rail token, h(8) header rhythm, stable zones', () => {
    const activityBar = readFileSync(
      join(REPO, 'apps/studio/src/studio-activity-bar/studio-activity-bar.sail'),
      'utf8',
    )
    const sidebar = readFileSync(
      join(REPO, 'apps/studio/src/studio-sidebar/studio-sidebar.sail'),
      'utf8',
    )
    const panel = readFileSync(
      join(REPO, 'apps/studio/src/studio-panel/studio-panel.sail'),
      'utf8',
    )
    const layout = readFileSync(join(REPO, 'apps/studio/routes/_layout.sail'), 'utf8')

    // Rail host width: one consistent outer token (items/brand may be w(10) hit targets)
    expect(activityBar).toMatch(/data-studio-activity-bar[\s\S]*\bw\(12\)/)
    expect(activityBar).toMatch(/minW\(12\)/)
    // Brand mark: same hit target as activity items (w/h 10), muted surface chip
    expect(activityBar).toMatch(/data-studio-brand-mark/)
    expect(activityBar).toMatch(/data-studio-brand-mark[\s\S]*\bw\(10\)/)
    expect(activityBar).toMatch(/icon name="ship"/)
    expect(activityBar).toMatch(/minH\(0\)/)
    expect(activityBar).toMatch(/h\(full\)/)
    expect(activityBar).toMatch(/aria-label="Studio activities"/)
    // Tooltips must paint — rail does not clip overflow
    expect(activityBar).toMatch(/overflow\(visible\)/)

    // Sidebar: fixed width token + h(8) header + body flex chain
    expect(sidebar).toMatch(/data-studio-sidebar[\s\S]*\bw\(64\)/)
    expect(sidebar).toMatch(/h\(8\)/)
    expect(sidebar).toMatch(/data-studio-sidebar-body[\s\S]*minH\(0\)/)
    expect(sidebar).toMatch(/basis\(0\)/)
    expect(sidebar).toMatch(/overflow\(hidden\)/)

    // Panel: same h(8) header rhythm + grow body chain
    expect(panel).toMatch(/h\(8\)/)
    expect(panel).toMatch(/data-studio-panel-body[\s\S]*minH\(0\)/)
    expect(panel).toMatch(/minH\(0\)/)

    // Chrome: stable surface slot; rail items labeled; SPA chrome reuse via layout
    expect(chrome).toMatch(/data-studio-surface[\s\S]*minH\(0\)/)
    expect(chrome).toMatch(/data-studio-surface[\s\S]*overflow\(hidden\)/)
    expect(chrome).toMatch(/aria-label=\{item\.label\}/)
    expect(chrome).toMatch(/<tooltip\b/)
    expect(chrome).toMatch(/title=\{item\.label\}/)
    expect(chrome).toMatch(/placement="right"/)
    expect(layout).toMatch(/<sail-studio-chrome/)
    // Status fixed height; batch tray fixed when present (no content-driven chrome resize)
    expect(chrome).toMatch(/data-studio-status[\s\S]*h\(6\)/)
    expect(chrome).toMatch(/data-batch-tray[\s\S]*h\(7\)/)
  })

  it('rail selection uses selected-bg + primary fg (only the active item is accented)', () => {
    expect(chrome).toMatch(/aria-current=\{item\.active/)
    expect(chrome).toMatch(/data-active=\{item\.active/)
    expect(chrome).toMatch(/when\(\{\s*current:\s*style\.bg\(selected-bg\)\.fg\(primary\)/)
    expect(chrome).toMatch(/fg\(muted-foreground\)/)
    // Distinct icons per activity (not one generic glyph)
    expect(shellVm).toMatch(/icon:\s*'files'/)
    expect(shellVm).toMatch(/icon:\s*'box'/)
    expect(shellVm).toMatch(/icon:\s*'clipboard-check'/)
    expect(shellVm).toMatch(/icon:\s*'palette'/)
    expect(shellVm).toMatch(/icon:\s*'messages-square'/)
    expect(shellVm).not.toMatch(/icon:\s*'swatch-book'/)
    // Exit matches activity item geometry
    expect(chrome).toMatch(/data-exit-studio[\s\S]*w\(10\)\.h\(10\)/)
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

  it('edit-surface: doc tabs + source editor only (no nested view menus)', () => {
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
    expect(surface).toMatch(/data-edit-tab-list/)
    expect(surface).toMatch(/data-tab-id/)
    expect(surface).toMatch(/data-tab-close|showClose/)
    expect(surface).toMatch(/overflowX\(auto\)/)
    expect(surface).toMatch(/onTabMiddle|on:mousedown=\{onTabMiddle\}/)
    expect(surface).toMatch(/Open a file/)
    expect(surface).toMatch(/data-source-editor/)
    expect(surface).toMatch(/data-source-host/)
    expect(surface).toMatch(/sail-code-editor/)
    // Nested view menus removed (preview/state/layout cram)
    expect(surface).not.toMatch(/data-view-switcher/)
    expect(surface).not.toMatch(/data-view-panel=/)
    expect(surface).not.toMatch(/data-preview-host/)
    expect(surface).not.toMatch(/data-live-preview/)
    expect(surface).not.toMatch(/name="preview"|name="state"|name="layout"/)
    expect(surface).not.toMatch(/data-struct-op|data-edit-rail|data-structured-edit/)
    expect(surface).not.toMatch(/toggle-group|onViewport|viewport picker/i)
    const tabsMod = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-tabs.ts'),
      'utf8',
    )
    expect(tabsMod).toMatch(/showClose/)
    expect(surfaceVm).toMatch(/mapDocTabs|onTabMiddle/)
    expect(surfaceVm).toMatch(/closeDocTab|openShellTabDocument/)
    expect(surfaceVm).toMatch(/setView\(\s*['"]source['"]\s*\)/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
  })

  it('source pane: CodeMirror host density + ready/empty/loading + save (no SOURCE toolbar)', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface.sail'),
      'utf8',
    )
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-vm.ts'),
      'utf8',
    )
    const tabsMod = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-tabs.ts'),
      'utf8',
    )
    const sailMode = readFileSync(join(REPO, 'apps/studio/src/cm/sail-mode.ts'), 'utf8')
    expect(surface).toMatch(/data-source-host/)
    // No second chrome row — status on tab + status bar
    expect(surface).not.toMatch(/data-source-toolbar/)
    expect(surface).not.toMatch(/>Source</)
    expect(surface).toMatch(/data-source-loading|isSourceLoading/)
    expect(surface).toMatch(/data-source-ready|isSourceReady/)
    expect(surface).toMatch(/data-source-empty|isSourceEmpty/)
    expect(surface).toMatch(/data-source-editor/)
    expect(surface).toMatch(/language="sail"/)
    expect(surface).toMatch(/line-numbers/)
    expect(surface).toMatch(/size="sm"/)
    expect(surface).toMatch(/chrome="flush"/)
    expect(surface).toMatch(/data-save-source/)
    expect(surface).toMatch(/on:sail-save=\{onSaveSource\}/)
    expect(surface).toMatch(/on:sail-change=\{onSourceSailChange\}/)
    expect(surfaceVm).toMatch(/isSourceReady/)
    expect(surfaceVm).toMatch(/isSourceEmpty/)
    expect(surfaceVm).toMatch(/isSourceLoading/)
    expect(surfaceVm).toMatch(/sourceDirty/)
    expect(surfaceVm).toMatch(/pushSourceDiagnostics|setDiagnostics/)
    expect(surfaceVm).toMatch(/trySourceScroll|scrollSourceEditor/)
    expect(surfaceVm).toMatch(/onSaveSource|saveActiveSource/)
    expect(surfaceVm).toMatch(/setSourceTextFromEditor/)
    expect(tabsMod).toMatch(/pathAwareTabLabel/)
    expect(sailMode).toMatch(/tokenizeSail|sailStreamTokenizer/)
  })

  it('edit-explorer: full-height project file tree only', () => {
    const explorer = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer.sail'),
      'utf8',
    )
    const explorerVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer-vm.ts'),
      'utf8',
    )
    expect(explorer).toMatch(/data-edit-explorer/)
    expect(explorer).toMatch(/data-file-tree/)
    expect(explorer).toMatch(/Explorer/)
    expect(explorer).toMatch(/<studio-sidebar\b/)
    expect(explorer).toMatch(/title="Explorer"/)
    expect(explorer).toMatch(/<sail-tree\b/)
    expect(explorer).toMatch(/data-file-tree-host/)
    expect(explorer).toMatch(/data-edit-filter/)
    expect(explorer).toMatch(/Filter files/)
    expect(explorer).toMatch(/No files/)
    // No nested Routes/Files/Components menus
    expect(explorer).not.toMatch(/data-rail-tabs/)
    expect(explorer).not.toMatch(/name="routes"|name="components"/)
    expect(explorer).not.toMatch(/data-rail-panel/)
    expect(explorer).not.toMatch(/edit-element-tree/)
    // Single full-height tree host (no dual-panel split)
    expect(explorer).toMatch(/data-file-tree[\s\S]*h\(full\)/)
    expect(explorerVm).toMatch(/createTable/)
    expect(explorerVm).toMatch(/fileSailTreeNodes/)
    expect(explorerVm).not.toMatch(/routeSailTreeNodes/)
    expect(explorerVm).not.toMatch(/railTab/)
    expect(explorerVm).toMatch(/startsWith\('group:'\)/)
    expect(explorerVm).toMatch(/startsWith\('dir:'\)/)
    // Identity bind — live Table returned + tree={tree} (no ambient assign)
    expect(explorerVm).toMatch(/\btree\b/)
    expect(explorerVm).not.toMatch(/assignTreeHost/)
    expect(explorerVm).not.toMatch(/querySelector/)
    expect(explorer).toMatch(/tree=\{tree\}/)
    expect(explorer).not.toMatch(/assignTreeHost/)
    expect(explorer).not.toMatch(/querySelector/)
    expect(explorer).not.toMatch(/<div\b/)
  })

  it('Files tree uses trailing dirty + nested paths (not label cram)', () => {
    const explorer = readFileSync(join(REPO, 'apps/studio/src/edit/explorer.ts'), 'utf8')
    const treeSail = readFileSync(
      join(REPO, 'packages/components/sail/primitives/src/sail-tree/sail-tree.sail'),
      'utf8',
    )
    const treeView = readFileSync(
      join(REPO, 'packages/components/sail/primitives/src/sail-tree/sail-tree.view.ts'),
      'utf8',
    )
    expect(explorer).toMatch(/function fileSegToSailTree/)
    expect(explorer).toMatch(/trailing:\s*dirty\?\.has\(node\.filePath\)\s*\?\s*'●'/)
    expect(explorer).toMatch(/id:\s*`dir:\$\{groupId\}\/\$\{rel\}`/)
    expect(explorer).toMatch(/export function fileSailTreeNodes/)
    expect(treeView).toMatch(/trailing:/)
    expect(treeSail).toMatch(/w\.trailing/)
    expect(treeSail).toMatch(/node-trailing/)
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
    expect(chrome).not.toMatch(/<components-surface\b/)
    expect(chrome).not.toMatch(/<design-surface\b/)
    expect(chrome).not.toMatch(/<themes-surface\b/)
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
    const vm = readFileSync(
      join(REPO, 'apps/studio/src/checks-surface/checks-surface-vm.ts'),
      'utf8',
    )
    expect(surface).toMatch(/data-checks-surface/)
    expect(surface).toMatch(/data-surface="checks"/)
    expect(surface).toMatch(/data-findings-list/)
    expect(surface).toMatch(/data-findings-table|data-rule-group/)
    expect(surface).toMatch(/data-findings-table-host/)
    expect(surface).toMatch(/data-severity-group/)
    expect(surface).toMatch(/data-filter-severity/)
    expect(surface).toMatch(/data-review-fixes/)
    expect(surface).toMatch(/data-review-dialog/)
    expect(surface).toMatch(/Review fixes/)
    expect(surface).toMatch(/applyFixesLabel|Apply/)
    expect(surface).toMatch(/data-both-viewports|showBothViewports/)
    expect(surface).toMatch(/data-deeplink-url|openSourceUrl/)
    expect(surface).toMatch(/data-error-count|data-warning-count/)
    expect(surface).toMatch(/emptyTitle|emptyBody/)
    expect(surface).toMatch(/No findings|project is clean|emptyTitle|Run checks/)
    expect(surface).toMatch(/data-empty-checked|emptyChecked/)
    expect(surface).toMatch(/data-autofix-control|data-autofix-mode/)
    expect(surface).toMatch(/data-recheck|onRecheck/)
    expect(surface).toMatch(/Open in Edit/)
    expect(surface).toMatch(/Select element/)
    expect(surface).toMatch(/<sail-diff\b/)
    expect(vm).toMatch(/createTable/)
    expect(vm).toMatch(/assignTableHost/)
    expect(vm).toMatch(/table\.rows\s*\(|groupsFromTable/)
    expect(vm).toMatch(/buildExpandedMap|setExpanded|applyExpandedMap/)
    expect(vm).not.toMatch(/expanded:\s*true/)
    // Error strip + retry (not empty-state-as-error only)
    expect(surface).toMatch(/data-checks-error/)
    expect(surface).toMatch(/data-retry-checks|onRetryChecks|Retry/)
    expect(surface).toMatch(/<collapsible\b/)
    expect(surface).toMatch(/<dialog\b/)
    expect(surface).not.toMatch(/commit intent/i)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    // Empty via studio-empty (wraps empty-state); may still use raw empty-state in dialogs
    expect(surface).toMatch(/<studio-empty\b|<empty-state\b/)
    expect(surface).toMatch(/<studio-page-header\b/)
    expect(surface).toMatch(/<studio-toolbar\b/)
    expect(surface).toMatch(/<scroll-area\b/)
  })

  it('Design product surface removed; Themes chrome has no permanent Used by', () => {
    expect(existsSync(join(REPO, 'apps/studio/src/design-surface'))).toBe(false)
    expect(existsSync(join(REPO, 'apps/studio/src/design-inspector'))).toBe(false)
    const chromeFacet = readFileSync(
      join(REPO, 'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail'),
      'utf8',
    )
    expect(chromeFacet).not.toMatch(/data-used-by/)
    expect(chromeFacet).toMatch(/data-theme-facet-chrome/)
    expect(chromeFacet).not.toMatch(/blast radius|data-blast-radius/i)
    expect(shellVm).not.toMatch(/previewInApp/)
    expect(shellVm).not.toMatch(/activityFromPath\(readPath\(\)\) === ['"]design['"]/)
  })

  it('themes-surface ban-clean + master–detail markers', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/themes-surface/themes-surface.sail'),
      'utf8',
    )
    const vm = readFileSync(
      join(REPO, 'apps/studio/src/themes-surface/themes-surface-vm.ts'),
      'utf8',
    )
    expect(surface).toMatch(/data-themes-surface/)
    expect(surface).toMatch(/data-surface="themes"/)
    expect(surface).toMatch(/data-themes-workbench|data-themes-split/)
    expect(surface).toMatch(/data-themes-list/)
    expect(surface).toMatch(/data-themes-preview|data-theme-preview-pane/)
    expect(surface).toMatch(/data-theme-row/)
    expect(surface).toMatch(/data-list-group/)
    expect(surface).toMatch(/data-new-theme/)
    // Job 5d: paint authority is theme={id}, not load-bearing data-theme-preview
    expect(surface).toMatch(/theme=\{row\.previewId\}|theme=\{previewPaintId\(\)\}/)
    expect(surface).not.toMatch(/data-theme-preview=\{/)
    expect(surface).toMatch(/createThemesSurface\(\)/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<scroll-area\b/)
    expect(surface).toMatch(/panel-min-w-md|panel-max-w-md/)
    expect(surface).not.toMatch(/document\.querySelector/)
    expect(surface).not.toMatch(/\.style\.setProperty/)
    expect(surface).not.toMatch(/data-action=/)
    expect(vm).toMatch(/loadThemes|openTheme|addThemeToProject|forkTheme/)
    expect(vm).toMatch(/subscribeDesign|getDesignState/)
    expect(vm).toMatch(/from ['"]@ship\/studio\/design['"]/)
    expect(vm).toMatch(/mountAmbient/)
    expect(vm).not.toMatch(/document\.querySelector/)
    expect(vm).not.toMatch(/\.style\.setProperty/)
    expect(vm).not.toMatch(/design-surface/)
    expect(existsSync(join(REPO, 'apps/studio/routes/themes/page.sail'))).toBe(true)
    const page = readFileSync(join(REPO, 'apps/studio/routes/themes/page.sail'), 'utf8')
    expect(page).toMatch(/<themes-surface\b/)
  })

  it('tasks-surface: status dots, chat-elements, ban-clean', () => {
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
    expect(surface).toMatch(/>\s*Commit\s*</)
    expect(surface).toMatch(/data-filter-actor/)
    expect(surface).toMatch(/data-status-badge|data-status-dot/)
    expect(surface).toMatch(/tone=\{row\.statusTone\}/)
    expect(surface).toMatch(/data-retry-tasks|onRetryTasks|Retry/)
    expect(surface).toMatch(/data-accept-live|data-rebind-selection/)
    expect(surface).toMatch(/attachCountLabel/)
    expect(surface).toMatch(/<sail-composer\b/)
    expect(surface).toMatch(/<sail-message-list\b/)
    expect(surface).toMatch(/<sail-markdown\b/)
    expect(surface).not.toMatch(/commit intent/i)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<empty-state\b/)
    expect(surface).toMatch(/<studio-empty\b/)
    expect(surface).toMatch(/<studio-page-header\b/)
    expect(surface).toMatch(/<studio-toolbar\b/)
    expect(surface).not.toMatch(/<dialog\b/)
    expect(surface).toMatch(/data-tasks-workbench/)
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