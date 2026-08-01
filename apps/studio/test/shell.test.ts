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
    // Signal switching deleted from shell (rail uses links + click→navigate, not setActivity)
    expect(shellVm).not.toMatch(/setActivity/)
    expect(shellVm).not.toMatch(/isEdit|isChecks|isDesign|isTasks/)
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

  it('STUDIO-0 hotkeys: digit map, rail labels, save gate, status markers', () => {
    expect(ACTIVITY_BY_DIGIT).toEqual({
      '1': 'edit',
      '2': 'checks',
      '3': 'design',
      '4': 'tasks',
    })
    const mod = hotkeyMod()
    expect(mod === '⌘' || mod === 'Ctrl+').toBe(true)
    expect(railLabel('edit', 'Ctrl+')).toBe('Edit Ctrl+1')
    expect(railLabel('checks', '⌘')).toBe('Checks ⌘2')
    const rows = railRowsForPath('/design', 'Ctrl+')
    expect(rows.find((r) => r.id === 'design')?.label).toBe('Design Ctrl+3')
    expect(rows.find((r) => r.id === 'design')?.active).toBe(true)

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

    // Rail width: one consistent token on the activity-bar host
    expect(activityBar).toMatch(/data-studio-activity-bar[\s\S]*\bw\(12\)/)
    expect(activityBar).not.toMatch(/\bw\(1[0-1]\)|\bw\(1[3-9]\)/)
    // Brand zone h(8) aligns with sidebar / edit tab headers
    expect(activityBar).toMatch(/h\(8\)/)
    expect(activityBar).toMatch(/minH\(0\)/)
    expect(activityBar).toMatch(/h\(full\)/)
    expect(activityBar).toMatch(/aria-label="Studio activities"/)
    // No padY on rail host (would desync brand from h(8) header row)
    expect(activityBar).not.toMatch(/padY\(/)

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
    expect(layout).toMatch(/<sail-studio-chrome/)
    // Status fixed height; batch tray fixed when present (no content-driven chrome resize)
    expect(chrome).toMatch(/data-studio-status[\s\S]*h\(6\)/)
    expect(chrome).toMatch(/data-batch-tray[\s\S]*h\(7\)/)
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
    expect(surface).toMatch(/data-edit-tab-list/)
    expect(surface).toMatch(/data-tab-id/)
    expect(surface).toMatch(/data-tab-close|showClose/)
    expect(surface).toMatch(/overflowX\(auto\)/)
    expect(surface).toMatch(/onTabMiddle|on:mousedown=\{onTabMiddle\}/)
    expect(surface).toMatch(/Open a file from the Explorer/)
    // Controller factory (explicit binding map is OK for surface wiring)
    // Structured-edit rail hidden until real ops (no decorative dead buttons)
    expect(surface).not.toMatch(/data-struct-op|data-edit-rail|data-structured-edit/)
    expect(surface).toMatch(/data-preview-error|data-preview-retry|hasPreviewError/)
    expect(surfaceVm).toMatch(/onRetryPreview|retryPreviewOnce/)
    const tabsMod = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-tabs.ts'),
      'utf8',
    )
    expect(tabsMod).toMatch(/showClose/)
    expect(surfaceVm).toMatch(/mapDocTabs|onTabMiddle/)
    expect(surface).toMatch(/data-view-panel="preview"/)
    expect(surface).toMatch(/data-view-panel="source"/)
    expect(surface).toMatch(/data-view-panel="state"/)
    expect(surface).toMatch(/data-view-panel="layout"/)
    expect(surface).toMatch(/data-view-panel="data"/)
    expect(surface).toMatch(/data-view-panel="a11y"/)
    expect(surface).toMatch(/data-view-switcher/)
    expect(surface).toMatch(/data-region-overlay/)
    expect(surface).toMatch(/data-source-editor/)
    expect(surface).toMatch(/sail-json-view/)
    // Single live iframe (not exclusive isPreview/!isPreview remount pair)
    expect(surface).toMatch(/data-live-preview-host/)
    expect(surface).toMatch(/hasLivePreview\(\)/)
    expect(surface).not.toMatch(/hasLivePreview\(\)\s*&&\s*!isPreview/)
    expect(surface).not.toMatch(/hasLivePreview\(\)\s*&&\s*isPreview/)
    // iframe attribute only (exclude data-live-preview-host)
    expect((surface.match(/data-live-preview(?!-host)/g) ?? []).length).toBe(1)
    // C2: live dual-pane dead — host stays collapsed (no h(48) expand)
    expect(surface).not.toMatch(/style\.h\(48\)/)
    expect(surface).toMatch(/data-preview-host/)
    expect(surface).toMatch(/data-preview-toolbar/)
    expect(surface).toMatch(/data-preview-ready|isPreviewReady/)
    expect(surface).toMatch(/data-preview-loading|isPreviewLoading/)
    expect(surface).toMatch(/data-preview-empty|isPreviewEmpty/)
    expect(surface).toMatch(/data-preview-error/)
    expect(surface).toMatch(/data-provenance/)
    expect(surface).not.toMatch(/toggle-group|onViewport|viewport picker/i)
    expect(surfaceVm).toMatch(/closeDocTab|openShellTabDocument/)
    expect(surfaceVm).toMatch(/setView|onViewChange/)
    expect(surfaceVm).toMatch(/writeSignalValue|onSignalWrite/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
  })

  it('C2 preview pane: static host state cycle + region overlay, no live dual-pane', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface.sail'),
      'utf8',
    )
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-vm.ts'),
      'utf8',
    )
    const helpers = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/preview-helpers.ts'),
      'utf8',
    )
    const overlay = readFileSync(
      join(REPO, 'apps/studio/src/edit/region-overlay.ts'),
      'utf8',
    )
    // Dense labeled host + full state cycle
    expect(surface).toMatch(/data-preview-host/)
    expect(surface).toMatch(/data-preview-toolbar/)
    expect(surface).toMatch(/data-preview-stage/)
    expect(surface).toMatch(/data-preview-ready/)
    expect(surface).toMatch(/data-preview-loading/)
    expect(surface).toMatch(/data-preview-empty/)
    expect(surface).toMatch(/data-preview-error/)
    expect(surface).toMatch(/Static render/)
    expect(surface).toMatch(/data-preview-provenance|previewProvenance/)
    expect(surface).toMatch(/data-preview-regions|previewRegionsLabel/)
    expect(surface).toMatch(/data-region-overlay/)
    expect(surface).toMatch(/data-region-tooltip|hasHover/)
    expect(surface).toMatch(/data-preview-img/)
    // Fixed toolbar height — no thrash with C1 tabs strip (h8)
    expect(surface).toMatch(/data-preview-toolbar[\s\S]*?h\(7\)/)
    expect(surface).toMatch(/data-edit-tabs[\s\S]*?h\(8\)|data-studio-tabs[\s\S]*?h\(8\)/)
    // Live bridge hidden, never expands as second pane
    expect(surface).toMatch(/data-live-preview-host/)
    expect(surface).toMatch(/maxH\(0\)|h\(0\)\.minH\(0\)\.maxH\(0\)/)
    expect(surface).not.toMatch(/when\(\{\s*current:\s*style\.h\(48\)/)
    // VM state signals for empty/loading/error/ready
    expect(surfaceVm).toMatch(/isPreviewReady/)
    expect(surfaceVm).toMatch(/isPreviewEmpty/)
    expect(surfaceVm).toMatch(/isPreviewLoading/)
    expect(surfaceVm).toMatch(/hasPreviewError/)
    expect(surfaceVm).toMatch(/regionMetaLabel|previewRegionsLabel/)
    expect(helpers).toMatch(/regionMetaLabel/)
    // Region overlay: hover vs selected paint
    expect(overlay).toMatch(/hover/)
    expect(overlay).toMatch(/selected/)
    expect(helpers).toMatch(/hover:\s*r\.nodeId === hover/)
    expect(helpers).toMatch(/selected:\s*r\.nodeId === selected/)
  })

  it('C3 source pane: CodeMirror host density + ready/empty/dirty + save path', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface.sail'),
      'utf8',
    )
    const surfaceVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-surface/edit-surface-vm.ts'),
      'utf8',
    )
    const sailMode = readFileSync(join(REPO, 'apps/studio/src/cm/sail-mode.ts'), 'utf8')
    // Dense labeled host + state cycle markers
    expect(surface).toMatch(/data-source-host/)
    expect(surface).toMatch(/data-source-toolbar/)
    expect(surface).toMatch(/data-source-label/)
    expect(surface).toMatch(/>Source</)
    expect(surface).toMatch(/data-source-status|sourceStatus/)
    expect(surface).toMatch(/data-source-ready|isSourceReady/)
    expect(surface).toMatch(/data-source-empty|isSourceEmpty/)
    expect(surface).toMatch(/data-source-dirty|sourceDirty/)
    expect(surface).toMatch(/data-source-line|sourceScrollLine/)
    expect(surface).toMatch(/data-source-editor/)
    expect(surface).toMatch(/language="sail"/)
    expect(surface).toMatch(/line-numbers/)
    expect(surface).toMatch(/size="sm"/)
    expect(surface).toMatch(/chrome="flush"/)
    // Fixed toolbar height — same h(7) rhythm as C2 preview chrome
    expect(surface).toMatch(/data-source-toolbar[\s\S]*?h\(7\)/)
    // Save path: strip button + CM Mod-s event
    expect(surface).toMatch(/data-save-source/)
    expect(surface).toMatch(/on:sail-save=\{onSaveSource\}|on:sail-save=\{onSaveSource/)
    expect(surface).toMatch(/on:sail-change=\{onSourceSailChange\}/)
    // No structured-edit rail stubs
    expect(surface).not.toMatch(/data-struct-op|data-edit-rail|data-structured-edit/)
    // VM: ready/empty, dirty, scroll, diagnostics push, save
    expect(surfaceVm).toMatch(/isSourceReady/)
    expect(surfaceVm).toMatch(/isSourceEmpty/)
    expect(surfaceVm).toMatch(/sourceDirty/)
    expect(surfaceVm).toMatch(/pushSourceDiagnostics|setDiagnostics/)
    expect(surfaceVm).toMatch(/trySourceScroll|scrollSourceEditor/)
    expect(surfaceVm).toMatch(/onSaveSource|saveActiveSource/)
    expect(surfaceVm).toMatch(/setSourceTextFromEditor/)
    // Sail mode tokenizer present for CM
    expect(sailMode).toMatch(/tokenizeSail|sailStreamTokenizer/)
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
    const explorerVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer-vm.ts'),
      'utf8',
    )
    const treeVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-element-tree/edit-element-tree-vm.ts'),
      'utf8',
    )
    expect(explorer).toMatch(/data-edit-explorer/)
    expect(explorer).toMatch(/data-file-tree/)
    expect(explorer).toMatch(/Explorer/)
    // Header chrome lives on studio-sidebar (uppercase title); explorer composes it.
    expect(explorer).toMatch(/<studio-sidebar\b/)
    expect(explorer).toMatch(/title="Explorer"/)
    expect(explorer).toMatch(/<sail-tree\b/)
    expect(explorer).toMatch(/<edit-element-tree\b/)
    expect(explorer).toMatch(/No files|emptyTitle/)
    expect(explorerVm).toMatch(/createTable/)
    expect(tree).toMatch(/data-element-tree/)
    expect(tree).toMatch(/<sail-tree\b/)
    expect(tree).toMatch(/layerBadge|data-layer-badge/)
    expect(treeVm).toMatch(/createTable/)
    expect(explorer).not.toMatch(/<div\b/)
    expect(tree).not.toMatch(/<div\b/)
  })

  it('R1: edit-explorer rail tabs Routes/Files/Components discipline', () => {
    const explorer = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer.sail'),
      'utf8',
    )
    const explorerVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer-vm.ts'),
      'utf8',
    )
    // L1 tabs: pill + size sm + full width; clear labels
    expect(explorer).toMatch(/data-rail-tabs/)
    expect(explorer).toMatch(/<tabs\b[^>]*size="sm"/)
    expect(explorer).toMatch(/<tab-list\b[^>]*variant="pill"/)
    expect(explorer).toMatch(/name="routes"[^>]*>Routes</)
    expect(explorer).toMatch(/name="files"[^>]*>Files</)
    expect(explorer).toMatch(/name="components"[^>]*>Components</)
    expect(explorer).toMatch(/on:sail-tab-change=\{onRailTabChange\}/)
    // One content host per surface — tree vs components panels, not stacked if-only
    expect(explorer).toMatch(/data-rail-panel="tree"/)
    expect(explorer).toMatch(/data-rail-panel="components"/)
    expect(explorer).toMatch(/data-components-tab/)
    expect(explorer).toMatch(/data-file-tree-host/)
    // Components stays mounted (aria-current height toggle — no remount thrash)
    expect(explorer).toMatch(/when\(\{\s*current:/)
    expect(explorer).toMatch(/aria-current=\{isTreeView\(\)/)
    expect(explorer).toMatch(/aria-current=\{isComponents\(\)/)
    // Filter only on tree views
    expect(explorer).toMatch(/isTreeView\(\)/)
    expect(explorer).toMatch(/data-edit-filter/)
    // Token spacing only (pad/gap scale) — no magic px in explorer chrome
    expect(explorer).not.toMatch(/padX\(\d+px\)|gap:\s*\d+px|style\s*=\s*["'][^"']*px/)
    // VM: railTab + routes/files data swap + collapse-safe meta select
    expect(explorerVm).toMatch(/railTab/)
    expect(explorerVm).toMatch(/'routes'\s*\|\s*'files'\s*\|\s*'components'/)
    expect(explorerVm).toMatch(/routeSailTreeNodes/)
    expect(explorerVm).toMatch(/fileSailTreeNodes/)
    expect(explorerVm).toMatch(/onRailTabChange/)
    expect(explorerVm).toMatch(/isTreeMetaId|startsWith\('group:'\)|startsWith\('route:'\)/)
    // R3: dir intermediates from nested Files tree are expand-only (with group/route)
    expect(explorerVm).toMatch(/startsWith\('dir:'\)/)
    expect(explorerVm).toMatch(/assignTreeHost/)
  })

  it('R3: Files tree uses trailing dirty + nested paths (not label cram)', () => {
    const explorer = readFileSync(join(REPO, 'apps/studio/src/edit/explorer.ts'), 'utf8')
    const treeSail = readFileSync(
      join(REPO, 'packages/components/sail/primitives/src/sail-tree/sail-tree.sail'),
      'utf8',
    )
    const treeView = readFileSync(
      join(REPO, 'packages/components/sail/primitives/src/sail-tree/sail-tree.view.ts'),
      'utf8',
    )
    // Files path uses fileSegToSailTree + trailing dirty (R2 routes may still ●-prefix labels)
    expect(explorer).toMatch(/function fileSegToSailTree/)
    expect(explorer).toMatch(/trailing:\s*dirty\?\.has\(node\.filePath\)\s*\?\s*'●'/)
    expect(explorer).toMatch(/id:\s*`dir:\$\{groupId\}\/\$\{rel\}`/)
    expect(explorer).toMatch(/export function fileSailTreeNodes/)
    // L2 trail slot renders node.trailing
    expect(treeView).toMatch(/trailing:/)
    expect(treeSail).toMatch(/w\.trailing/)
    expect(treeSail).toMatch(/node-trailing/)
  })

  it('R4: Components element tree — trailing layer badges, click → source', () => {
    const explorer = readFileSync(join(REPO, 'apps/studio/src/edit/explorer.ts'), 'utf8')
    const treeSail = readFileSync(
      join(REPO, 'apps/studio/src/edit-element-tree/edit-element-tree.sail'),
      'utf8',
    )
    const treeVm = readFileSync(
      join(REPO, 'apps/studio/src/edit-element-tree/edit-element-tree-vm.ts'),
      'utf8',
    )
    const host = readFileSync(
      join(REPO, 'apps/studio/src/edit-explorer/edit-explorer.sail'),
      'utf8',
    )
    const primTree = readFileSync(
      join(REPO, 'packages/components/sail/primitives/src/sail-tree/sail-tree.sail'),
      'utf8',
    )
    // Builder: badges in trailing, not label cram
    expect(explorer).toMatch(/export function elementSailTreeNodes/)
    expect(explorer).toMatch(/trailing:\s*badge/)
    expect(explorer).not.toMatch(/label:\s*badge\s*\?\s*`\$\{base\}/)
    // Chrome: Components tab host, exclusive height, density sm, layer marker
    expect(host).toMatch(/data-rail-panel="components"/)
    expect(host).toMatch(/<edit-element-tree\b/)
    expect(treeSail).toMatch(/data-element-tree/)
    expect(treeSail).toMatch(/data-components-tree/)
    expect(treeSail).toMatch(/data-layer-badge/)
    expect(treeSail).toMatch(/size="sm"/)
    expect(treeSail).toMatch(/label="Component tree"/)
    expect(treeSail).toMatch(/Components/)
    expect(treeSail).toMatch(/data-elements-empty|No elements yet/)
    // VM: select → source view + span scroll
    expect(treeVm).toMatch(/elementSailTreeNodes/)
    expect(treeVm).toMatch(/selectEditNode/)
    expect(treeVm).toMatch(/setView\(\s*['"]source['"]\s*\)/)
    expect(treeVm).toMatch(/scrollSourceEditor/)
    expect(treeVm).toMatch(/assignTreeHost\('\s*\[data-element-tree-host\]/)
    // L2 trail: dirty ● vs text badge (layer) styling
    expect(primTree).toMatch(/w\.trailing === '●'|trailing === '●'/)
    expect(primTree).toMatch(/muted-foreground/)
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
    expect(surface).toMatch(/Open in editor/)
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
    expect(surface).toMatch(/data-design-workbench/)
    expect(surface).toMatch(/data-primary-pane/)
    expect(surface).toMatch(/data-specimen-pane/)
    expect(surface).toMatch(/<design-inspector\b/)
    expect(surface).toMatch(/controller\(\(\)\s*=>\s*createDesignSurface\(\)\)/)
    expect(surface).not.toMatch(/w="28"/)
    expect(inspector).toMatch(/data-design-inspector/)
    expect(inspector).toMatch(/data-specimen-live|data-specimen-canvas/)
    expect(inspector).toMatch(/controller\(\(\)\s*=>\s*createDesignInspector\(\)\)/)
    expect(combined).not.toMatch(/<div\b/)
    expect(combined).not.toMatch(/<span\b/)
    expect(combined).toMatch(/<empty-state\b/)
    expect(combined).toMatch(/<scroll-area\b/)
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