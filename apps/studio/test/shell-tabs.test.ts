/**
 * Editor tab strip close/promote + C1 doc-tabs mapping (Unit 7).
 * Split from shell.test.ts for LOC cap.
 */
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { readFileSync } from 'node:fs'
import {
  __resetShellForTests,
  getShellState,
  openTab,
  setActiveTab,
} from '@ship/sail-studio-shell'
import { createCliOpsClient } from './helpers/ops-adapter-cli'
import {
  __resetEditForTests,
  getEditState,
  openDocument,
  setOpsClient,
} from '../src/edit'
import {
  closeDocTab,
  isMiddleButton,
  mapDocTabs,
  openShellTabDocument,
  resolveDocTabs,
  tabIdFromEvent,
} from '../src/compositions/edit-surface/edit-surface-tabs'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const FIX_A = 'apps/studio/test/fixtures/sample.sail'
const FIX_B = 'apps/studio/test/fixtures/tier-s.sail'
const SURFACE = join(REPO, 'apps/studio/src/edit-surface/edit-surface.sail')

describe('Unit 7 close active tab promotes document', () => {
  beforeEach(() => {
    __resetEditForTests()
    __resetShellForTests()
    setOpsClient(createCliOpsClient(REPO))
  })

  afterEach(() => {
    __resetEditForTests()
    __resetShellForTests()
  })

  it('closeDocTab returns promoted id only when closed tab was active', () => {
    openTab({ id: FIX_A, label: 'sample.sail', dirty: false })
    openTab({ id: FIX_B, label: 'tier-s.sail', dirty: false })
    setActiveTab(FIX_B)
    expect(getShellState().activeTabId).toBe(FIX_B)

    const stay = closeDocTab(FIX_A)
    expect(stay).toBeNull()
    expect(getShellState().activeTabId).toBe(FIX_B)
    expect(getShellState().tabs.map((t) => t.id)).not.toContain(FIX_A)

    openTab({ id: FIX_A, label: 'sample.sail', dirty: false })
    setActiveTab(FIX_B)
    const next = closeDocTab(FIX_B)
    expect(next).toBe(FIX_A)
    expect(getShellState().activeTabId).toBe(FIX_A)
  })

  it('close active tab + openShellTabDocument loads survivor edit path', async () => {
    await openDocument(FIX_A, { loadPreview: false })
    await openDocument(FIX_B, { loadPreview: false })
    expect(getEditState().activePath).toBe(FIX_B)
    expect(getShellState().activeTabId).toBe(FIX_B)

    const next = closeDocTab(FIX_B)
    expect(next).toBe(FIX_A)
    await openShellTabDocument(next!)
    expect(getShellState().activeTabId).toBe(FIX_A)
    expect(getEditState().activePath).toBe(FIX_A)
  }, 120_000)
})

describe('C1 doc tabs strip — mapping + chrome contract', () => {
  it('mapDocTabs: label, dirty, active, close always when multi', () => {
    const rows = mapDocTabs(
      [
        { id: FIX_A, label: 'sample.sail', dirty: false },
        { id: FIX_B, label: 'tier-s.sail', dirty: true },
      ],
      FIX_B,
    )
    expect(rows).toHaveLength(2)
    expect(rows[0]!.active).toBe(false)
    expect(rows[0]!.showClose).toBe(true)
    expect(rows[0]!.dirtyDot).toBe('')
    expect(rows[1]!.active).toBe(true)
    expect(rows[1]!.dirtyDot).toBe('●')
    expect(rows[1]!.showClose).toBe(true)
    expect(rows[1]!.canClose).toBe(true)
  })

  it('mapDocTabs: single tab hides close; dirtyById overrides shell', () => {
    const alone = mapDocTabs([{ id: FIX_A, label: 'sample.sail', dirty: false }], FIX_A)
    expect(alone[0]!.showClose).toBe(false)
    expect(alone[0]!.canClose).toBe(false)

    const fromEdit = mapDocTabs(
      [{ id: FIX_A, label: 'sample.sail', dirty: false }],
      FIX_A,
      new Set([FIX_A]),
    )
    expect(fromEdit[0]!.dirtyDot).toBe('●')
    expect(fromEdit[0]!.dirty).toBe(true)
  })

  it('resolveDocTabs drops shell placeholder and synthesizes from activePath', () => {
    expect(
      resolveDocTabs([{ id: 'shell', label: 'shell', dirty: false }], null, null, false),
    ).toEqual([])
    expect(
      resolveDocTabs(
        [{ id: 'shell', label: 'shell', dirty: false }],
        null,
        FIX_A,
        true,
      ),
    ).toEqual([{ id: FIX_A, label: 'sample.sail', dirty: true }])
    expect(
      resolveDocTabs(
        [
          { id: 'shell', label: 'shell', dirty: false },
          { id: FIX_A, label: 'sample.sail', dirty: false },
        ],
        FIX_A,
        FIX_A,
        false,
      ),
    ).toEqual([{ id: FIX_A, label: 'sample.sail', dirty: false }])
  })

  it('middle-click helpers: button 1 + tab id from tab-id', () => {
    expect(isMiddleButton({ button: 1 } as MouseEvent)).toBe(true)
    expect(isMiddleButton({ button: 0 } as MouseEvent)).toBe(false)

    const host = document.createElement('div')
    host.setAttribute('tab-id', FIX_A)
    const child = document.createElement('button')
    host.appendChild(child)
    document.body.appendChild(host)
    expect(tabIdFromEvent({ target: child } as unknown as Event)).toBe(FIX_A)
    host.remove()
  })

  it('edit-surface markup: scrollable tab list, h(8), close, middle-click, one accent', () => {
    const src = readFileSync(SURFACE, 'utf8')
    // Bounded strip + independent scroll for open files (not wrap)
    expect(src).toMatch(/edit-tabs[\s\S]*h\(8\)/)
    expect(src).toMatch(/edit-tab-list/)
    expect(src).toMatch(/overflowX\(auto\)/)
    expect(src).toMatch(/nowrap/)
    expect(src).toMatch(/label="Open documents"/)
    // Per-tab: label path title, dirty, close, active accent via current/primary
    expect(src).toMatch(/tab-id=\{tab\.id\}/)
    expect(src).toMatch(/dirty-dot/)
    expect(src).toMatch(/tab-close/)
    expect(src).toMatch(
      /when\(\{\s*current:[\s\S]*borderT\(primary\)|when\(current,\s*\{[^}]*borderT\(primary\)/,
    )
    expect(src).toMatch(/on:mousedown=\{onTabMiddle\}/)
    // Save/actions pinned outside the scroll list
    expect(src).toMatch(/save-source/)
    expect(src).toMatch(/absorption:\s*studio-tabs|absorption vote|studio-tabs/)
  })
})
