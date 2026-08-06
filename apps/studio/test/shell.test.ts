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
const SHELL_RAIL = join(REPO, 'apps/studio/src/shell/shell-rail.ts')
const ROUTES_GEN = join(REPO, 'apps/studio/routes.gen.ts')

describe('Studio routes + chrome (STUDIO-1)', () => {
  const chrome = readFileSync(CHROME, 'utf8')
  const shellVm = readFileSync(SHELL_VM, 'utf8')
  const shellRail = readFileSync(SHELL_RAIL, 'utf8')
  const routesGen = readFileSync(ROUTES_GEN, 'utf8')
  const layout = readFileSync(join(REPO, 'apps/studio/routes/_layout.sail'), 'utf8')

  it('routes.gen exposes leaf surfaces + layout chrome', () => {
    expect(routesGen).toMatch(/pattern: '\/edit'/)
    expect(routesGen).toMatch(/pattern: '\/checks'/)
    expect(routesGen).toMatch(/pattern: '\/config'/)
    expect(routesGen).toMatch(/pattern: '\/components'/)
    expect(routesGen).toMatch(/pattern: '\/design'/)
    expect(routesGen).toMatch(/pattern: '\/themes'/)
    expect(routesGen).toMatch(/pattern: '\/tasks'/)
    expect(routesGen).toMatch(/tag: 'sail-studio-chrome'/)
    expect(routesGen).toMatch(/layoutModules/)
    expect(layout).toMatch(/<sail-studio-chrome/)
    expect(existsSync(join(REPO, 'apps/studio/routes/edit/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/checks/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/config/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/components/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/design/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/themes/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/routes/tasks/page.sail'))).toBe(true)
    // /design is redirect-only (Themes cutover)
    const designPage = readFileSync(join(REPO, 'apps/studio/routes/design/page.sail'), 'utf8')
    expect(designPage).toMatch(/\/themes/)
    expect(designPage).not.toMatch(/<design-surface\b/)
    // /checks deep-link opens panel + lands on Edit (not a rail page)
    const checksPage = readFileSync(join(REPO, 'apps/studio/routes/checks/page.sail'), 'utf8')
    expect(checksPage).toMatch(/setChecksPanelOpen/)
    expect(checksPage).toMatch(/\/edit/)
    expect(checksPage).not.toMatch(/<checks-surface\b/)
  })

  it('leaf routes mount surfaces; no activity-signal switching in chrome', () => {
    for (const [leaf, tag] of [
      ['edit', 'edit-surface'],
      ['config', 'config-surface'],
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
    expect(chrome).toMatch(/studio-rail/)
    expect(chrome).toMatch(/on:click=\{onRailClick\}/)
    expect(shellVm).toMatch(/onRailClick/)
    expect(shellVm).toMatch(/navigate\(href\)/)
    expect(chrome).toMatch(/<tooltip\b/)
    expect(shellVm).toMatch(/hrefForActivity|activityFromPath/)
  })

  it('activityFromPath + hrefForActivity cover all surfaces', () => {
    expect(activityFromPath('/edit')).toBe('edit')
    expect(activityFromPath('/')).toBe('edit')
    // Checks deep-link maps to edit activity (panel is chrome-hosted)
    expect(activityFromPath('/checks')).toBe('edit')
    expect(activityFromPath('/config')).toBe('config')
    expect(activityFromPath('/components')).toBe('components')
    expect(activityFromPath('/components/sail-tree')).toBe('components')
    expect(activityFromPath('/design')).toBe('themes')
    expect(activityFromPath('/themes')).toBe('themes')
    expect(activityFromPath('/tasks')).toBe('tasks')
    expect(hrefForActivity('edit')).toMatch(/\/edit$/)
    expect(hrefForActivity('config')).toMatch(/\/config$/)
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
    expect(edit.find((r) => r.id === 'checks')).toBeUndefined()

    // /checks deep-link highlights Edit (panel opens separately)
    const checksPath = railRowsForPath('/checks')
    expect(checksPath.find((r) => r.id === 'edit')?.active).toBe(true)
    expect(checksPath.find((r) => r.id === 'checks')).toBeUndefined()

    const config = railRowsForPath('/config')
    expect(config.find((r) => r.id === 'config')?.active).toBe(true)
    expect(config.find((r) => r.id === 'edit')?.active).toBe(false)

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
      '5': 'config',
    })
    const mod = hotkeyMod()
    expect(mod === '⌘' || mod === 'Ctrl+').toBe(true)
    expect(railLabel('edit', 'Ctrl+')).toBe('Edit Ctrl+1')
    expect(railLabel('components', '⌘')).toBe('Components ⌘2')
    expect(railLabel('themes', 'Ctrl+')).toBe('Themes Ctrl+3')
    expect(railLabel('config', '⌘')).toBe('Config ⌘5')
    const themeRows = railRowsForPath('/themes', 'Ctrl+')
    expect(themeRows.find((r) => r.id === 'themes')?.label).toBe('Themes Ctrl+3')
    expect(themeRows.find((r) => r.id === 'tasks')?.label).toBe('Tasks Ctrl+4')
    expect(themeRows.find((r) => r.id === 'components')?.label).toBe('Components Ctrl+2')
    expect(themeRows.find((r) => r.id === 'config')?.label).toBe('Config Ctrl+5')
    expect(themeRows.find((r) => r.id === 'design')).toBeUndefined()
    expect(themeRows.find((r) => r.id === 'checks')).toBeUndefined()

    // Cmd/Ctrl+S via declarative hotkeys; save only when activePath
    expect(shellVm).toMatch(/hotkeys\(/)
    expect(shellVm).toMatch(/['"]mod\+s['"]/)
    expect(shellVm).toMatch(/activePath/)
    expect(shellVm).not.toMatch(/view === ['"]source['"]/)
    expect(shellVm).toMatch(/subscribeEdit/)
    expect(shellVm).toMatch(/sync\(href\)|sync\(p\)|sync\(path/)
    expect(shellVm).toMatch(/ACTIVITY_BY_DIGIT/)
    expect(shellVm).toMatch(/onRailClick/)
    expect(shellVm).toMatch(/toggleChecksPanel|setChecksPanelOpen/)

    expect(chrome).toMatch(/save-status/)
    expect(chrome).toMatch(/theme-status/)
    expect(chrome).toMatch(/batch-count/)
    expect(chrome).toMatch(/checks-badge/)
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
    expect(chrome).toMatch(/sail-studio-shell/)
    expect(chrome).toMatch(/studio-rail/)
    expect(chrome).toMatch(/studio-status/)
    expect(chrome).toMatch(/slot name="route"/)
    // Explicit removals
    expect(chrome).not.toMatch(/toggle-group|onViewport|viewport/)
    expect(chrome).not.toMatch(/hotkey-hints|>esc</)
    expect(chrome).not.toMatch(/studio-tabs/)
    expect(chrome).not.toMatch(/selectionLabel/)
    expect(shellVm).not.toMatch(/setViewport|onViewport/)
    expect(shellVm).not.toMatch(/ViewportId/)
    // Health tones bound (soft badge tone from ops/watch health signals)
    expect(chrome).toMatch(/tone=\{opsHealthTone\(\)\}/)
    expect(chrome).toMatch(/tone=\{watchHealthTone\(\)\}/)
    // Checks panel host (not rail)
    expect(chrome).toMatch(/checks-panel/)
    expect(chrome).toMatch(/checksPanelOpen/)
    expect(chrome).toMatch(/checks-badge/)
  })

  it('status bar is slim with health segments', () => {
    expect(chrome).toMatch(/studio-status/)
    expect(chrome).toMatch(/health="ops"/)
    expect(chrome).toMatch(/health="watch"/)
    expect(chrome).toMatch(/studio-status[\s\S]*\bnowrap\b/)
    expect(chrome).toMatch(/studio-status[\s\S]*minW\(0\)|studio-status[\s\S]*min-w="0"/)
    expect(chrome).toMatch(/studio-status[\s\S]*truncate\(\)/)
    expect(chrome).toMatch(/studio-status[\s\S]*h\(6\)/)
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
    expect(activityBar).toMatch(/studio-activity-bar[\s\S]*\bw\(12\)/)
    expect(activityBar).toMatch(/minW\(12\)/)
    // Brand mark: same hit target as activity items (w/h 10), muted surface chip
    expect(activityBar).toMatch(/studio-brand-mark/)
    expect(activityBar).toMatch(/studio-brand-mark[\s\S]*\bw\(10\)/)
    expect(activityBar).toMatch(/icon name="ship"/)
    expect(activityBar).toMatch(/minH\(0\)/)
    expect(activityBar).toMatch(/h\(full\)/)
    expect(activityBar).toMatch(/label="Studio activities"/)
    // Tooltips must paint — rail does not clip overflow
    expect(activityBar).toMatch(/overflow\(visible\)/)

    // Sidebar: fixed width token + h(8) header + body flex chain
    expect(sidebar).toMatch(/studio-sidebar[\s\S]*\bw\(64\)/)
    expect(sidebar).toMatch(/h\(8\)/)
    expect(sidebar).toMatch(/studio-sidebar-body[\s\S]*minH\(0\)/)
    expect(sidebar).toMatch(/basis\(0\)/)
    expect(sidebar).toMatch(/overflow\(hidden\)/)

    // Panel: same h(8) header rhythm + grow body chain
    expect(panel).toMatch(/h\(8\)/)
    expect(panel).toMatch(/studio-panel-body[\s\S]*minH\(0\)/)
    expect(panel).toMatch(/minH\(0\)/)

    // Chrome: stable surface slot; rail items labeled; SPA chrome reuse via layout
    expect(chrome).toMatch(/studio-surface[\s\S]*minH\(0\)/)
    expect(chrome).toMatch(/studio-surface[\s\S]*overflow\(hidden\)/)
    expect(chrome).toMatch(/label=\{item\.label\}/)
    expect(chrome).toMatch(/<tooltip\b/)
    expect(chrome).toMatch(/title=\{item\.label\}/)
    expect(chrome).toMatch(/placement="right"/)
    expect(layout).toMatch(/<sail-studio-chrome/)
    // Status fixed height; batch tray fixed when present (no content-driven chrome resize)
    expect(chrome).toMatch(/studio-status[\s\S]*h\(6\)/)
    expect(chrome).toMatch(/batch-tray[\s\S]*h\(7\)/)
  })

  it('rail selection uses selected-bg + primary fg (only the active item is accented)', () => {
    expect(chrome).toMatch(/current=\{item\.active/)
    expect(chrome).toMatch(/active=\{item\.active/)
    expect(chrome).toMatch(
      /when\(\{\s*current:\s*style\.bg\(selected-bg\)\.fg\(primary\)|when\(current,\s*\{\s*paint:\s*bg\(selected-bg\)\.fg\(primary\)/,
    )
    expect(chrome).toMatch(/fg\(muted-foreground\)/)
    // Distinct icons per activity (not one generic glyph)
    expect(shellRail).toMatch(/icon:\s*'files'/)
    expect(shellRail).toMatch(/icon:\s*'box'/)
    expect(shellRail).toMatch(/icon:\s*'settings'/)
    expect(shellRail).toMatch(/icon:\s*'palette'/)
    expect(shellRail).toMatch(/icon:\s*'messages-square'/)
    expect(shellRail).not.toMatch(/icon:\s*'clipboard-check'/)
    expect(shellRail).not.toMatch(/icon:\s*'swatch-book'/)
    // Exit matches activity item geometry
    expect(chrome).toMatch(/exit-studio[\s\S]*w\(10\)\.h\(10\)/)
    expect(chrome).not.toMatch(/variant=\{item\.active \? 'solid'/)
  })

  it('activity rail has Exit / Back to app (never a dead-end)', () => {
    expect(chrome).toMatch(/exit-studio/)
    expect(chrome).toMatch(/Back to app/)
    expect(chrome).toMatch(/onExitStudio|on:click=\{onExitStudio\}/)
    const exitVm = readFileSync(join(REPO, 'apps/studio/src/shell/exit.ts'), 'utf8')
    expect(exitVm).toMatch(/export function exitStudio/)
    expect(exitVm).toMatch(/type: 'close'/)
    expect(exitVm).toMatch(/postMessage/)
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
    expect(surface).toMatch(/edit-surface/)
    expect(surface).toMatch(/surface="edit"/)
    expect(surface).toMatch(/studio-tabs|edit-tabs/)
    expect(surface).toMatch(/edit-tab-list/)
    expect(surface).toMatch(/tab-id/)
    expect(surface).toMatch(/tab-close|showClose/)
    expect(surface).toMatch(/overflowX\(auto\)/)
    expect(surface).toMatch(/onTabMiddle|on:mousedown=\{onTabMiddle\}/)
    expect(surface).toMatch(/Open a file/)
    expect(surface).toMatch(/source-editor/)
    expect(surface).toMatch(/source-host/)
    expect(surface).toMatch(/sail-code-editor/)
    // Nested view menus removed (preview/state/layout cram)
    expect(surface).not.toMatch(/view-switcher/)
    expect(surface).not.toMatch(/view-panel=/)
    expect(surface).not.toMatch(/preview-host/)
    expect(surface).not.toMatch(/live-preview/)
    expect(surface).not.toMatch(/name="preview"|name="state"|name="layout"/)
    expect(surface).not.toMatch(/struct-op|edit-rail|structured-edit/)
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
    expect(surface).toMatch(/source-host/)
    // No second chrome row — status on tab + status bar
    expect(surface).not.toMatch(/source-toolbar/)
    expect(surface).not.toMatch(/>Source</)
    expect(surface).toMatch(/source-loading|isSourceLoading/)
    expect(surface).toMatch(/source-ready|isSourceReady/)
    expect(surface).toMatch(/source-empty|isSourceEmpty/)
    expect(surface).toMatch(/source-editor/)
    expect(surface).toMatch(/language="sail"/)
    expect(surface).toMatch(/line-numbers/)
    expect(surface).toMatch(/size="sm"/)
    expect(surface).toMatch(/chrome="flush"/)
    expect(surface).toMatch(/save-source/)
    expect(surface).toMatch(/on-save=\{onSaveSource\}/)
    expect(surface).toMatch(/on-change=\{onSourceSailChange\}/)
    // Bulk diagnostic list above editor removed — Checks panel + CM lints remain
    expect(surface).not.toMatch(/source-gutter/)
    expect(surface).not.toMatch(/checkRows/)
    expect(surfaceVm).not.toMatch(/\bcheckRows\b/)
    expect(surfaceVm).not.toMatch(/\bhasChecks\b/)
    expect(surfaceVm).toMatch(/isSourceReady/)
    expect(surfaceVm).toMatch(/isSourceEmpty/)
    expect(surfaceVm).toMatch(/isSourceLoading/)
    expect(surfaceVm).toMatch(/sourceDirty/)
    expect(surfaceVm).toMatch(/pushSourceDiagnostics|setDiagnostics/)
    expect(surfaceVm).toMatch(/uiCheckDiags/)
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
    expect(explorer).toMatch(/edit-explorer/)
    expect(explorer).toMatch(/file-tree/)
    expect(explorer).toMatch(/Explorer/)
    expect(explorer).toMatch(/<studio-sidebar\b/)
    expect(explorer).toMatch(/title="Explorer"/)
    expect(explorer).toMatch(/<sail-tree\b/)
    expect(explorer).toMatch(/file-tree-host/)
    expect(explorer).toMatch(/edit-filter/)
    expect(explorer).toMatch(/Filter files…|Filter files/)
    expect(explorer).toMatch(/No files/)
    // No nested Routes/Files/Components menus
    expect(explorer).not.toMatch(/rail-tabs/)
    expect(explorer).not.toMatch(/name="routes"|name="components"/)
    expect(explorer).not.toMatch(/rail-panel/)
    expect(explorer).not.toMatch(/edit-element-tree/)
    // Single full-height tree host (no dual-panel split)
    expect(explorer).toMatch(/file-tree[\s\S]*h\(full\)/)
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
      // Comments may mention createOpsClient(); ban call/import only.
      expect(vm).not.toMatch(/(?:^|\n)\s*(?:import|const|let|var)[^\n]*createOpsClient/)
      expect(vm).not.toMatch(/(?:^|\n)\s*[^/\n]*\bsetOpsClient\s*\(/)
    }
    // surface openDocument only via tab open helper
    expect(explorerVm).not.toMatch(/(?:^|\n)\s*(?:import|const|let|var)[^\n]*createOpsClient/)
    expect(explorerVm).not.toMatch(/(?:^|\n)\s*[^/\n]*\bsetOpsClient\s*\(/)
    expect(explorerVm).toMatch(/openDocument\s*\(/)
    const mountBody =
      explorerVm.match(/mountAmbient\s*\(\s*\(\)\s*=>\s*\{([\s\S]*?)\n  \}\)/)?.[1] ?? ''
    expect(mountBody.length).toBeGreaterThan(0)
    expect(mountBody).not.toMatch(/openDocument/)
  })

  it('chrome boots ops + seed; rail surfaces not dual-mounted (checks panel hosts findings)', () => {
    expect(chrome).not.toMatch(/<edit-surface\b/)
    expect(chrome).not.toMatch(/<components-surface\b/)
    expect(chrome).not.toMatch(/<design-surface\b/)
    expect(chrome).not.toMatch(/<themes-surface\b/)
    expect(chrome).not.toMatch(/<tasks-surface\b/)
    expect(chrome).not.toMatch(/<config-surface\b/)
    // Findings: L1 dialog side sheet overlay (not in-flow layout column)
    expect(chrome).toMatch(/checks-panel/)
    expect(chrome).toMatch(/<dialog\b[^>]*side="end"/)
    expect(chrome).toMatch(/checks-panel[\s\S]*<checks-surface\b|<checks-surface\b[\s\S]*checks-panel/)
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
    expect(surface).toMatch(/checks-surface/)
    expect(surface).toMatch(/surface="checks"/)
    expect(surface).toMatch(/findings-list/)
    expect(surface).toMatch(/findings-table|rule-group/)
    expect(surface).toMatch(/findings-table-host/)
    expect(surface).toMatch(/severity-group/)
    expect(surface).toMatch(/filter-severity/)
    expect(surface).toMatch(/review-fixes/)
    expect(surface).toMatch(/review-dialog/)
    expect(surface).toMatch(/Review fixes/)
    expect(surface).toMatch(/applyFixesLabel|Apply/)
    expect(surface).toMatch(/both-viewports|showBothViewports/)
    expect(surface).toMatch(/deeplink-url|openSourceUrl/)
    expect(surface).toMatch(/error-count|warning-count/)
    expect(surface).toMatch(/emptyTitle|emptyBody/)
    expect(surface).toMatch(/No findings|project is clean|emptyTitle|Run checks/)
    expect(surface).toMatch(/empty-checked|emptyChecked/)
    expect(surface).toMatch(/autofix-control|autofix-mode/)
    expect(surface).toMatch(/recheck|onRecheck/)
    expect(surface).toMatch(/Open in Edit/)
    expect(surface).toMatch(/Select element/)
    expect(surface).toMatch(/<sail-diff\b/)
    expect(vm).toMatch(/createTable/)
    expect(vm).toMatch(/assignTableHost/)
    expect(vm).toMatch(/table\.rows\s*\(|groupsFromTable/)
    expect(vm).toMatch(/buildExpandedMap|setExpanded|applyExpandedMap/)
    expect(vm).not.toMatch(/expanded:\s*true/)
    // Error strip + retry (not empty-state-as-error only)
    expect(surface).toMatch(/checks-error/)
    expect(surface).toMatch(/retry-checks|onRetryChecks|Retry/)
    expect(surface).toMatch(/<accordion\b/)
    // Review UI is an inline sheet; findings overlay dialog lives in chrome
    expect(surface).not.toMatch(/<dialog\b/)
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
    expect(chromeFacet).not.toMatch(/used-by/)
    expect(chromeFacet).toMatch(/theme-facet-chrome/)
    expect(chromeFacet).not.toMatch(/blast radius|blast-radius/i)
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
    expect(surface).toMatch(/themes-surface/)
    expect(surface).toMatch(/surface="themes"/)
    expect(surface).toMatch(/themes-workbench|themes-split/)
    expect(surface).toMatch(/themes-list/)
    expect(surface).toMatch(/themes-preview|theme-preview-pane/)
    expect(surface).toMatch(/theme-row/)
    expect(surface).toMatch(/list-group/)
    expect(surface).toMatch(/new-theme/)
    // Job 5d: paint authority is theme={id}, not load-bearing theme
    expect(surface).toMatch(/theme=\{row\.previewId\}|theme=\{previewPaintId\(\)\}/)
    expect(surface).not.toMatch(/theme-preview=\{/)
    expect(surface).toMatch(/createThemesSurface\(\)/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<scroll-area\b/)
    expect(surface).toMatch(/panel-min-w-md|panel-max-w-md/)
    expect(surface).not.toMatch(/document\.querySelector/)
    expect(surface).not.toMatch(/\.style\.setProperty/)
    expect(surface).not.toMatch(/action=/)
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
    expect(surface).toMatch(/tasks-surface/)
    expect(surface).toMatch(/surface="tasks"/)
    expect(surface).toMatch(/threads-list/)
    expect(surface).toMatch(/tasks-table/)
    expect(surface).toMatch(/task-detail/)
    expect(surface).toMatch(/task-conversation/)
    expect(surface).toMatch(/anchor-card/)
    expect(surface).toMatch(/filter-actor/)
    expect(surface).toMatch(/status-badge/)
    expect(surface).toMatch(/retry-tasks|Retry/)
    expect(surface).toMatch(/<sail-composer\b/)
    expect(surface).toMatch(/<sail-message-list\b/)
    expect(surface).not.toMatch(/commit-intent/)
    expect(surface).not.toMatch(/>\s*Commit\s*</)
    expect(surface).not.toMatch(/Re-link/)
    expect(surface).not.toMatch(/Crops from the tray/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
    expect(surface).toMatch(/<empty-state\b/)
    expect(surface).toMatch(/tasks-workbench/)
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