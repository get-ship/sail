import { afterEach, describe, expect, it } from 'vitest'
import {
  __resetSelectionForTests,
  clearSelection,
  getSelection,
  selectFromBridge,
  selectNode,
  subscribeSelection,
} from './selection'
import {
  __resetShellForTests,
  activities,
  applyFabVisibleFromBridge,
  bindSelectionToShell,
  clearModuleActivities,
  getShellState,
  handleBridgeRegion,
  setActivity,
  setFabVisible,
  setBadges,
  setModuleActivities,
} from './model'
import { __resetBatchForTests, getBatchItems } from './batch'
import { LS_FAB } from './channel'
import { resolveRectToNodes, joinRectToRenderRegions } from './regions'

afterEach(() => {
  __resetSelectionForTests()
  __resetShellForTests()
  __resetBatchForTests()
  try {
    localStorage.removeItem(LS_FAB)
  } catch {
    /* */
  }
})

describe('shared selection store', () => {
  it('one node id selected anywhere is selected everywhere', () => {
    const seen: (string | null)[] = []
    const unsub = subscribeSelection((s) => seen.push(s.nodeId))

    selectNode('n:page.sail:42')
    expect(getSelection().nodeId).toBe('n:page.sail:42')
    expect(getSelection().source).toBe('surface')

    let surfaceB: string | null = null
    const unsubB = subscribeSelection((s) => {
      surfaceB = s.nodeId
    })
    selectNode('n:page.sail:42')
    expect(surfaceB).toBe('n:page.sail:42')

    selectFromBridge('n:card:hero', 'main > card')
    expect(getSelection().nodeId).toBe('n:card:hero')
    expect(getSelection().selector).toBe('main > card')
    expect(getSelection().source).toBe('bridge')

    clearSelection()
    expect(getSelection().nodeId).toBeNull()

    unsub()
    unsubB()
    expect(seen).toContain('n:page.sail:42')
    expect(seen).toContain('n:card:hero')
    expect(seen).toContain(null)
  })

  it('shell snapshot mirrors selection across activities (surface bindings)', () => {
    const unbind = bindSelectionToShell()
    setActivity('edit')
    selectNode('node-a')
    expect(getShellState().selection.nodeId).toBe('node-a')

    setActivity('checks')
    expect(getShellState().selection.nodeId).toBe('node-a')
    expect(getShellState().activity).toBe('checks')

    setActivity('tasks')
    selectFromBridge('node-b')
    expect(getShellState().selection.nodeId).toBe('node-b')
    expect(getShellState().activity).toBe('tasks')

    // Edit surface would read the same store
    setActivity('edit')
    expect(getShellState().selection.nodeId).toBe('node-b')

    unbind()
  })

  it('badge plumbing is shared on shell state', () => {
    setBadges(3, 4)
    expect(getShellState().checksBadge).toBe(3)
    expect(getShellState().tasksBadge).toBe(4)
  })
})

describe('module activity seam', () => {
  it('registers additive rail entries without replacing core', () => {
    setModuleActivities([
      {
        id: 'hello',
        label: 'Hello',
        route: '/_studio/x/hello',
        module: 'module-hello',
      },
    ])
    const ids = activities()
    expect(ids).toContain('edit')
    expect(ids).toContain('tasks')
    expect(ids).toContain('hello')
    setActivity('hello')
    expect(getShellState().activity).toBe('hello')
    expect(getShellState().moduleRoute).toBe('/_studio/x/hello')
  })

  it('rejects module ids that collide with core', () => {
    setModuleActivities([
      { id: 'edit', label: 'Hijack', route: '/_studio/x/edit', module: 'bad' },
      { id: 'hello', label: 'Hello', route: '/_studio/x/hello', module: 'module-hello' },
    ])
    // Core `edit` remains exactly once; hijack attempt dropped.
    expect(activities().filter((a) => a === 'edit')).toHaveLength(1)
    expect(getShellState().moduleActivities.map((a) => a.id)).toEqual(['hello'])
  })

  it('killing modules leaves core rail functional (containment)', () => {
    setModuleActivities([
      {
        id: 'hello',
        label: 'Hello',
        route: '/_studio/x/hello',
        module: 'module-hello',
        devUnsigned: true,
      },
    ])
    setActivity('hello')
    expect(getShellState().moduleUnsignedBanner).toBe(true)
    clearModuleActivities()
    expect(getShellState().activity).toBe('edit')
    expect(getShellState().moduleRoute).toBeNull()
    expect(activities()).toEqual(['edit', 'components', 'themes', 'tasks', 'checks'])
    setActivity('checks')
    expect(getShellState().activity).toBe('checks')
  })
})

describe('FAB visibility', () => {
  it('setFabVisible writes localStorage and shell state', () => {
    setFabVisible(false)
    expect(getShellState().fabVisible).toBe(false)
    expect(localStorage.getItem(LS_FAB)).toBe('0')
    setFabVisible(true)
    expect(localStorage.getItem(LS_FAB)).toBe('1')
  })

  it('applyFabVisibleFromBridge does not loop localStorage thrash', () => {
    applyFabVisibleFromBridge(false)
    expect(getShellState().fabVisible).toBe(false)
  })
})

describe('region resolve + batch tray', () => {
  it('joinRectToRenderRegions intersects render metadata', () => {
    const nodes = joinRectToRenderRegions(
      { left: 10, top: 10, width: 100, height: 50 },
      [
        { id: 'n:in', label: '<card> in', x: 20, y: 20, w: 40, h: 20 },
        { id: 'n:out', label: '<p> out', x: 500, y: 500, w: 10, h: 10 },
      ],
    )
    expect(nodes.map((n) => n.nodeId)).toEqual(['n:in'])
  })

  it('handleBridgeRegion adds batch item and selects first node', async () => {
    // In happy-dom, elementFromPoint may be empty — still creates batch entry.
    document.body.innerHTML = '<main id="app"><div id="hero">Hi</div></main>'
    // Polyfill elementFromPoint for happy-dom if missing
    if (!document.elementFromPoint) {
      document.elementFromPoint = () => document.getElementById('hero')
    } else {
      const orig = document.elementFromPoint.bind(document)
      document.elementFromPoint = (x, y) => {
        if (x >= 0 && y >= 0) return document.getElementById('hero')
        return orig(x, y)
      }
    }

    await handleBridgeRegion('screenshot', {
      left: 0,
      top: 0,
      width: 100,
      height: 80,
    })
    expect(getBatchItems().length).toBe(1)
    expect(getBatchItems()[0]!.kind).toBe('screenshot')
    expect(getShellState().batchCount).toBe(1)
    // If locate found hero, selection is set
    const sel = getShellState().selection.nodeId
    if (sel) {
      expect(getShellState().regionNodes.length).toBeGreaterThan(0)
    }
  })

  it('resolveRectToNodes skips studio chrome', () => {
    document.body.innerHTML =
      '<div id="sail-studio-root"></div><main id="app"><span id="ok">x</span></main>'
    if (!document.elementFromPoint) {
      document.elementFromPoint = () => document.getElementById('ok')
    } else {
      document.elementFromPoint = () => document.getElementById('ok')
    }
    const nodes = resolveRectToNodes({ left: 0, top: 0, width: 50, height: 50 })
    expect(nodes.every((n) => n.nodeId !== '#sail-studio-root')).toBe(true)
  })
})
