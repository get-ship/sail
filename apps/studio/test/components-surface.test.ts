/**
 * Components surface — ban-clean + composition + shell wiring markers.
 */
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SURFACE = join(REPO, 'apps/studio/src/components-surface/components-surface.sail')
const INSPECT_SURFACE = join(
  REPO,
  'apps/studio/src/components-inspect-surface/components-inspect-surface.sail',
)
const SESSION = join(REPO, 'apps/studio/src/components-session/components-session.ts')
const LIBRARY = join(REPO, 'apps/studio/src/components-library/components-library.ts')
const INSPECT = join(REPO, 'apps/studio/src/components-inspect/components-inspect.ts')
const CATALOG = join(REPO, 'apps/studio/src/components/catalog.ts')
const BRIEF = join(REPO, 'apps/studio/src/components/brief.ts')
const PAGE = join(REPO, 'apps/studio/routes/components/page.sail')
const PAGE_ID = join(REPO, 'apps/studio/routes/components/[id]/page.sail')
const PAGE_INSPECT = join(REPO, 'apps/studio/routes/components/inspect/page.sail')

describe('components-surface', () => {
  it('route + surface + domain modules exist', () => {
    for (const p of [
      SURFACE,
      INSPECT_SURFACE,
      SESSION,
      LIBRARY,
      INSPECT,
      CATALOG,
      BRIEF,
      PAGE,
      PAGE_ID,
      PAGE_INSPECT,
    ]) {
      expect(existsSync(p), p).toBe(true)
    }
    const page = readFileSync(PAGE, 'utf8')
    expect(page).toMatch(/<components-surface\b/)
    expect(readFileSync(PAGE_INSPECT, 'utf8')).toMatch(/<components-inspect-surface\b/)
  })

  it('library + inspect surfaces ban-clean markers', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const inspectSail = readFileSync(INSPECT_SURFACE, 'utf8')
    expect(sail).toMatch(/components-surface/)
    expect(sail).toMatch(/surface="components"/)
    expect(sail).toMatch(/components-library/)
    expect(sail).toMatch(/mode="library"/)
    expect(sail).toMatch(/mode="inspect"/)
    expect(sail).toMatch(/This project/)
    expect(sail).toMatch(/Installed packs/)
    expect(sail).toMatch(/Open in Edit/)
    expect(sail).toMatch(/Start thread/)
    expect(sail).toMatch(/Copy agent brief/)
    expect(sail).toMatch(/new-component|New…/)
    expect(sail).toMatch(/components-preview/)
    expect(sail).toMatch(/l1-footnote|Themes → Components/)
    expect(sail).toMatch(/createComponentsSession/)
    expect(sail).toMatch(/createComponentsLibrary/)
    expect(sail).toMatch(/return \{ session, library \}/)
    expect(sail).toMatch(/<toast-region\b/)
    expect(sail).toMatch(/panel-min-w-md|panel-max-w-md/)
    expect(inspectSail).toMatch(/components-inspect/)
    expect(inspectSail).toMatch(/data-inspect-tree|sail-tree/)
    expect(inspectSail).toMatch(/surface-picker/)
    expect(inspectSail).toMatch(/Select a surface to inspect/)
    expect(inspectSail).toMatch(/createComponentsInspect/)
    expect(inspectSail).toMatch(/return \{ session, inspect \}/)
    for (const src of [sail, inspectSail]) {
      expect(src).not.toMatch(/<div\b/)
      expect(src).not.toMatch(/<span\b/)
      expect(src).not.toMatch(/document\.querySelector/)
      expect(src).not.toMatch(/\.style\.setProperty/)
      expect(src).not.toMatch(/theme-facet-chrome/)
      expect(src).not.toMatch(/theme-components-surface/)
      expect(src).not.toMatch(/L1_CATALOG/)
      expect(src).not.toMatch(/#[0-9a-fA-F]{3,8}\b/)
    }
  })

  it('session uses openDocument + createThread + generation-safe domains', () => {
    const session = readFileSync(SESSION, 'utf8')
    const library = readFileSync(LIBRARY, 'utf8')
    const inspect = readFileSync(INSPECT, 'utf8')
    expect(session).toMatch(/openDocument/)
    expect(session).toMatch(/if\s*\(\s*!doc\s*\)/)
    expect(session).toMatch(/createThread/)
    expect(session).toMatch(/buildAgentBrief/)
    expect(session).toMatch(/requestSourceScroll/)
    expect(session).toMatch(/writeComponentsActivity|readComponentsActivity/)
    expect(session).not.toMatch(/listeners\s*=\s*new Set/)
    expect(library).toMatch(/loadGen/)
    expect(library).toMatch(/if \(gen !== loadGen\)/)
    expect(library).toMatch(/mergeProjectPathAuthority|unionFromEditPaths/)
    expect(library).toMatch(/COMPONENTS_CATALOG/)
    expect(library).toMatch(/recordsFromCatalog|recordsFromProjectPaths/)
    expect(library).toMatch(/finally/)
    expect(inspect).toMatch(/loadGen/)
    expect(inspect).toMatch(/if \(gen !== loadGen\)/)
    expect(inspect).toMatch(/loadGen \+= 1/)
    expect(inspect).toMatch(/createTable/)
    expect(inspect).toMatch(/elementSailTreeNodes|sailInspectTree/)
    expect(inspect).toMatch(/knownLibraryIds/)
    expect(inspect).not.toMatch(/listeners\s*=\s*new Set/)
    expect(library).not.toMatch(/listeners\s*=\s*new Set/)
  })

  it('line budgets roughly honored', () => {
    for (const [rel, max] of [
      ['apps/studio/src/components-session/components-session.ts', 400],
      ['apps/studio/src/components-library/components-library.ts', 400],
      ['apps/studio/src/components-inspect/components-inspect.ts', 480],
      ['apps/studio/src/components/catalog.ts', 250],
      ['apps/studio/src/components/brief.ts', 100],
    ] as const) {
      const src = readFileSync(join(REPO, rel), 'utf8')
      expect(src.split('\n').length, rel).toBeLessThanOrEqual(max)
    }
  })

  it('shell rail + routes include components', () => {
    const shellRail = readFileSync(join(REPO, 'apps/studio/src/shell/shell-rail.ts'), 'utf8')
    const shellVm = readFileSync(join(REPO, 'apps/studio/src/shell/shell-page-vm.ts'), 'utf8')
    const routesGen = readFileSync(join(REPO, 'apps/studio/routes.gen.ts'), 'utf8')
    const model = readFileSync(
      join(REPO, 'packages/sail/studio-shell/src/model.ts'),
      'utf8',
    )
    expect(shellRail).toMatch(/'2':\s*'components'/)
    expect(shellRail).toMatch(/components:\s*\{\s*label:\s*'Components'/)
    expect(shellRail).toMatch(/icon:\s*'box'/)
    expect(shellVm).toMatch(/shell-rail/)
    expect(shellVm).toMatch(/ACTIVITY_BY_DIGIT/)
    expect(routesGen).toMatch(/pattern: '\/components'/)
    expect(routesGen).toMatch(/pattern: '\/components\/inspect'/)
    expect(routesGen).toMatch(/pattern: '\/components\/:id'/)
    expect(model).toMatch(/'components'/)
    expect(model).toMatch(/CoreActivityId/)
  })
})
