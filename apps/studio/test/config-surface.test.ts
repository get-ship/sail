/**
 * Config surface — fixture honesty + category publish markers.
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

  it('page mounts config-surface; surface ban-clean markers', () => {
    const page = readFileSync(join(REPO, 'apps/studio/routes/config/page.sail'), 'utf8')
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/config-surface/config-surface.sail'),
      'utf8',
    )
    expect(page).toMatch(/<config-surface\b/)
    expect(surface).toMatch(/data-config-surface/)
    expect(surface).toMatch(/data-config-nav/)
    expect(surface).toMatch(/data-config-main/)
    expect(surface).toMatch(/data-plugin="ship-cloud"/)
    expect(surface).toMatch(/data-section="danger"/)
    expect(surface).toMatch(/data-install-ship-cloud/)
    expect(surface).toMatch(/disabled/)
    expect(surface).not.toMatch(/showPlugins/)
    expect(surface).not.toMatch(/<div\b/)
    expect(surface).not.toMatch(/<span\b/)
  })

  it('buildConfigFields: live vs fixture honesty', () => {
    const ctx = { root: 'apps/studio', devUrl: 'http://127.0.0.1:7893', moduleCount: 2 }
    const project = buildConfigFields('project', ctx)
    expect(project.find((f) => f.key === 'root')?.source).toBe('project')
    expect(project.find((f) => f.key === 'root')?.value).toBe('apps/studio')
    expect(project.find((f) => f.key === 'pkg')?.source).toBe('fixture')
    expect(project.find((f) => f.key === 'render')?.source).toBe('fixture')

    const compiler = buildConfigFields('compiler', ctx)
    expect(compiler.every((f) => f.source === 'fixture')).toBe(true)

    const deck = buildConfigFields('deck', ctx)
    expect(deck.find((f) => f.key === 'dev')?.source).toBe('live')
    expect(deck.find((f) => f.key === 'dev')?.value).toBe(ctx.devUrl)
    expect(deck.find((f) => f.key === 'route')?.source).toBe('fixture')

    const studio = buildConfigFields('studio', ctx)
    expect(studio.find((f) => f.key === 'checks-hotkey')?.value).toBe(CHECKS_HOTKEY_LABEL)
    expect(studio.find((f) => f.key === 'checks-hotkey')?.source).toBe('fixture')
    expect(CHECKS_HOTKEY_LABEL).toMatch(/Shift\+K/)
    expect(CHECKS_HOTKEY_LABEL).not.toMatch(/Shift\+C$/)

    const plugins = buildConfigFields('plugins', ctx)
    expect(plugins.find((f) => f.key === 'modules')?.source).toBe('live')
    expect(plugins.find((f) => f.key === 'modules')?.value).toMatch(/2 module/)

    const danger = buildConfigFields('danger', ctx)
    expect(danger.every((f) => f.source === 'disabled')).toBe(true)
  })

  it('createConfigSurfaceCore defaults to project; selectCategory flips flags', () => {
    // Core has no mountAmbient — unit-safe outside controller setup.
    const vm = createConfigSurfaceCore()
    expect(vm.selected()).toBe('project')
    expect(vm.sectionTitle()).toBe('Project')
    expect(vm.showShipCloud()).toBe(false)
    expect(vm.showDanger()).toBe(false)
    expect(vm.categories()).toHaveLength(CONFIG_CATS.length)
    expect(vm.categories().find((c) => c.id === 'project')?.selected).toBe(true)

    vm.selectCategory('plugins')
    expect(vm.selected()).toBe('plugins')
    expect(vm.showShipCloud()).toBe(true)
    expect(vm.showDanger()).toBe(false)
    expect(vm.categories().find((c) => c.id === 'plugins')?.selected).toBe(true)

    vm.selectCategory('danger')
    expect(vm.showDanger()).toBe(true)
    expect(vm.showShipCloud()).toBe(false)
    expect(vm.fields().every((f) => f.source === 'disabled')).toBe(true)

    for (const id of CONFIG_CATS.map((c) => c.id) as ConfigCatId[]) {
      vm.selectCategory(id)
      expect(vm.selected()).toBe(id)
      expect(vm.fields().length).toBeGreaterThan(0)
    }
  })

  it('onCatClick reads data-config-cat', () => {
    const vm = createConfigSurfaceCore()
    const target = {
      closest: (sel: string) =>
        sel === '[data-config-cat]' ? { getAttribute: () => 'compiler' } : null,
    }
    vm.onCatClick({ target } as unknown as Event)
    expect(vm.selected()).toBe('compiler')
    expect(vm.sectionTitle()).toBe('Compiler')
  })
})
