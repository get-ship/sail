/**
 * Themes home — behavioral honesty contracts.
 * Ops + createThemesSurface handlers; not template-string-only.
 */
import { afterEach, describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  __resetDesignForTests,
  addThemeToProject,
  deleteTheme,
  getDesignState,
  loadThemes,
  openTheme,
  setDesignOpsClient,
  setDesignProjectRoot,
  type DesignOpsClient,
} from '../src/design'
import { setThemeConfirm } from '../src/compositions/theme-session/theme-confirm'
import { createThemesSurface } from '../src/compositions/themes-surface/themes-surface-vm'
// mountAmbient requires a setup scope window (controller/setup).
// Import from the package entry so CURRENT_SCOPE matches @ship/sail-runtime's mount.
import {
  createScope,
  withScope,
  type EffectScope,
} from '@ship/sail-runtime/internal'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SURFACE = join(REPO, 'apps/studio/src/themes-surface/themes-surface.sail')
const VM = join(REPO, 'apps/studio/src/compositions/themes-surface/themes-surface-vm.ts')

let liveScope: EffectScope | null = null

function makeSurface() {
  liveScope?.dispose()
  liveScope = createScope()
  return withScope(liveScope, () => createThemesSurface())
}

afterEach(() => {
  liveScope?.dispose()
  liveScope = null
  __resetDesignForTests()
  setDesignOpsClient(null)
  setThemeConfirm(null)
})

function mockClient(partial: Partial<DesignOpsClient>): DesignOpsClient {
  return partial as DesignOpsClient
}

const DOC = JSON.stringify({
  $extensions: { ship: { themeId: 'my-ship' } },
  modes: {
    light: {
      color: {
        primary: { default: { $value: 'oklch(0.6 0.1 40)', $type: 'color' } },
      },
    },
  },
})

/** Dual-mode catalog theme present in package THEME_META (not project-only studio). */
const SHIP_DOC = JSON.stringify({
  $extensions: { ship: { themeId: 'ship' } },
  modes: {
    light: {
      color: {
        primary: { default: { $value: 'oklch(0.55 0.1 40)', $type: 'color' } },
      },
    },
    dark: {
      color: {
        primary: { default: { $value: 'oklch(0.7 0.12 40)', $type: 'color' } },
      },
    },
  },
})

function wireProjectAndCatalog(opts?: {
  projectFiles?: string[]
  onDelete?: (path: string) => void
  onFork?: (args: { source_path: string; dest_path: string; new_id: string }) => void
}) {
  let projectFiles = opts?.projectFiles ?? ['my-ship.json']
  setDesignProjectRoot('apps/studio')
  setDesignOpsClient(
    mockClient({
      sailThemeList: async ({ path }) => {
        if (String(path).includes('apps/studio/themes')) {
          return { files: [...projectFiles] }
        }
        // Catalog = package model themes (ship is dual-mode and always registered).
        return { files: ['ship.json'] }
      },
      sailThemeRead: async ({ path }) => {
        if (String(path).includes('my-ship')) {
          return { path, content: DOC }
        }
        return { path, content: SHIP_DOC }
      },
      sailThemeDelete: async (args: { path: string }) => {
        opts?.onDelete?.(args.path)
        projectFiles = projectFiles.filter((f) => !args.path.endsWith(f))
        return { ok: true }
      },
      sailThemeFork: async (args) => {
        opts?.onFork?.(args)
        const name = `${args.new_id}.json`
        if (!projectFiles.includes(name)) projectFiles.push(name)
        return { ok: true }
      },
      sailThemeGenerate: async () => ({ ok: true }),
      sailThemeWrite: async () => ({ ok: true }),
    }),
  )
  return {
    getProjectFiles: () => projectFiles,
  }
}

function menuDetail(value: string): { value: string } {
  return { value }
}

describe('themes home — delete behavior (store)', () => {
  it('refuses delete when SailTheme/Delete is not wired (honest offline error)', async () => {
    setDesignOpsClient(null)
    const ok = await deleteTheme('my-ship')
    expect(ok).toBe(false)
    expect(getDesignState().lastWriteError).toMatch(/not available|offline/i)
  })

  it('refuses catalog theme delete (ops never called)', async () => {
    let deleteCalls = 0
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async ({ path }) => {
          if (String(path).includes('model/themes')) {
            return { files: ['studio.json'] }
          }
          return { files: [] }
        },
        sailThemeRead: async ({ path }) => ({
          path,
          content: JSON.stringify({
            $extensions: { ship: { themeId: 'studio' } },
          }),
        }),
        sailThemeDelete: async () => {
          deleteCalls++
          return { ok: true }
        },
      }),
    )
    await loadThemes({ themeId: 'studio' })
    const ok = await deleteTheme('studio')
    expect(ok).toBe(false)
    expect(deleteCalls).toBe(0)
    expect(getDesignState().lastWriteError).toMatch(/Only project themes|project themes/i)
  })

  it('deletes a project theme via SailTheme/Delete and clears the open doc', async () => {
    const deleted: string[] = []
    wireProjectAndCatalog({ onDelete: (p) => deleted.push(p) })

    await loadThemes({ themeId: 'my-ship' })
    await openTheme('my-ship')
    const before = getDesignState()
    expect(before.themeId).toBe('my-ship')
    expect(before.themePath).toMatch(/my-ship\.json/)
    expect(before.themeDoc).not.toBeNull()

    const ok = await deleteTheme('my-ship')
    expect(ok).toBe(true)
    expect(deleted).toEqual(['apps/studio/themes/my-ship.json'])
    const after = getDesignState()
    expect(after.themeId === '' || after.themeId === 'ship').toBe(true)
    if (after.themeId === '') {
      expect(after.themeDoc).toBeNull()
    } else {
      expect(after.themePath).toBeNull()
    }
    expect(after.dirty).toBe(false)
  })
})

describe('themes home — UI contracts (source)', () => {
  it('catalog rows use Add to project and mode switch', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const vm = readFileSync(VM, 'utf8')
    expect(sail).toMatch(/theme-add-project/)
    expect(sail).toMatch(/Add to project/)
    expect(sail).toMatch(/Project themes live on disk|Add catalog themes to the project/)
    expect(sail).toMatch(/mode-switch/)
    expect(vm).toMatch(/addThemeToProject|openTheme|loadThemes/)
  })

  it('passes theme ids via closed-over handlers (not theme-id / dataset)', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const vm = readFileSync(VM, 'utf8')
    // Markup closes over id — fails if closures dropped back to DOM bus.
    expect(sail).toMatch(/on:click=\{\(\)\s*=>\s*onListClick\(id\)\}/)
    expect(sail).toMatch(/on:click=\{\(e\)\s*=>\s*onMenuOpen\(id,\s*e\)\}/)
    expect(sail).toMatch(/on:click=\{\(e\)\s*=>\s*onAddToProject\(id,\s*e\)\}/)
    expect(sail).not.toMatch(/theme-id=\{/)
    expect(vm).not.toMatch(/dataset\.themeId/)
    expect(vm).not.toMatch(/closest\?\.\(\s*['"]\[theme-id\]/)
    expect(vm).toMatch(/const onListClick = \(id: string\)/)
    expect(vm).toMatch(/const onMenuOpen = \(id: string/)
    expect(vm).toMatch(/const onAddToProject = \(id: string/)
  })

  it('paints catalog/project hosts via theme={id}', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    expect(sail).toMatch(/theme=\{previewId\}|theme=\{row\.previewId\}/)
    expect(sail).toMatch(/theme=\{previewPaintId\(\)\}/)
  })

  it('preview mode stamps data-mode via use:paintMode (not authored data-* / bare mode=)', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    // themes.css selects [theme=id][data-mode=light|dark]. App markup must not
    // author data-mode= (sail-no-data-attr); local use: action stamps the cascade bit.
    expect(sail).toMatch(/use:paintMode=\{previewModeId\(\)\}/)
    expect(sail).toMatch(/use:paintMode=\{paintMode\}|use:paintMode=\{row\.paintMode\}/)
    expect(sail).toMatch(/const paintMode:\s*Action/)
    expect(sail).toMatch(/setAttribute\(['"]data-mode['"]/)
  })

  it('New theme uses confirmNewTheme, not Add-to-project copy', () => {
    const vm = readFileSync(VM, 'utf8')
    const confirm = readFileSync(
      join(REPO, 'apps/studio/src/compositions/theme-session/theme-confirm.ts'),
      'utf8',
    )
    expect(vm).toMatch(/confirmNewTheme/)
    expect(confirm).toMatch(/Create new project theme/)
    expect(confirm).toMatch(/confirmNewTheme/)
  })
})

describe('themes home — surface handlers (behavioral)', () => {
  it('list click selects theme id + marks active row (closure path)', async () => {
    wireProjectAndCatalog({ projectFiles: ['my-ship.json'] })
    await loadThemes({ themeId: 'ship' })
    const vm = makeSurface()
    // Catalog ship is selected from load; switch to project id via id-param handler.
    expect(vm.selectedId()).toBe('ship')
    vm.onListClick('my-ship')
    expect(vm.selectedId()).toBe('my-ship')
    expect(vm.hasSelected()).toBe(true)
    const row = vm.projectRows().find((r) => r.id === 'my-ship')
    expect(row?.selected).toBe(true)
    // Preview hosts follow selection (theme= paint id for registered/project tables).
    expect(vm.previewThemeId()).toBe('my-ship')
    expect(vm.previewPaintId()).toBeTruthy()
  })

  it('delete menu + confirm true → deleteTheme (project only)', async () => {
    const deleted: string[] = []
    wireProjectAndCatalog({ onDelete: (p) => deleted.push(p) })
    await loadThemes({ themeId: 'my-ship' })

    const confirms: string[] = []
    setThemeConfirm((msg) => {
      confirms.push(msg)
      return true
    })

    const vm = makeSurface()
    // Project row selected
    expect(vm.projectRows().some((r) => r.id === 'my-ship')).toBe(true)
    // Drive menuTarget via onMenuOpen(id) then onRowMenu — no DOM dataset bus
    vm.onMenuOpen('my-ship')
    vm.onRowMenu(menuDetail('delete'))

    // Wait microtasks for deleteTheme promise
    await new Promise((r) => setTimeout(r, 20))
    expect(confirms.some((m) => /Delete project theme/i.test(m))).toBe(true)
    expect(deleted).toEqual(['apps/studio/themes/my-ship.json'])
  })

  it('delete confirm false → never calls delete', async () => {
    const deleted: string[] = []
    wireProjectAndCatalog({ onDelete: (p) => deleted.push(p) })
    await loadThemes({ themeId: 'my-ship' })
    setThemeConfirm(() => false)

    const vm = makeSurface()
    vm.onMenuOpen('my-ship')
    vm.onRowMenu(menuDetail('delete'))
    await new Promise((r) => setTimeout(r, 20))
    expect(deleted).toEqual([])
    expect(vm.toastMessage()).toMatch(/cancelled/i)
  })

  it('catalog delete menu refused — no ops delete', async () => {
    const deleted: string[] = []
    wireProjectAndCatalog({
      projectFiles: [],
      onDelete: (p) => deleted.push(p),
    })
    await loadThemes({ themeId: 'ship' })
    setThemeConfirm(() => true)

    const vm = makeSurface()
    expect(vm.catalogRows().some((r) => r.id === 'ship')).toBe(true)
    vm.onMenuOpen('ship')
    vm.onRowMenu(menuDetail('delete'))
    await new Promise((r) => setTimeout(r, 20))
    expect(deleted).toEqual([])
    expect(vm.toastMessage()).toMatch(/Catalog themes cannot be deleted/i)
  })

  it('add to project confirm true → addThemeToProject', async () => {
    const forks: { new_id: string; source_path: string }[] = []
    wireProjectAndCatalog({
      projectFiles: [],
      onFork: (a) => forks.push({ new_id: a.new_id, source_path: a.source_path }),
    })
    await loadThemes({ themeId: 'ship' })
    setThemeConfirm(() => true)

    const vm = makeSurface()
    vm.onAddToProject('ship')
    await new Promise((r) => setTimeout(r, 30))
    expect(forks.length).toBe(1)
    expect(forks[0]!.new_id).toBe('my-ship')
    expect(vm.selectedId()).toBe('my-ship')
  })

  it('add to project confirm false → no fork', async () => {
    const forks: string[] = []
    wireProjectAndCatalog({
      projectFiles: [],
      onFork: (a) => forks.push(a.new_id),
    })
    await loadThemes({ themeId: 'ship' })
    setThemeConfirm(() => false)

    const vm = makeSurface()
    vm.onAddToProject('ship')
    await new Promise((r) => setTimeout(r, 20))
    expect(forks).toEqual([])
    expect(vm.toastMessage()).toMatch(/cancelled/i)
  })

  it('mode switch updates previewModeId for dual-mode themes', async () => {
    wireProjectAndCatalog({ projectFiles: [] })
    await loadThemes({ themeId: 'ship' })
    const vm = makeSurface()
    // loadThemes opens ship (dual-mode catalog); surface sync selects it.
    expect(vm.selectedId()).toBe('ship')
    expect(vm.canToggleMode()).toBe(true)
    const start = vm.previewModeId()
    const flipToDark = start !== 'dark'
    const input = document.createElement('input')
    input.type = 'checkbox'
    input.checked = flipToDark
    vm.onPreviewModeSwitch({ target: input } as unknown as Event)
    expect(vm.previewModeId()).toBe(flipToDark ? 'dark' : 'light')
    expect(vm.modeDarkOn()).toBe(flipToDark)
    expect(vm.modeLightOn()).toBe(!flipToDark)
    // Toggle back
    input.checked = !flipToDark
    vm.onPreviewModeSwitch({ target: input } as unknown as Event)
    expect(vm.previewModeId()).toBe(flipToDark ? 'light' : 'dark')
  })

  it('project rows have no enable-off path; surface does not export onEnableChange', async () => {
    wireProjectAndCatalog()
    await loadThemes({ themeId: 'my-ship' })
    const vm = makeSurface() as Record<string, unknown>
    expect(vm.onEnableChange).toBeUndefined()
    expect(typeof vm.onAddToProject).toBe('function')
    // Project rows: inProject true, no fake default flag
    const proj = (vm.projectRows as () => Array<{ id: string; inProject: boolean }>)()
    expect(proj.find((r) => r.id === 'my-ship')?.inProject).toBe(true)
    // Off-switch gone so cannot silent-delete via enable theater
    const sail = readFileSync(SURFACE, 'utf8')
    expect(sail).not.toMatch(/data-theme-enable/)
  })

  it('New theme uses create copy and confirmNewTheme wording', async () => {
    const forks: string[] = []
    const confirms: string[] = []
    wireProjectAndCatalog({
      onFork: (a) => forks.push(a.new_id),
    })
    await loadThemes({ themeId: 'studio' })
    setThemeConfirm((msg) => {
      confirms.push(msg)
      return true
    })
    const vm = makeSurface()
    vm.onNewTheme()
    await new Promise((r) => setTimeout(r, 30))
    expect(confirms[0]).toMatch(/Create new project theme/i)
    expect(confirms[0]).not.toMatch(/^Add “/)
    expect(forks.length).toBe(1)
    expect(forks[0]).toMatch(/^my-theme-/)
  })
})

describe('themes home — addThemeToProject store still honest', () => {
  it('addThemeToProject creates my-* project membership', async () => {
    const forks: string[] = []
    wireProjectAndCatalog({
      projectFiles: [],
      onFork: (a) => forks.push(a.new_id),
    })
    await loadThemes({ themeId: 'studio' })
    const ok = await addThemeToProject('my-studio', 'studio')
    expect(ok).toBe(true)
    expect(forks).toContain('my-studio')
  })
})
