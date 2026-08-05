/**
 * Themes home — behavioral honesty contracts (H-L1–H-L7).
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
import { setThemeConfirm } from '../src/theme-session/theme-confirm'
import { createThemesSurface } from '../src/themes-surface/themes-surface-vm'
// mountAmbient requires a setup scope window (controller/setup).
import {
  createScope,
  withScope,
  type EffectScope,
} from '../../../packages/sail/runtime/src/effect-scope'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SURFACE = join(REPO, 'apps/studio/src/themes-surface/themes-surface.sail')
const VM = join(REPO, 'apps/studio/src/themes-surface/themes-surface-vm.ts')

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

const STUDIO_DOC = JSON.stringify({
  $extensions: { ship: { themeId: 'studio' } },
  modes: {
    light: {
      color: {
        primary: { default: { $value: 'oklch(0.55 0.1 40)', $type: 'color' } },
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
        if (String(path).includes('apps/studio/tokens/themes')) {
          return { files: [...projectFiles] }
        }
        return { files: ['studio.json'] }
      },
      sailThemeRead: async ({ path }) => {
        if (String(path).includes('my-ship')) {
          return { path, content: DOC }
        }
        return { path, content: STUDIO_DOC }
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

function menuEvent(value: string): CustomEvent<{ value: string }> {
  return new CustomEvent('sail-menu-select', { detail: { value } })
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
    expect(deleted).toEqual(['apps/studio/tokens/themes/my-ship.json'])
    const after = getDesignState()
    expect(after.themeId === '' || after.themeId === 'studio').toBe(true)
    if (after.themeId === '') {
      expect(after.themeDoc).toBeNull()
    } else {
      expect(after.themePath).toBeNull()
    }
    expect(after.dirty).toBe(false)
  })
})

describe('themes home — honesty UI contracts (source)', () => {
  it('removes fake enable switch, default star, and dishonest enable copy', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    const vm = readFileSync(VM, 'utf8')
    // H-L1 / H-L2: no enable-switch theater
    expect(sail).not.toMatch(/data-theme-enable/)
    expect(sail).toMatch(/theme-add-project/)
    expect(sail).toMatch(/Add to project/)
    expect(vm).not.toMatch(/onEnableChange/)
    // H-L4 option A: no default star / Set default UI
    expect(sail).not.toMatch(/data-theme-default/)
    expect(sail).not.toMatch(/data-preview-set-default/)
    expect(sail).not.toMatch(/Set as default/)
    expect(sail).not.toMatch(/>Default</)
    expect(vm).not.toMatch(/onSetDefault|onPreviewSetDefault|defaultId/)
    // H-L5 honest blurb
    expect(sail).not.toMatch(/Enable catalog packs/i)
    expect(sail).toMatch(/Project themes live on disk|Add catalog themes to the project/)
    // H-L6 never say fork in product UI
    expect(sail).not.toMatch(/\bfork\b/i)
    // Mode switch only (not enable) may remain for dual-mode preview
    expect(sail).toMatch(/mode-switch/)
  })

  it('paints catalog/project hosts via theme= not theme (Job 5d)', () => {
    const sail = readFileSync(SURFACE, 'utf8')
    expect(sail).toMatch(/theme=\{row\.previewId\}/)
    expect(sail).toMatch(/theme=\{previewPaintId\(\)\}/)
    // Load-bearing preview attr must not remain on home hosts
    expect(sail).not.toMatch(/theme-preview=\{/)
  })

  it('New theme uses confirmNewTheme, not Add-to-project copy', () => {
    const vm = readFileSync(VM, 'utf8')
    const confirm = readFileSync(
      join(REPO, 'apps/studio/src/theme-session/theme-confirm.ts'),
      'utf8',
    )
    expect(vm).toMatch(/confirmNewTheme/)
    expect(confirm).toMatch(/Create new project theme/)
    expect(confirm).toMatch(/confirmNewTheme/)
  })
})

describe('themes home — surface handlers (behavioral)', () => {
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
    // Drive menuTarget via onMenuOpen then onRowMenu
    const btn = document.createElement('button')
    btn.dataset.themeId = 'my-ship'
    document.body.appendChild(btn)
    vm.onMenuOpen({ target: btn, stopPropagation() {} } as unknown as Event)
    vm.onRowMenu(menuEvent('delete'))

    // Wait microtasks for deleteTheme promise
    await new Promise((r) => setTimeout(r, 20))
    expect(confirms.some((m) => /Delete project theme/i.test(m))).toBe(true)
    expect(deleted).toEqual(['apps/studio/tokens/themes/my-ship.json'])
  })

  it('delete confirm false → never calls delete', async () => {
    const deleted: string[] = []
    wireProjectAndCatalog({ onDelete: (p) => deleted.push(p) })
    await loadThemes({ themeId: 'my-ship' })
    setThemeConfirm(() => false)

    const vm = makeSurface()
    const btn = document.createElement('button')
    btn.dataset.themeId = 'my-ship'
    document.body.appendChild(btn)
    vm.onMenuOpen({ target: btn, stopPropagation() {} } as unknown as Event)
    vm.onRowMenu(menuEvent('delete'))
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
    await loadThemes({ themeId: 'studio' })
    setThemeConfirm(() => true)

    const vm = makeSurface()
    expect(vm.catalogRows().some((r) => r.id === 'studio')).toBe(true)
    const btn = document.createElement('button')
    btn.dataset.themeId = 'studio'
    document.body.appendChild(btn)
    vm.onMenuOpen({ target: btn, stopPropagation() {} } as unknown as Event)
    vm.onRowMenu(menuEvent('delete'))
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
    await loadThemes({ themeId: 'studio' })
    setThemeConfirm(() => true)

    const vm = makeSurface()
    const btn = document.createElement('button')
    btn.dataset.themeId = 'studio'
    document.body.appendChild(btn)
    vm.onAddToProject({ target: btn, stopPropagation() {} } as unknown as Event)
    await new Promise((r) => setTimeout(r, 30))
    expect(forks.length).toBe(1)
    expect(forks[0]!.new_id).toBe('my-studio')
    expect(vm.selectedId()).toBe('my-studio')
  })

  it('add to project confirm false → no fork', async () => {
    const forks: string[] = []
    wireProjectAndCatalog({
      projectFiles: [],
      onFork: (a) => forks.push(a.new_id),
    })
    await loadThemes({ themeId: 'studio' })
    setThemeConfirm(() => false)

    const vm = makeSurface()
    const btn = document.createElement('button')
    btn.dataset.themeId = 'studio'
    document.body.appendChild(btn)
    vm.onAddToProject({ target: btn, stopPropagation() {} } as unknown as Event)
    await new Promise((r) => setTimeout(r, 20))
    expect(forks).toEqual([])
    expect(vm.toastMessage()).toMatch(/cancelled/i)
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
