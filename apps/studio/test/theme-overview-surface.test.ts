/**
 * Theme overview — ban-clean editorial + docked specimen + design store wiring.
 */
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SURFACE = join(
  REPO,
  'apps/studio/src/theme-overview-surface/theme-overview-surface.sail',
)
const VM = join(
  REPO,
  'apps/studio/src/theme-overview-surface/theme-overview-surface-vm.ts',
)
const EDIT = join(
  REPO,
  'apps/studio/src/theme-overview-surface/theme-overview-edit.ts',
)
const SESSION = join(REPO, 'apps/studio/src/theme-session/theme-session.ts')
const NAV = join(REPO, 'apps/studio/src/theme-session/theme-nav.ts')
const CHROME = join(
  REPO,
  'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail',
)
const PAGE = join(REPO, 'apps/studio/routes/themes/[id]/page.sail')
const THEMES_VM = join(REPO, 'apps/studio/src/themes-surface/themes-surface-vm.ts')
const ROUTES_GEN = join(REPO, 'apps/studio/routes.gen.ts')

describe('theme-overview-surface', () => {
  it('surface + route files exist with ban-clean markers', () => {
    expect(existsSync(SURFACE)).toBe(true)
    expect(existsSync(VM)).toBe(true)
    expect(existsSync(PAGE)).toBe(true)
    expect(existsSync(SESSION)).toBe(true)
    expect(existsSync(CHROME)).toBe(true)
    const sail = readFileSync(SURFACE, 'utf8')
    const page = readFileSync(PAGE, 'utf8')
    const chrome = readFileSync(CHROME, 'utf8')
    expect(sail).toMatch(/data-theme-overview-surface/)
    expect(sail).toMatch(/data-surface="themes"/)
    expect(sail).toMatch(/data-overview-chrome/)
    expect(sail).toMatch(/<theme-facet-chrome\b/)
    expect(sail).toMatch(/createThemeSession/)
    expect(sail).toMatch(/createOverviewEdit/)
    expect(sail).not.toMatch(/Single path control|Docked|Live preview/)
    expect(sail).toMatch(/data-facet-doors/)
    expect(sail).toMatch(/data-identity-column/)
    expect(sail).toMatch(/data-specimen-dock/)
    expect(sail).toMatch(/data-overview-specimen|data-preview-specimen/)
    expect(sail).toMatch(/data-color-hero|data-hero-colors/)
    expect(sail).toMatch(/data-editorial-split|data-overview-workbench/)
    // Draft paint = applyScope bind; no load-bearing data-theme-preview on facet host (D7/Job 3).
    expect(sail).not.toMatch(/data-theme-preview=\{/)
    expect(sail).toMatch(/watchPreviewHost/)
    expect(sail).toMatch(/data-mode=\{/)
    expect(sail).toMatch(/variant="solid"\s+tone="primary"/)
    expect(sail).toMatch(/variant="solid"\s+tone="accent"/)
    expect(sail).toMatch(/<toast-region\b/)
    expect(sail).toMatch(/data-overview-toast/)
    expect(sail).not.toMatch(/data-overview-status/)
    expect(sail).not.toMatch(/document\.querySelector/)
    expect(sail).not.toMatch(/\.style\.setProperty/)
    expect(sail).not.toMatch(/data-action=/)
    expect(sail).not.toMatch(/theme\.preview\s*\(/)
    expect(chrome).toMatch(/data-theme-facet-chrome/)
    expect(chrome).toMatch(/data-mode-switch/)
    expect(page).toMatch(/<theme-overview-surface\b/)
  })

  it('does not dual-path into design-surface', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const vm = readFileSync(VM, 'utf8')
    const edit = readFileSync(EDIT, 'utf8')
    expect(sail).not.toMatch(/design-surface|design-inspector/)
    expect(vm).not.toMatch(/design-surface|design-inspector/)
    expect(edit).not.toMatch(/design-surface|design-inspector/)
    expect(edit).toMatch(/@ship\/studio\/design/)
  })

  it('session + domain use design store ops + toast + mode-aware paint', () => {
    const session = readFileSync(SESSION, 'utf8')
    const actions = readFileSync(
      join(REPO, 'apps/studio/src/theme-session/theme-session-actions.ts'),
      'utf8',
    )
    const edit = readFileSync(EDIT, 'utf8')
    const nav = readFileSync(NAV, 'utf8')
    expect(session).toMatch(/mountAmbient\s*\(/)
    expect(session).toMatch(/subscribeDesign/)
    expect(session).toMatch(/getDesignState/)
    expect(session).toMatch(/openTheme/)
    expect(session).toMatch(/loadThemes/)
    expect(actions).toMatch(/addThemeToProject|forkTheme/)
    expect(actions).toMatch(/requestSaveTheme|saveTheme/)
    expect(actions).toMatch(/previewInApp/)
    expect(session).toMatch(/toastItems|onToastDismiss/)
    expect(session).toMatch(/paintIdFor|modesForPaintId|resolvePaintId/)
    expect(edit).toMatch(/setBrandColor/)
    expect(edit).toMatch(/setTokenValue/)
    expect(edit).toMatch(/setTypeRole/)
    expect(edit).toMatch(/setIconPack/)
    expect(edit).toMatch(/setDesignDensity/)
    expect(nav).toMatch(/PATH_NAV/)
    expect(nav).toMatch(/assets/)
    expect(nav).toMatch(/components/)
    expect(session).not.toMatch(/document\.querySelector/)
    expect(session).not.toMatch(/setProperty/)
    expect(session).not.toMatch(/document\.createElement/)
    expect(session).not.toMatch(/theme\.preview/)
  })

  it('Open from themes home navigates to /themes/:id after openTheme', () => {
    const themesVm = readFileSync(THEMES_VM, 'utf8')
    expect(themesVm).toMatch(/navigate/)
    expect(themesVm).toMatch(/openTheme/)
    expect(themesVm).toMatch(/\/themes\/\$\{/)
    expect(themesVm).toMatch(/appRouteBase/)
    expect(themesVm).toMatch(/@ship\/deck\/router/)
  })

  it('routes.gen includes dynamic /themes/:id', () => {
    const gen = readFileSync(ROUTES_GEN, 'utf8')
    expect(gen).toMatch(/\/themes\/:id|\/themes\/\[id\]/)
    expect(gen).toMatch(/themes\/\[id\]\/page\.sail|themes\/\[id\]/)
  })

  it('activityFromPath treats /themes/* as themes', () => {
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/shell/shell-page-vm.ts'), 'utf8')
    expect(shellVm).toMatch(/p === '\/themes' \|\| p\.startsWith\('\/themes\/'\)/)
  })

  it('fluid specimen dock uses panel size tokens (no fixed w prison)', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    expect(sail).toMatch(/panel-min-w-md|minW\(panel-min-w-md\)/)
    expect(sail).toMatch(/panel-max-w-md|maxW\(panel-max-w-md\)/)
    expect(sail).not.toMatch(/\bw="48"|\bw="56"/)
  })

  it('Wave 1: hero swatch hit targets + leave guard + no overflow', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const nav = readFileSync(NAV, 'utf8')
    const session = readFileSync(SESSION, 'utf8')
    // Hero color pickers need relative shells for absolute-fill hit targets
    expect(sail).toMatch(/position\(relative\)/)
    expect(sail).toMatch(/position\(absolute\)/)
    expect(sail).toMatch(/use:swatch=/)
    expect(sail).toMatch(/stateEdit|stateLocked/)
    // Facet overflow removed — delete/remix on Themes home
    expect(nav).not.toMatch(/OVERFLOW_NAV/)
    expect(nav).not.toMatch(/value: 'peek'|value: 'export'/)
    expect(session).not.toMatch(/In project · editable/)
    expect(session).toMatch(/leaveOpen|onLeaveSave|createLeaveGuard/)
    // Draft paint wiring (functional — not Overview UX redesign)
    expect(sail).toMatch(/watchPreviewHost/)
  })
})
