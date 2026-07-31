/**
 * Editor tab strip close/promote (Unit 7) — split from shell.test.ts for LOC cap.
 */
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  __resetShellForTests,
  getShellState,
  openTab,
  setActiveTab,
} from '@ship/sail-studio-shell'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
import {
  __resetEditForTests,
  getEditState,
  openDocument,
  setOpsClient,
} from '../src/edit'
import { closeDocTab, openShellTabDocument } from '../src/edit-surface/edit-surface-vm'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const FIX_A = 'apps/studio/test/fixtures/sample.sail'
const FIX_B = 'apps/studio/test/fixtures/tier-s.sail'

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
