/**
 * Theme overview — editorial + docked specimen + design store wiring.
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
  'apps/studio/src/compositions/theme-overview-surface/theme-overview-surface-vm.ts',
)
const EDIT = join(
  REPO,
  'apps/studio/src/compositions/theme-overview-surface/theme-overview-edit.ts',
)
const SESSION = join(REPO, 'apps/studio/src/compositions/theme-session/theme-session.ts')
const NAV = join(REPO, 'apps/studio/src/compositions/theme-session/theme-nav.ts')
const CHROME = join(
  REPO,
  'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail',
)
const PAGE = join(REPO, 'apps/studio/routes/themes/[id]/page.sail')
const THEMES_VM = join(REPO, 'apps/studio/src/compositions/themes-surface/themes-surface-vm.ts')
const ROUTES_GEN = join(REPO, 'apps/studio/routes.gen.ts')

describe('theme-overview-surface', () => {
  it('surface + route files exist with composition markers', () => {
    expect(existsSync(SURFACE)).toBe(true)
    expect(existsSync(VM)).toBe(true)
    expect(existsSync(PAGE)).toBe(true)
    expect(existsSync(SESSION)).toBe(true)
    expect(existsSync(CHROME)).toBe(true)
    const sail = readFileSync(SURFACE, 'utf8')
    const page = readFileSync(PAGE, 'utf8')
    const chrome = readFileSync(CHROME, 'utf8')
    expect(sail).toMatch(/theme-overview-surface/)
    expect(sail).toMatch(/surface="themes"/)
    expect(sail).toMatch(/overview-chrome/)
    expect(sail).toMatch(/<theme-facet-chrome\b/)
    expect(sail).toMatch(/createThemeSession/)
    expect(sail).toMatch(/createOverviewEdit/)
    expect(sail).toMatch(/facet-doors/)
    expect(sail).toMatch(/identity-column/)
    expect(sail).toMatch(/specimen-dock/)
    expect(sail).toMatch(/overview-specimen|preview-specimen/)
    expect(sail).toMatch(/color-hero|hero-colors/)
    expect(sail).toMatch(/editorial-split|overview-workbench/)
    expect(sail).toMatch(/watchPreviewHost/)
    expect(sail).toMatch(/mode=\{/)
    expect(sail).toMatch(/variant="solid"\s+tone="primary"/)
    expect(sail).toMatch(/variant="solid"\s+tone="accent"/)
    expect(sail).toMatch(/<toast-region\b/)
    expect(sail).toMatch(/overview-toast/)
    expect(chrome).toMatch(/theme-facet-chrome/)
    expect(chrome).toMatch(/mode-switch/)
    expect(page).toMatch(/<theme-overview-surface\b/)
  })

  it('overview edit domain uses design store module', () => {
    const edit = readFileSync(EDIT, 'utf8')
    expect(edit).toMatch(/@ship\/studio\/design/)
  })

  it('session + domain use design store ops + toast + mode-aware paint', () => {
    const session = readFileSync(SESSION, 'utf8')
    const actions = readFileSync(
      join(REPO, 'apps/studio/src/compositions/theme-session/theme-session-actions.ts'),
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
    expect(session).toMatch(/modesForPaintId|resolvePaintId/)
    expect(edit).toMatch(/setBrandColor/)
    expect(edit).toMatch(/setTokenValue/)
    expect(edit).toMatch(/setTypeRole/)
    expect(edit).toMatch(/setIconPack/)
    expect(edit).toMatch(/setDesignDensity/)
    expect(nav).toMatch(/PATH_NAV/)
    expect(nav).toMatch(/assets/)
    expect(nav).toMatch(/components/)
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
    const shellRail = readFileSync(join(REPO, 'apps/studio/src/compositions/shell/shell-rail.ts'), 'utf8')
    expect(shellRail).toMatch(/p === '\/themes' \|\| p\.startsWith\('\/themes\/'\)/)
  })

  it('fluid specimen dock uses panel size tokens (no fixed w prison)', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    expect(sail).toMatch(/overlay-max-w-lg|maxW\(overlay-max-w-lg\)/)
  })

  it('hero swatch hit targets + leave guard', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const nav = readFileSync(NAV, 'utf8')
    const session = readFileSync(SESSION, 'utf8')
    // Hero color pickers need relative shells for absolute-fill hit targets
    expect(sail).toMatch(/position\(relative\)/)
    expect(sail).toMatch(/position\(absolute\)/)
    expect(sail).toMatch(/use:swatch=/)
    expect(sail).toMatch(/stateEdit|stateLocked/)
    expect(session).toMatch(/leaveOpen|onLeaveSave|createLeaveGuard/)
    expect(sail).toMatch(/watchPreviewHost/)
  })
})
