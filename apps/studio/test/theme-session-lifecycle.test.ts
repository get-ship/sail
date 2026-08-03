/**
 * Themes session lifecycle — behavioral (draft / dirty / save+generate / leave / paint).
 */
import { afterEach, describe, expect, it } from 'vitest'
import {
  __getSavedDocForTests,
  __resetDesignForTests,
  __setThemePathForTests,
  __setWorkingDocForTests,
  addThemeToProject,
  discardThemeEdits,
  getDesignState,
  saveTheme,
  saveThemeForLeave,
  setBrandColor,
  setDesignOpsClient,
  setDesignProjectRoot,
  setTokenValue,
  undoThemeEdit,
  type DesignOpsClient,
} from '../src/design'
import { createLeaveGuard } from '../src/theme-session/theme-leave-guard'
import {
  __draftPaintHostCountForTests,
  __resetDraftPaintForTests,
  bindPreviewHost,
  flushDraftPaint,
  setDraftPaintApplier,
  unbindPreviewHost,
} from '../src/theme-session/theme-draft-paint'
import { setThemeConfirm } from '../src/theme-session/theme-confirm'

afterEach(() => {
  __resetDesignForTests()
  __resetDraftPaintForTests()
  setDesignOpsClient(null)
  setThemeConfirm(null)
})

function mockClient(partial: Partial<DesignOpsClient>): DesignOpsClient {
  return partial as DesignOpsClient
}

const LIGHT_DOC = {
  $extensions: { ship: { themeId: 'my-ship' } },
  modes: {
    light: {
      color: {
        primary: { default: { $value: 'oklch(0.6 0.1 40)', $type: 'color' } },
      },
    },
  },
}

describe('theme draft lifecycle', () => {
  it('edit mutates draft, sets dirty, does not write disk or generate', async () => {
    const writes: string[] = []
    const generates: unknown[] = []
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: ['my-ship.json'] }),
        sailThemeRead: async ({ path }) => ({
          path,
          content: JSON.stringify(LIGHT_DOC),
        }),
        sailThemeWrite: async ({ path, content }) => {
          writes.push(path + content.slice(0, 20))
          return { ok: true }
        },
        sailThemeGenerate: async (args) => {
          generates.push(args)
          return { ok: true }
        },
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    expect(getDesignState().dirty).toBe(false)

    await setTokenValue('modes.light.color.primary.default.$value', 'oklch(0.5 0.12 30)')
    expect(getDesignState().dirty).toBe(true)
    expect(writes).toHaveLength(0)
    expect(generates).toHaveLength(0)
    expect(__getSavedDocForTests()).toEqual(LIGHT_DOC)
  })

  it('Save writes JSON + SailTheme Generate mode=app and clears dirty', async () => {
    const writes: { path: string; content: string }[] = []
    const generates: { mode?: string; project?: string }[] = []
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: ['my-ship.json'] }),
        sailThemeRead: async ({ path }) => ({
          path,
          content: JSON.stringify(LIGHT_DOC),
        }),
        sailThemeWrite: async (args) => {
          writes.push(args)
          return { ok: true }
        },
        sailThemeGenerate: async (args) => {
          generates.push(args)
          return { ok: true }
        },
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    await setTokenValue('modes.light.color.primary.default.$value', 'oklch(0.4 0.1 20)')
    expect(getDesignState().dirty).toBe(true)

    const ok = await saveTheme()
    expect(ok).toBe(true)
    expect(getDesignState().dirty).toBe(false)
    expect(writes).toHaveLength(1)
    expect(writes[0]!.path).toBe('apps/studio/tokens/themes/my-ship.json')
    expect(writes[0]!.content).toMatch(/oklch\(0\.4/)
    expect(generates).toEqual([{ mode: 'app', project: 'apps/studio' }])
  })

  it('discardThemeEdits restores baseline and clears dirty + history', async () => {
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: [] }),
        sailThemeRead: async () => ({ content: '{}' }),
        sailThemeWrite: async () => ({ ok: true }),
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    await setTokenValue('modes.light.color.primary.default.$value', 'oklch(0.1 0.1 1)')
    expect(getDesignState().dirty).toBe(true)
    expect(getDesignState().canUndo).toBe(true)

    discardThemeEdits()
    expect(getDesignState().dirty).toBe(false)
    expect(getDesignState().canUndo).toBe(false)
    const primary = (
      getDesignState().themeDoc as {
        modes: { light: { color: { primary: { default: { $value: string } } } } }
      }
    ).modes.light.color.primary.default.$value
    expect(primary).toBe('oklch(0.6 0.1 40)')
  })

  it('undo/redo tracks draft edits', async () => {
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: [] }),
        sailThemeRead: async () => ({ content: '{}' }),
        sailThemeWrite: async () => ({ ok: true }),
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    await setTokenValue('modes.light.color.primary.default.$value', 'oklch(0.2 0.1 10)')
    expect(getDesignState().canUndo).toBe(true)
    undoThemeEdit()
    const doc = getDesignState().themeDoc as typeof LIGHT_DOC
    expect(doc.modes.light.color.primary.default.$value).toBe('oklch(0.6 0.1 40)')
  })
})

describe('dirty leave guard', () => {
  it('Save / Discard / Cancel resolve navigation correctly', async () => {
    const navigated: string[] = []
    const opened: string[] = []
    let dirty = true
    const guard = createLeaveGuard({
      isDirty: () => dirty,
      save: async () => {
        dirty = false
        return true
      },
      discard: () => {
        dirty = false
      },
      navigate: (href) => navigated.push(href),
      openTheme: (id) => opened.push(id),
    })

    // Cancel keeps dirty and does not navigate.
    expect(guard.requestLeave({ href: '/themes', openThemeId: null, reason: 'back' })).toBe(
      false,
    )
    expect(guard.isOpen()).toBe(true)
    await guard.choose('cancel')
    expect(navigated).toEqual([])
    expect(dirty).toBe(true)

    // Discard navigates.
    expect(guard.requestLeave({ href: '/themes', openThemeId: null, reason: 'back' })).toBe(
      false,
    )
    await guard.choose('discard')
    expect(dirty).toBe(false)
    expect(navigated).toEqual(['/themes'])

    // Clean leave is immediate.
    navigated.length = 0
    dirty = false
    expect(guard.requestLeave({ href: '/edit', openThemeId: null, reason: 'activity' })).toBe(
      true,
    )
    expect(navigated).toEqual(['/edit'])
  })

  it('Save path navigates after successful save; failed save keeps dialog open', async () => {
    const navigated: string[] = []
    let dirty = true
    let saveOk = false
    const guard = createLeaveGuard({
      isDirty: () => dirty,
      save: async () => {
        if (!saveOk) return false
        dirty = false
        return true
      },
      discard: () => {
        dirty = false
      },
      navigate: (href) => navigated.push(href),
      openTheme: () => {},
    })

    expect(guard.requestLeave({ href: '/themes', openThemeId: null, reason: 'back' })).toBe(
      false,
    )
    expect(await guard.choose('save')).toBe(false)
    expect(guard.isOpen()).toBe(true)
    expect(navigated).toEqual([])
    expect(dirty).toBe(true)

    saveOk = true
    expect(await guard.choose('save')).toBe(true)
    expect(guard.isOpen()).toBe(false)
    expect(dirty).toBe(false)
    expect(navigated).toEqual(['/themes'])
  })

  it('theme switch openThemeId completes after discard', async () => {
    const opened: string[] = []
    let dirty = true
    const guard = createLeaveGuard({
      isDirty: () => dirty,
      save: async () => true,
      discard: () => {
        dirty = false
      },
      navigate: () => {},
      openTheme: (id) => opened.push(id),
    })
    expect(
      guard.requestLeave({ href: null, openThemeId: 'other-theme', reason: 'switch' }),
    ).toBe(false)
    await guard.choose('discard')
    expect(opened).toEqual(['other-theme'])
    expect(dirty).toBe(false)
  })

  it('saveThemeForLeave skips Used-by review gate', async () => {
    const writes: string[] = []
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: [] }),
        sailThemeRead: async () => ({ content: '{}' }),
        sailThemeWrite: async ({ path }) => {
          writes.push(path)
          return { ok: true }
        },
        sailThemeGenerate: async () => ({ ok: true }),
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    await setBrandColor('#ff0000')
    expect(getDesignState().dirty).toBe(true)
    const ok = await saveThemeForLeave()
    expect(ok).toBe(true)
    expect(writes).toHaveLength(1)
    expect(getDesignState().dirty).toBe(false)
  })
})

describe('theme-draft-paint adapter', () => {
  it('calls injected applyScope-style applier with draft doc + mode', async () => {
    const calls: { mode: string; hasDoc: boolean }[] = []
    setDraftPaintApplier((host, opts) => {
      host.setAttribute('data-applied', opts.mode)
      calls.push({ mode: opts.mode, hasDoc: !!opts.doc })
    })
    const host = document.createElement('div')
    bindPreviewHost(host)
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    flushDraftPaint({ mode: 'dark' })
    expect(calls.length).toBeGreaterThan(0)
    expect(calls[calls.length - 1]).toEqual({ mode: 'dark', hasDoc: true })
    expect(host.getAttribute('data-applied')).toBe('dark')
    expect(host.getAttribute('data-draft-paint')).toBe('apply')
    unbindPreviewHost(host)
    expect(__draftPaintHostCountForTests()).toBe(0)
  })

  it('re-applies after store write when host is bound', async () => {
    let applyCount = 0
    setDraftPaintApplier(() => {
      applyCount++
    })
    const host = document.createElement('div')
    bindPreviewHost(host)
    __setWorkingDocForTests(structuredClone(LIGHT_DOC) as Record<string, unknown>)
    __setThemePathForTests('apps/studio/tokens/themes/my-ship.json')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async () => ({ files: [] }),
        sailThemeRead: async () => ({ content: '{}' }),
        sailThemeWrite: async () => ({ ok: true }),
        sailThemeFork: async () => ({ ok: true }),
      }),
    )
    const before = applyCount
    await setTokenValue('modes.light.color.primary.default.$value', 'oklch(0.3 0.1 5)')
    // emit schedules rAF — flush sync for test certainty
    flushDraftPaint()
    expect(applyCount).toBeGreaterThan(before)
  })
})

describe('add to project + confirm', () => {
  it('addThemeToProject refuses clobber of existing project theme', async () => {
    setDesignProjectRoot('apps/studio')
    setDesignOpsClient(
      mockClient({
        sailThemeList: async ({ path }) => {
          if (String(path).includes('tokens/themes')) {
            return { files: ['my-studio.json'] }
          }
          return { files: ['studio.json'] }
        },
        sailThemeRead: async ({ path }) => ({
          path,
          content: JSON.stringify({
            $extensions: { ship: { themeId: path.includes('my-studio') ? 'my-studio' : 'studio' } },
          }),
        }),
        sailThemeWrite: async () => ({ ok: true }),
        sailThemeFork: async () => ({ ok: true }),
        sailThemeGenerate: async () => ({ ok: true }),
      }),
    )
    // Load so projectThemes includes my-studio
    const { loadThemes } = await import('../src/design')
    await loadThemes({ themeId: 'my-studio' })
    const ok = await addThemeToProject('my-studio', 'studio')
    expect(ok).toBe(false)
    expect(getDesignState().lastWriteError).toMatch(/already has/i)
  })

  it('confirm can block delete path (injected)', async () => {
    setThemeConfirm(() => false)
    const { confirmDeleteTheme } = await import('../src/theme-session/theme-confirm')
    expect(confirmDeleteTheme('My Ship')).toBe(false)
  })
})

describe('chrome contracts (hygiene, not product proof)', () => {
  it('facet chrome has leave dialog and no overflow menu', async () => {
    const { readFileSync } = await import('node:fs')
    const { join, dirname } = await import('node:path')
    const { fileURLToPath } = await import('node:url')
    const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..')
    const chrome = readFileSync(
      join(repo, 'apps/studio/src/theme-facet-chrome/theme-facet-chrome.sail'),
      'utf8',
    )
    expect(chrome).toMatch(/data-theme-leave-dialog/)
    expect(chrome).toMatch(/data-leave-save/)
    expect(chrome).toMatch(/data-leave-discard/)
    expect(chrome).toMatch(/data-leave-cancel/)
    // Modal isolation: bindOpen via open={leaveOpen()} — NOT static open (non-modal).
    expect(chrome).toMatch(/open=\{leaveOpen\(\)\}/)
    expect(chrome).not.toMatch(/<dialog[^>]*\sopen[\s>]/)
    expect(chrome).toMatch(/on:sail-close=\{onLeaveCancel\}/)
    expect(chrome).toMatch(/<header\b[^>]*>[\s\S]*Unsaved theme changes/)
    expect(chrome).toMatch(/<footer\b/)
    expect(chrome).not.toMatch(/data-facet-overflow/)
    expect(chrome).not.toMatch(/overflowMenuId|overflowNav/)
    expect(chrome).toMatch(/Add to project/)
    expect(chrome).not.toMatch(/\bfork\b/i)
  })

  it('session actions never expose onOverflow', async () => {
    const { readFileSync } = await import('node:fs')
    const { join, dirname } = await import('node:path')
    const { fileURLToPath } = await import('node:url')
    const repo = join(dirname(fileURLToPath(import.meta.url)), '../../..')
    const actions = readFileSync(
      join(repo, 'apps/studio/src/theme-session/theme-session-actions.ts'),
      'utf8',
    )
    expect(actions).not.toMatch(/onOverflow/)
    expect(actions).toMatch(/addThemeToProject|Add to project/)
    expect(actions).not.toMatch(/['"]fork['"]/)
  })
})
