import { describe, expect, it, vi } from 'vitest'
import { getShellState, openTab, setActiveTab, setActivity } from './model'

describe('chrome state <-> URL hash', () => {
  it('activity switches write #/<activity>', () => {
    setActivity('checks')
    expect(location.hash).toBe('#/checks')
    setActivity('themes')
    expect(location.hash).toBe('#/themes')
  })

  it('non-shell doc tabs append to the hash', () => {
    setActivity('edit')
    openTab({ id: 'page.sail', label: 'page.sail', dirty: false })
    expect(location.hash).toBe('#/edit/page.sail')
    setActiveTab('shell')
    expect(location.hash).toBe('#/edit')
  })

  it('initial state honors a preset hash (HMR remount / reload survival)', async () => {
    // Dynamic import is the point: initial chrome state is read once at
    // module evaluation, so the test must re-import after resetModules to
    // exercise the boot path. Static import cannot re-run module init.
    location.hash = '#/tasks'
    vi.resetModules()
    const fresh = await import('./model')
    expect(fresh.getShellState().activity).toBe('tasks')
    location.hash = '#/edit'
    vi.resetModules()
    await import('./model')
  })

  it('unknown activity in hash falls back to edit', async () => {
    // Same module-boundary exception as above: re-import to re-run init.
    location.hash = '#/bogus'
    vi.resetModules()
    const fresh = await import('./model')
    expect(fresh.getShellState().activity).toBe('edit')
    location.hash = ''
    vi.resetModules()
    await import('./model')
  })

  it('getShellState reflects written activity', () => {
    setActivity('tasks')
    expect(getShellState().activity).toBe('tasks')
    setActivity('edit')
  })
})
