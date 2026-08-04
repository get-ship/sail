/**
 * Theme shape facet — radius / elevation / border / glass + specimen-first layout.
 * Wave 2: mock re-port + review fixes (mock ramp, glass card α, honesty).
 */
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SURFACE = join(
  REPO,
  'apps/studio/src/theme-shape-surface/theme-shape-surface.sail',
)
const VM = join(
  REPO,
  'apps/studio/src/theme-shape-surface/theme-shape-surface-vm.ts',
)
const EDIT = join(REPO, 'apps/studio/src/theme-shape-surface/theme-shape-edit.ts')
const RECIPES = join(REPO, 'apps/studio/src/theme-shape-surface/theme-shape-recipes.ts')
const SESSION = join(REPO, 'apps/studio/src/theme-session/theme-session.ts')
const PAGE = join(REPO, 'apps/studio/routes/themes/[id]/shape/page.sail')
const ROUTES_GEN = join(REPO, 'apps/studio/routes.gen.ts')

describe('theme-shape-surface', () => {
  it('surface + route files exist with facet markers', () => {
    expect(existsSync(SURFACE)).toBe(true)
    expect(existsSync(VM)).toBe(true)
    expect(existsSync(PAGE)).toBe(true)
    const sail = readFileSync(SURFACE, 'utf8')
    const page = readFileSync(PAGE, 'utf8')
    expect(sail).toMatch(/data-theme-shape-surface/)
    expect(sail).toMatch(/data-surface="themes"/)
    expect(sail).toMatch(/data-shape-chrome/)
    expect(sail).toMatch(/<theme-facet-chrome\b/)
    expect(sail).toMatch(/data-shape-controls/)
    expect(sail).toMatch(/data-specimen-dock|data-shape-canvas/)
    expect(sail).toMatch(/data-shape-specimen|data-preview-specimen/)
    expect(sail).toMatch(/data-section="recipes"/)
    expect(sail).toMatch(/data-section="radius"/)
    expect(sail).toMatch(/data-section="shadow"/)
    expect(sail).toMatch(/data-section="border"/)
    expect(sail).toMatch(/data-radius-chips/)
    expect(sail).toMatch(/data-radius-master/)
    expect(sail).toMatch(/data-elev-chips/)
    expect(sail).toMatch(/data-radius-ladder/)
    // Draft paint = applyScope bind; no load-bearing theme on facet host (D7/Job 3).
    expect(sail).not.toMatch(/data-theme-preview=\{/)
    expect(sail).toMatch(/watchPreviewHost/)
    expect(sail).toMatch(/data-mode=\{/)
    expect(sail).toMatch(/createThemeSession|createShapeEdit/)
    expect(sail).toMatch(/variant="solid"\s+tone="primary"/)
    expect(sail).toMatch(/data-radius="sm"/)
    expect(sail).toMatch(/variant="outline"/)
    expect(sail).toMatch(/<toast-region\b/)
    // Radius chips must stay outline (never solid color blobs).
    expect(sail).not.toMatch(/data-radius="[^"]+"[\s\S]{0,120}variant="solid"/)
    expect(sail).not.toMatch(/document\.querySelector/)
    expect(sail).not.toMatch(/\.style\.setProperty/)
    expect(sail).not.toMatch(/data-action=/)
    expect(sail).not.toMatch(/theme\.preview\s*\(/)
    expect(sail).not.toMatch(/escape hatch|power tooling|power-path|power path/i)
    expect(page).toMatch(/<theme-shape-surface\b/)
  })

  it('Wave 2: specimen-first IA + outline radius + draft paint host', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const edit = readFileSync(EDIT, 'utf8')
    const recipes = readFileSync(RECIPES, 'utf8')
    expect(sail).toMatch(/data-shape-workbench/)
    expect(sail).toMatch(/panel-max-w-sm|maxW\(panel-max-w-sm\)/)
    expect(sail).toMatch(/data-shape-canvas/)
    expect(recipes).toMatch(/soft:|sharp:|glass:|flat:|elevated:/)
    expect(recipes).toMatch(/RECIPE_IDS/)
    expect(sail).toMatch(/bg\(transparent\)/)
    expect(sail).toMatch(/data-radius="sm"[\s\S]{0,80}variant="outline"/)
    expect(sail).toMatch(/data-elev=/)
    expect(sail).toMatch(/data-section="glass"|showGlass/)
    expect(sail).toMatch(/data-preview-specimen/)
    expect(sail).not.toMatch(/data-theme-preview=/)
    expect(sail).toMatch(/bg\(background\)/)
    expect(sail).toMatch(/data-spec-card/)
    // No "live" badge theater.
    expect(sail).not.toMatch(/>live</)
    // Draft paint adapter wired on surface host
    expect(sail).toMatch(/watchPreviewHost/)
    expect(edit).toMatch(/liveCardRadius\.set\(px\(lg\)\)/)
    expect(edit).toMatch(/setRadiusFamily|setRadiusStep/)
    expect(edit).toMatch(/setTokenValue/)
  })

  it('review: mock radius ramp Soft 5/10/15/20 and absolute family writes', async () => {
    const edit = readFileSync(EDIT, 'utf8')
    // Absolute family write via setRadiusFamily (single store write — not 4× setRadiusStep).
    expect(edit).toMatch(/writeRadiusFamily|mockRadiusFamily/)
    expect(edit).toMatch(/setRadiusFamily/)
    expect(edit).toMatch(/RADIUS_RAMP|0\.5/)
    // Pure helper unit: Soft master 10 → 5/10/15/20; Sharp master 2 → 1/2/3/4.
    const { mockRadiusFamily, RADIUS_RAMP } = await import(
      '../src/theme-shape-surface/theme-shape-edit'
    )
    expect(RADIUS_RAMP).toEqual({ sm: 0.5, md: 1, lg: 1.5, xl: 2 })
    expect(mockRadiusFamily(10)).toEqual({ sm: 5, md: 10, lg: 15, xl: 20 })
    expect(mockRadiusFamily(2)).toEqual({ sm: 1, md: 2, lg: 3, xl: 4 })
    expect(mockRadiusFamily(8)).toEqual({ sm: 4, md: 8, lg: 12, xl: 16 })
  })

  it('review: glass writes card fill α + backdrop; freeform frost fill', async () => {
    const edit = readFileSync(EDIT, 'utf8')
    const sail = readFileSync(SURFACE, 'utf8')
    // Correct frost channel — not surface-overlay noise α.
    expect(edit).toMatch(/component\.card\.bg/)
    expect(edit).toMatch(/component\.card\.backdrop-filter/)
    expect(edit).not.toMatch(/color\.surface\.overlay-opacity/)
    expect(edit).toMatch(/withColorAlpha|liveCardFill/)
    // Specimen freeform fill (liveBox pack or bind).
    expect(sail).toMatch(/liveCardFill|liveCardBox|use:liveBox/)
    expect(sail).toMatch(/data-glass-mini/)
    expect(sail).not.toMatch(/Live paint after Save until F4/)
    const { withColorAlpha, stripColorAlpha } = await import(
      '../src/theme-shape-surface/theme-shape-edit'
    )
    expect(withColorAlpha('oklch(1 0 0)', 0.72)).toBe('oklch(1 0 0 / 72%)')
    expect(withColorAlpha('oklch(0.99 0.01 240 / 100%)', 0.55)).toBe(
      'oklch(0.99 0.01 240 / 55%)',
    )
    expect(stripColorAlpha('oklch(1 0 0 / 55%)')).toBe('oklch(1 0 0)')
  })

  it('review: stepped elev marks + no dead radiusChips solid helper', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const edit = readFileSync(EDIT, 'utf8')
    expect(sail).toMatch(/elevMarkNone|elevMarkXs|elevMarkXl/)
    expect(sail).toMatch(/shadow\(xs\)/)
    expect(sail).toMatch(/shadow\(xl\)/)
    expect(edit).not.toMatch(/radiusChips/)
    expect(edit).not.toMatch(/btnVariant.*solid/)
    expect(edit).not.toMatch(/onStageBg|stageSolidOn/)
  })

  it('does not dual-path into design-surface / design-inspector UI', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const vm = readFileSync(VM, 'utf8')
    const edit = readFileSync(EDIT, 'utf8')
    expect(sail).not.toMatch(/design-surface|design-inspector/)
    expect(vm).not.toMatch(/design-surface|design-inspector/)
    expect(edit).not.toMatch(/design-surface|design-inspector/)
    expect(edit).toMatch(/@ship\/studio\/design/)
  })

  it('domain uses design store radius/token ops + curated paths', () => {
    const edit = readFileSync(EDIT, 'utf8')
    const session = readFileSync(SESSION, 'utf8')
    const actions = readFileSync(
      join(REPO, 'apps/studio/src/theme-session/theme-session-actions.ts'),
      'utf8',
    )
    expect(session).toMatch(/mountAmbient\s*\(/)
    expect(session).toMatch(/subscribeDesign/)
    expect(session).toMatch(/getDesignState/)
    expect(session).toMatch(/openTheme/)
    expect(edit).toMatch(/setRadiusFamily|setRadiusStep/)
    expect(edit).toMatch(/setTokenValue/)
    expect(actions).toMatch(/requestSaveTheme|saveTheme/)
    expect(edit).toMatch(/shadow\.card-rest/)
    expect(edit).toMatch(/color\.border\.width/)
    expect(edit).toMatch(/component\.card\.bg/)
    expect(edit).toMatch(/component\.card\.backdrop-filter/)
    expect(edit).not.toMatch(/document\.querySelector/)
    expect(edit).not.toMatch(/document\.createElement/)
    // Host draft paint goes through applyScope adapter — not theme.preview(:root).
    expect(edit).not.toMatch(/theme\.preview\s*\(/)
    expect(edit).not.toMatch(/escape hatch|power tooling/i)
  })

  it('session path navigates shape among facets', () => {
    const session = readFileSync(SESSION, 'utf8')
    const actions = readFileSync(
      join(REPO, 'apps/studio/src/theme-session/theme-session-actions.ts'),
      'utf8',
    )
    const nav = readFileSync(join(REPO, 'apps/studio/src/theme-session/theme-nav.ts'), 'utf8')
    expect(nav).toMatch(/shape/)
    expect(nav).toMatch(/color/)
    expect(session).toMatch(/navigateFacet/)
    expect(actions).toMatch(/themeFacetHref|onPathSelect/)
  })

  it('routes.gen includes /themes/:id/shape when regenerated', () => {
    if (!existsSync(ROUTES_GEN)) return
    const gen = readFileSync(ROUTES_GEN, 'utf8')
    if (gen.includes('shape')) {
      expect(gen).toMatch(/\/themes\/:id\/shape|themes\/\[id\]\/shape/)
    }
  })
})
