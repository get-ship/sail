/**
 * Config surface — live/project fields + category publish.
 */
import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  buildConfigFields,
  CHECKS_HOTKEY_LABEL,
  CONFIG_CATS,
  createConfigSurfaceCore,
  liveShellChanged,
  type ConfigCatId,
} from '../src/config/config-surface-vm'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')

describe('config-surface', () => {
  it('route + surface modules exist', () => {
    expect(existsSync(join(REPO, 'apps/studio/routes/config/page.sail'))).toBe(true)
    expect(existsSync(join(REPO, 'apps/studio/src/config-surface/config-surface.sail'))).toBe(
      true,
    )
    expect(existsSync(join(REPO, 'apps/studio/src/config/config-surface-vm.ts'))).toBe(true)
  })

  it('surface uses button nav + table fields (no diary chrome)', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/config-surface/config-surface.sail'),
      'utf8',
    )
    expect(surface).toMatch(/data-config-surface/)
    expect(surface).toMatch(/data-config-nav/)
    expect(surface).toMatch(/data-config-cat=/)
    expect(surface).toMatch(/<button\b/)
    expect(surface).toMatch(/<table\b/)
    expect(surface).toMatch(/\bdense\b/)
    expect(surface).toMatch(/data-config-fields/)
    expect(surface).not.toMatch(/L1 table dense/)
    expect(surface).not.toMatch(/honest/)
    expect(surface).not.toMatch(/dogfood/)
    expect(surface).not.toMatch(/scaffold/)
    expect(surface).not.toMatch(/showPlugins/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
  })

  it('buildConfigFields: only live/project/disabled sources', () => {
    const ctx = {
      root: 'apps/studio',
      devUrl: 'http://127.0.0.1:7893',
      moduleCount: 2,
      themeId: 'studio',
      modeId: 'dark',
    }
    const project = buildConfigFields('project', ctx)
    expect(project.find((f) => f.key === 'root')?.source).toBe('project')
    expect(project.find((f) => f.key === 'theme')?.value).toBe('studio')
    expect(project.every((f) => f.source === 'project' || f.source === 'live')).toBe(true)

    const runtime = buildConfigFields('runtime', ctx)
    expect(runtime.find((f) => f.key === 'dev')?.source).toBe('live')
    expect(runtime.find((f) => f.key === 'findings-hotkey')?.value).toBe(CHECKS_HOTKEY_LABEL)
    expect(runtime.every((f) => f.source === 'live')).toBe(true)

    const danger = buildConfigFields('danger', ctx)
    expect(danger.every((f) => f.source === 'disabled')).toBe(true)

    // No invented fixture compiler knobs
    for (const id of CONFIG_CATS.map((c) => c.id) as ConfigCatId[]) {
      const rows = buildConfigFields(id, ctx)
      expect(rows.some((f) => f.source === 'fixture')).toBe(false)
    }
  })

  it('createConfigSurfaceCore defaults to project; selectCategory flips flags', () => {
    const c = createConfigSurfaceCore()
    expect(c.selected()).toBe('project')
    expect(c.sectionTitle()).toBe('Project')
    expect(c.showShipCloud()).toBe(false)
    expect(c.showDanger()).toBe(false)
    expect(c.categories()).toHaveLength(CONFIG_CATS.length)

    c.selectCategory('plugins')
    expect(c.selected()).toBe('plugins')
    expect(c.showShipCloud()).toBe(true)

    c.selectCategory('danger')
    expect(c.showDanger()).toBe(true)
    expect(c.fields().every((f) => f.source === 'disabled')).toBe(true)
  })

  it('onCatClick reads data-config-cat', () => {
    const c = createConfigSurfaceCore()
    const target = {
      closest: (sel: string) =>
        sel === '[data-config-cat]' ? { getAttribute: () => 'runtime' } : null,
    }
    c.onCatClick({ target } as unknown as Event)
    expect(c.selected()).toBe('runtime')
  })

  it('liveShellChanged: shell live fields only', () => {
    const base = { devUrl: 'sail dev', moduleCount: 0, themeId: 'studio', modeId: 'dark' }
    expect(liveShellChanged(base, base)).toBe(false)
    expect(liveShellChanged(base, { ...base, devUrl: 'http://x' })).toBe(true)
    expect(liveShellChanged(base, { ...base, themeId: 'ship' })).toBe(true)
  })

  it('applyLiveShell republishes module count', () => {
    const c = createConfigSurfaceCore()
    c.selectCategory('plugins')
    expect(c.applyLiveShell({
      devUrl: c.devUrl(),
      moduleCount: 3,
      themeId: 'studio',
      modeId: 'dark',
    })).toBe(true)
    expect(c.moduleCount()).toBe(3)
    expect(c.fields().find((f) => f.key === 'modules')?.value).toMatch(/3/)
  })
})
