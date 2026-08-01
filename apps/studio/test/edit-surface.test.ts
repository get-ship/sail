import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'
import { __resetSelectionForTests, getSelection, getShellState } from '@ship/sail-studio-shell'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
import {
  __resetEditForTests,
  activeDoc,
  applyModelUpdate,
  applySourceEditorScroll,
  applySourceScroll,
  bindEditSelection,
  elementRows,
  getEditState,
  hitTestRegion,
  hoverPreviewAt,
  openDocument,
  parseTreeNodes,
  saveActiveSource,
  scrollSourceEditor,
  selectEditNode,
  selectFromPreviewClick,
  setOpsClient,
  setSourceText,
  setView,
  subscribeEdit,
  pathToLiveRoute,
  fileTreeGroups,
  fileSailTreeNodes,
  nextTreeFocus,
  fileDisplayName,
  writeSignalValue,
  uiAnatomy,
  uiElementRows,
  uiStateCards,
  type OpsClient,
} from '../src/edit'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
// Chrome shell (activity bar + status) — richer tree than the root redirect page.
const PAGE = 'apps/studio/src/sail-studio-chrome/sail-studio-chrome.sail'
const SAIL_SITE_PAGE = 'apps/sail-site/routes/page.sail'

let cli: OpsClient
let unbind: (() => void) | null = null

beforeAll(() => {
  cli = createCliOpsClient(REPO)
}, 120_000)

afterEach(() => {
  __resetEditForTests()
  __resetSelectionForTests()
  unbind?.()
  unbind = null
})

describe('element tree matches SailInspect/Tree', () => {
  it('openDocument tree equals sailInspectTree for page.sail', async () => {
    setOpsClient(cli)
    unbind = bindEditSelection()
    const tree = await cli.sailInspectTree({ path: PAGE })
    // No sourceText — must load via Model.source (High #1)
    const doc = await openDocument(PAGE, { loadPreview: false })
    expect(doc).not.toBeNull()
    const expected = parseTreeNodes(tree.nodes)
    expect(doc!.tree.nodes.map((n) => n.id)).toEqual(expected.map((n) => n.id))
    expect(doc!.tree.nodes.map((n) => n.tag)).toEqual(expected.map((n) => n.tag))
    expect(doc!.tree.roots).toEqual(tree.roots)

    const rows = elementRows(doc!.tree.nodes, doc!.tree.roots, null)
    expect(rows.length).toBeGreaterThan(5)
    expect(rows.some((r) => r.tag === 'row' || r.tag === 'column')).toBe(true)
    // layer badges present
    expect(rows.every((r) => r.layerBadge.length > 0)).toBe(true)
  }, 120_000)
})

describe('live Source load from Model.source', () => {
  it('openDocument without sourceText fills source from ops model', async () => {
    setOpsClient(cli)
    const disk = readFileSync(join(REPO, PAGE), 'utf8')
    const doc = await openDocument(PAGE, { loadPreview: false })
    expect(doc).not.toBeNull()
    expect(doc!.source.text.length).toBeGreaterThan(100)
    expect(doc!.source.text).toContain('<controller>')
    // Chrome shell markers (activity rail + status); surfaces live on leaf routes
    expect(doc!.source.text).toContain('data-sail-studio-shell')
    expect(doc!.source.text).toContain('data-studio-rail')
    // Matches disk (Model reads same file)
    expect(doc!.source.text).toBe(disk)
    expect(doc!.source.dirty).toBe(false)
    expect(doc!.source.status).toMatch(/Saved|saved/)
  }, 120_000)
})

describe('preview region click selects node', () => {
  it('hit-test painted regions → selection shared with explorer', async () => {
    setOpsClient(cli)
    unbind = bindEditSelection()
    const path = 'crates/sail/render/tests/fixtures/paint.sail'
    const doc = await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: true,
      viewport: { width: 400, height: 300 },
      theme: 'ship',
      mode: 'dark',
    })
    expect(doc).not.toBeNull()
    expect(doc!.preview.imagePng).toBeTruthy()
    expect(doc!.preview.regions.length).toBeGreaterThan(0)
    expect(doc!.preview.provenance).toMatch(/Audit snapshot/i)

    // Click center of the smallest region so hit-test is unambiguous.
    const region = [...doc!.preview.regions]
      .filter((r) => r.w > 4 && r.h > 4)
      .sort((a, b) => a.w * a.h - b.w * b.h)[0]!
    const cx = region.x + region.w / 2
    const cy = region.y + region.h / 2
    const expected = hitTestRegion(doc!.preview.regions, cx, cy)
    expect(expected?.nodeId).toBe(region.nodeId)
    const id = selectFromPreviewClick(cx, cy)
    expect(id).toBe(region.nodeId)
    expect(getSelection().nodeId).toBe(region.nodeId)
    expect(getEditState().selectionNodeId).toBe(region.nodeId)

    // Explorer rows reflect selection
    const rows = uiElementRows()
    const selected = rows.filter((r) => r.selected)
    expect(selected.some((r) => r.id === region.nodeId)).toBe(true)

    // Source scroll line set from span when node is in tree
    const inTree = doc!.tree.nodes.some((n) => n.id === region.nodeId)
    if (inTree) {
      expect(getEditState().sourceScrollLine).toBeTruthy()
      expect(getEditState().sourceScrollNonce).toBeGreaterThan(0)
    }

    // Hover tooltip
    const tip = hoverPreviewAt(cx, cy)
    expect(tip.visible).toBe(true)
    expect(tip.nodeId).toBe(region.nodeId)
    expect(tip.label).toMatch(/</)
  }, 180_000)

  it('hitTestRegion prefers smallest containing box', () => {
    const hit = hitTestRegion(
      [
        { nodeId: 'outer', x: 0, y: 0, w: 100, h: 100 },
        { nodeId: 'inner', x: 20, y: 20, w: 20, h: 20 },
      ],
      25,
      25,
    )
    expect(hit?.nodeId).toBe('inner')
  })
})

describe('source edit + save model rev', () => {
  it('apply write path bumps model rev and updates second subscriber', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'studio-edit-'))
    try {
      // Write a temp sail under repo-relative path via workspace...
      // Use Apply against a path under apps/studio/test/fixtures by copying.
      const rel = 'apps/studio/test/fixtures/_tmp-save.sail'
      const abs = join(REPO, rel)
      const original = `<column gap="1"><span>v1</span></column>\n`
      writeFileSync(abs, original)

      setOpsClient(cli)
      unbind = bindEditSelection()
      const revs: number[] = []
      const unsub = subscribeEdit((s) => revs.push(s.modelRev))

      await openDocument(rel, { sourceText: original, loadPreview: false })
      const revAfterOpen = getEditState().modelRev
      expect(revAfterOpen).toBeGreaterThan(0)

      setSourceText(rel, `<column gap="1"><span>v2-saved</span></column>\n`)
      expect(activeDoc()?.source.dirty).toBe(true)

      const rev = await saveActiveSource()
      expect(rev).toBeTruthy()
      expect(rev!).toBeGreaterThan(revAfterOpen)
      expect(activeDoc()?.source.dirty).toBe(false)
      expect(activeDoc()?.source.status).toMatch(/Saved|saved/)
      expect(getShellState().modelRev).toBe(getEditState().modelRev)

      // Disk reflects write
      const disk = readFileSync(abs, 'utf8')
      expect(disk).toContain('v2-saved')

      // Watch-style push to second open view
      const before = revs[revs.length - 1]!
      applyModelUpdate(rel, {
        tree: {
          roots: activeDoc()!.tree.roots,
          nodes: activeDoc()!.tree.nodes as unknown as unknown[],
        },
        modelVersion: (activeDoc()?.modelVersion ?? 1) + 1,
      })
      expect(getEditState().modelRev).toBeGreaterThan(before)
      unsub()

      // cleanup temp file
      try {
        rmSync(abs)
      } catch {
        /* */
      }
    } finally {
      try {
        rmSync(dir, { recursive: true })
      } catch {
        /* */
      }
    }
  }, 180_000)
})

describe('State view ticks readers match SailInspect/State', () => {
  // Heavy: sailInspectState + openDocument on sail-site page; needs headroom under full-suite load
  it.skipIf(!existsSync(join(REPO, SAIL_SITE_PAGE)))(
    'ticks binding readers/writers/ssr_tier',
    async () => {
      setOpsClient(cli)
      const stateRet = await cli.sailInspectState({ path: SAIL_SITE_PAGE })
      const ticks = (stateRet.bindings as { name: string }[]).find(
        (b) => b.name === 'ticks',
      ) as {
        name: string
        kind: string
        ssr_tier: string
        readers: unknown[]
        writers: unknown[]
      }
      expect(ticks).toBeTruthy()
      expect(ticks.kind).toBe('signal')
      expect(ticks.ssr_tier).toBe('C')
      expect(ticks.readers.length).toBeGreaterThanOrEqual(2)

      const doc = await openDocument(SAIL_SITE_PAGE, {
        sourceText: readFileSync(join(REPO, SAIL_SITE_PAGE), 'utf8'),
        loadPreview: false,
      })
      expect(doc).not.toBeNull()
      const cards = uiStateCards()
      const card = cards.find((c) => c.name === 'ticks')
      expect(card).toBeTruthy()
      expect(card!.ssrTier).toBe('C')
      expect(card!.readers.length).toBe(ticks.readers.length)
      expect(card!.kind).toBe('signal')
    },
    240_000,
  )
})

describe('SSR tier badges S / I / C on fixtures', () => {
  it('classifies pure=S, seed=I, ticks=C', async () => {
    setOpsClient(cli)
    const cases: { file: string; name: string; tier: string }[] = [
      { file: 'apps/studio/test/fixtures/tier-s.sail', name: 'pure', tier: 'S' },
      { file: 'apps/studio/test/fixtures/tier-i.sail', name: 'seed', tier: 'I' },
      { file: 'apps/studio/test/fixtures/tier-c.sail', name: 'ticks', tier: 'C' },
    ]

    for (const c of cases) {
      const st = await cli.sailInspectState({ path: c.file })
      const b = (st.bindings as { name: string; ssr_tier: string }[]).find(
        (x) => x.name === c.name,
      )
      expect(b, c.file).toBeTruthy()
      expect(b!.ssr_tier, `${c.file} ${c.name}`).toBe(c.tier)
    }

    // Full open path surfaces tiers on state cards (no sourceText — model load)
    await openDocument('apps/studio/test/fixtures/tier-c.sail', {
      loadPreview: false,
    })
    expect(uiStateCards().find((c) => c.name === 'ticks')?.ssrTier).toBe('C')

    await openDocument('apps/studio/test/fixtures/tier-s.sail', {
      loadPreview: false,
    })
    expect(uiStateCards().find((c) => c.name === 'pure')?.ssrTier).toBe('S')

    await openDocument('apps/studio/test/fixtures/tier-i.sail', {
      loadPreview: false,
    })
    const seedCard = uiStateCards().find((c) => c.name === 'seed')
    expect(seedCard?.ssrTier).toBe('I')
    expect(seedCard?.ssrTier).not.toBe('S')
  }, 180_000)
})

describe('live route + explorer tree groups (Unit 3)', () => {
  it('pathToLiveRoute maps routes/*.sail to served paths', () => {
    expect(pathToLiveRoute('apps/studio/routes/page.sail')).toBe('/')
    expect(pathToLiveRoute('apps/sail-site/routes/page.sail')).toBe('/')
    expect(pathToLiveRoute('apps/sail-site/routes/sail/tutorial/page.sail')).toBe(
      '/sail/tutorial',
    )
    expect(pathToLiveRoute('apps/studio/test/fixtures/tier-s.sail')).toBeNull()
    expect(pathToLiveRoute(null)).toBeNull()
    // Dynamic params are not live-iframe-able
    expect(pathToLiveRoute('apps/sail-site/routes/components/[tag]/page.sail')).toBeNull()
    expect(pathToLiveRoute('apps/x/routes/docs/[...slug]/page.sail')).toBeNull()
  })

  it('fileTreeGroups yields collapsible dir buckets with selection', () => {
    const groups = fileTreeGroups(
      [
        { path: 'apps/studio/routes/page.sail', name: 'page.sail', group: 'routes', checks: 0 },
        { path: 'apps/studio/src/edit/store.ts', name: 'store.ts', group: 'src', checks: 2 },
        { path: 'apps/studio/test/fixtures/a.sail', name: 'a.sail', group: 'other', checks: 0 },
      ],
      '',
      'apps/studio/routes/page.sail',
    )
    expect(groups.map((g) => g.id)).toEqual(['routes', 'src', 'other'])
    expect(groups[0]!.files[0]!.selected).toBe(true)
    expect(groups[1]!.files[0]!.hasChecks).toBe(true)
    // Disambiguate multi-page.sail via group-relative display name
    expect(
      fileDisplayName('apps/sail-site/routes/sail/tutorial/page.sail', 'routes'),
    ).toBe('sail/tutorial/page.sail')
  })

  it('nextTreeFocus moves selection for Arrow/Home/End', () => {
    const paths = ['a', 'b', 'c']
    expect(nextTreeFocus(paths, 'a', 'ArrowDown')).toBe('b')
    expect(nextTreeFocus(paths, 'b', 'ArrowUp')).toBe('a')
    expect(nextTreeFocus(paths, 'b', 'Home')).toBe('a')
    expect(nextTreeFocus(paths, 'b', 'End')).toBe('c')
    expect(nextTreeFocus(paths, 'c', 'ArrowDown')).toBe('c')
    expect(nextTreeFocus([], null, 'ArrowDown')).toBeNull()
  })
})

describe('selection sync explorer ↔ source span', () => {
  it('selectEditNode updates shell + scroll line and scrolls textarea', async () => {
    setOpsClient(cli)
    unbind = bindEditSelection()
    // Load source from Model.source only
    await openDocument(PAGE, { loadPreview: false })
    const rows = uiElementRows()
    const target = rows.find((r) => r.spanLine > 0) ?? rows[0]
    expect(target).toBeTruthy()
    selectEditNode(target!.id)
    expect(getSelection().nodeId).toBe(target!.id)
    expect(getEditState().sourceScrollLine).toBe(target!.spanLine)
    expect(getEditState().sourceScrollCol).toBeTruthy()
    expect(getEditState().sourceScrollNonce).toBeGreaterThan(0)

    // UI helper: caret moves to span line in textarea
    const ta = document.createElement('textarea')
    ta.value = activeDoc()!.source.text
    document.body.appendChild(ta)
    expect(
      applySourceEditorScroll(
        ta,
        getEditState().sourceScrollLine!,
        getEditState().sourceScrollCol ?? 1,
      ),
    ).toBe(true)
    expect(scrollSourceEditor(ta)).toBe(true)
    const line = getEditState().sourceScrollLine!
    // selectionStart should be near that line's offset
    const before = ta.value.split('\n').slice(0, line - 1).join('\n')
    const expectedStart = line > 1 ? before.length + 1 : 0
    expect(ta.selectionStart).toBeGreaterThanOrEqual(expectedStart - 1)
    expect(ta.selectionStart).toBeLessThanOrEqual(expectedStart + 80)

    // sail-code-editor host API path
    const cm = document.createElement('div') as HTMLElement & {
      setCursor?: (line: number, col?: number) => void
    }
    let cursored: [number, number?] | null = null
    cm.setCursor = (l, c) => {
      cursored = [l, c]
    }
    expect(applySourceEditorScroll(cm, line, 2)).toBe(true)
    expect(cursored).toEqual([line, 2])
    ta.remove()

    // Anatomy / tokens rails for selection
    selectEditNode(target!.id)
    expect(uiAnatomy().length).toBeGreaterThan(0)
  }, 120_000)
})

describe('STUDIO-1 review gates', () => {
  it('setView persists per document and restores on re-open', async () => {
    setOpsClient(cli)
    unbind = bindEditSelection()
    await openDocument(PAGE, { loadPreview: false })
    setView('state')
    expect(getEditState().view).toBe('state')
    expect(activeDoc()?.view).toBe('state')
    setView('layout')
    expect(activeDoc()?.view).toBe('layout')
    // Re-open same path keeps prior view
    await openDocument(PAGE, { loadPreview: false })
    expect(getEditState().view).toBe('layout')
    expect(activeDoc()?.view).toBe('layout')
  }, 120_000)

  it('writeSignalValue stores override and posts to live iframe origin', async () => {
    setOpsClient(cli)
    await openDocument(PAGE, { loadPreview: false })
    const iframe = document.createElement('iframe')
    iframe.setAttribute('data-live-preview', '')
    document.body.appendChild(iframe)
    const posts: { data: unknown; origin: string }[] = []
    const cw = {
      postMessage: (data: unknown, origin: string) => {
        posts.push({ data, origin })
      },
      location: { origin: window.location.origin },
    }
    Object.defineProperty(iframe, 'contentWindow', { value: cw, configurable: true })
    writeSignalValue('ticks', '42')
    expect(activeDoc()?.signalValues.ticks).toBe('42')
    expect(posts.length).toBe(1)
    expect(posts[0]!.data).toMatchObject({ type: 'sail-studio-signal', name: 'ticks', value: '42' })
    expect(posts[0]!.origin).toBe(window.location.origin)
    iframe.remove()
  }, 120_000)

  it('fileSailTreeNodes folds dirty + size meta into labels', () => {
    const nodes = fileSailTreeNodes(
      [
        { path: 'apps/x/routes/page.sail', name: 'page.sail', group: 'routes', checks: 2 },
        { path: 'apps/x/src/a.sail', name: 'a.sail', group: 'src', checks: 0 },
      ],
      '',
      'apps/x/routes/page.sail',
      {
        dirtyPaths: new Set(['apps/x/routes/page.sail']),
        sourceLen: (p) => (p.endsWith('page.sail') ? 2048 : null),
      },
    )
    expect(nodes[0]!.id).toBe('group:routes')
    expect(nodes[0]!.children![0]!.label).toMatch(/●/)
    expect(nodes[0]!.children![0]!.label).toMatch(/KB|2\.0/)
    expect(nodes[0]!.children![0]!.label).toMatch(/2/)
  })

  it('hoverPreviewAt skips emit when tip unchanged', async () => {
    setOpsClient(cli)
    unbind = bindEditSelection()
    const path = 'crates/sail/render/tests/fixtures/paint.sail'
    const { readFileSync } = await import('node:fs')
    const { join } = await import('node:path')
    await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: true,
      viewport: { width: 400, height: 300 },
      theme: 'ship',
      mode: 'dark',
    })
    const region = getEditState().docs.get(path)?.preview.regions?.[0]
    if (!region) return
    const cx = region.x + region.w / 2
    const cy = region.y + region.h / 2
    let emits = 0
    const unsub = subscribeEdit(() => {
      emits++
    })
    hoverPreviewAt(cx, cy)
    const afterFirst = emits
    hoverPreviewAt(cx, cy)
    hoverPreviewAt(cx, cy)
    // Second/third identical hover should not emit
    expect(emits).toBe(afterFirst)
    unsub()
  }, 180_000)
})
