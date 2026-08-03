/**
 * Themes home — behavioral contracts (ops + store), not template string scans.
 */
import { afterEach, describe, expect, it } from 'vitest'
import {
  __resetDesignForTests,
  deleteTheme,
  getDesignState,
  loadThemes,
  openTheme,
  setDesignOpsClient,
  setDesignProjectRoot,
  type DesignOpsClient,
} from '../src/design'

afterEach(() => {
  __resetDesignForTests()
  setDesignOpsClient(null)
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

describe('themes home — delete behavior', () => {
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
          // Catalog only — no project themes.
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
    let projectFiles = ['my-ship.json']
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async ({ path }) => {
          if (String(path).includes('apps/studio/tokens/themes')) {
            return { files: [...projectFiles] }
          }
          // catalog
          return { files: ['studio.json'] }
        },
        sailThemeRead: async ({ path }) => {
          if (String(path).includes('my-ship')) {
            return { path, content: DOC }
          }
          return {
            path,
            content: JSON.stringify({ $extensions: { ship: { themeId: 'studio' } } }),
          }
        },
        sailThemeDelete: async (args: { path: string }) => {
          deleted.push(args.path)
          // File gone — subsequent loadThemes must not re-open it.
          projectFiles = projectFiles.filter((f) => !args.path.endsWith(f))
          return { ok: true }
        },
      }),
    )

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
    // Active project theme cleared; may fall through to a catalog theme on reload.
    expect(after.themeId === '' || after.themeId === 'studio').toBe(true)
    if (after.themeId === '') {
      expect(after.themeDoc).toBeNull()
    } else {
      // Reloaded catalog studio — not the deleted project path.
      expect(after.themePath).toBeNull()
    }
    expect(after.dirty).toBe(false)
  })
})
