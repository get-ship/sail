import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'
import { __resetSelectionForTests, getSelection, getShellState } from '@ship/sail-studio-shell'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
import {
  __previewGenForTests,
  __resetEditForTests,
  __setPreviewTimeoutForTests,
  activeDoc,
  applyModelUpdate,
  applySourceEditorScroll,
  applySourceScroll,
  bindEditSelection,
  elementRows,
  elementSailTreeNodes,
  getEditState,
  hitTestRegion,
  hoverPreviewAt,
  openDocument,
  parseTreeNodes,
  refreshPreview,
  saveActiveSource,
  scrollSourceEditor,
  selectEditNode,
  selectFromPreviewClick,
  setOpsClient,
  setSourceText,
  setView,
  subscribeEdit,
  pathToLiveRoute,
  pathToRoutePath,
  routeSailTreeNodes,
  fileTreeGroups,
  fileSailTreeNodes,
  nextTreeFocus,
  fileDisplayName,
  writeSignalValue,
  uiAnatomy,
  uiElementRows,
  uiStateCards,
  uiStructuredOps,
  type OpsClient,
} from '../src/edit'
import {
  buildRegionBoxes,
  ensurePreviewOnce,
  regionMetaLabel,
  retryPreviewOnce,
  type PreviewFlight,
} from '../src/edit-surface/preview-helpers'
import { paintRegionOverlay } from '../src/edit/region-overlay'

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

describe('C2 static preview helpers + region overlay', () => {
  it('regionMetaLabel covers zero / singular / plural', () => {
    expect(regionMetaLabel(0)).toBe('')
    expect(regionMetaLabel(1)).toBe('1 region')
    expect(regionMetaLabel(12)).toBe('12 regions')
  })

  it('paintRegionOverlay is a no-op (CSSOM paint removed; SAIL-LINT-CHAT)', () => {
    const host = document.createElement('div')
    host.setAttribute('data-region-overlay', '')
    document.body.appendChild(host)
    // Must not throw and must not inject DOM / style.
    paintRegionOverlay([
      {
        id: 'sel',
        left: '0%',
        top: '0%',
        width: '10%',
        height: '10%',
        selected: true,
        hover: false,
        key: 'sel',
      },
    ])
    expect(host.querySelectorAll('[data-region-box]').length).toBe(0)
    expect(host.children.length).toBe(0)
    host.remove()
  })

  it('buildRegionBoxes flags selected vs hover independently', async () => {
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
    expect(doc!.preview.regions.length).toBeGreaterThan(0)
    const region = doc!.preview.regions.find((r) => r.w > 0 && r.h > 0)!
    selectEditNode(region.nodeId)
    const boxes = buildRegionBoxes()
    const row = boxes.find((b) => b.id === region.nodeId)
    expect(row).toBeTruthy()
    expect(row!.selected).toBe(true)
    expect(row!.hover).toBe(false)
  }, 180_000)
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

describe('C3 source pane: ready/empty/dirty + scroll-to-span', () => {
  it('openDocument source is ready (has text) and not dirty', async () => {
    setOpsClient(cli)
    const doc = await openDocument(PAGE, { loadPreview: false })
    expect(doc).not.toBeNull()
    expect(doc!.source.text.length).toBeGreaterThan(0)
    expect(doc!.source.dirty).toBe(false)
    expect(doc!.source.status).toMatch(/Saved|saved|Loaded/i)
    // Empty only when bytes missing
    expect(!!doc!.source.text).toBe(true)
  }, 120_000)

  it('setSourceText marks dirty; save clears dirty; status cycle', async () => {
    const rel = 'apps/studio/test/fixtures/_tmp-c3-dirty.sail'
    const abs = join(REPO, rel)
    const original = `<column gap="1"><span>c3-a</span></column>\n`
    writeFileSync(abs, original)
    try {
      setOpsClient(cli)
      await openDocument(rel, { sourceText: original, loadPreview: false })
      expect(activeDoc()?.source.dirty).toBe(false)
      setSourceText(rel, `<column gap="1"><span>c3-b</span></column>\n`)
      expect(activeDoc()?.source.dirty).toBe(true)
      expect(activeDoc()?.source.status).toMatch(/Unsaved/i)
      const rev = await saveActiveSource()
      expect(rev).toBeTruthy()
      expect(activeDoc()?.source.dirty).toBe(false)
      expect(activeDoc()?.source.status).toMatch(/Saved|saved/)
    } finally {
      try {
        rmSync(abs)
      } catch {
        /* */
      }
    }
  }, 180_000)

  it('selectFromPreviewClick sets sourceScrollLine for CM scroll-to-span', async () => {
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
    const region = [...(doc!.preview.regions ?? [])]
      .filter((r) => r.w > 4 && r.h > 4)
      .sort((a, b) => a.w * a.h - b.w * b.h)[0]
    if (!region) return
    const cx = region.x + region.w / 2
    const cy = region.y + region.h / 2
    selectFromPreviewClick(cx, cy)
    const inTree = doc!.tree.nodes.some((n) => n.id === region.nodeId)
    if (inTree) {
      expect(getEditState().sourceScrollLine).toBeTruthy()
      expect(getEditState().sourceScrollNonce).toBeGreaterThan(0)
      // setCursor host path still applies after C3 chrome
      const cm = document.createElement('div') as HTMLElement & {
        setCursor?: (line: number, col?: number) => void
      }
      let cursored: [number, number?] | null = null
      cm.setCursor = (l, c) => {
        cursored = [l, c]
      }
      expect(
        applySourceEditorScroll(
          cm,
          getEditState().sourceScrollLine!,
          getEditState().sourceScrollCol ?? 1,
        ),
      ).toBe(true)
      expect(cursored?.[0]).toBe(getEditState().sourceScrollLine)
    }
  }, 180_000)
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

  it('pathToRoutePath maps pages to route shapes including dynamics', () => {
    expect(pathToRoutePath('apps/studio/routes/page.sail')).toBe('/')
    expect(pathToRoutePath('apps/studio/routes/edit/page.sail')).toBe('/edit')
    expect(pathToRoutePath('apps/sail-site/routes/sail/tutorial/page.sail')).toBe(
      '/sail/tutorial',
    )
    expect(pathToRoutePath('apps/sail-site/routes/components/[tag]/page.sail')).toBe(
      '/components/[tag]',
    )
    expect(pathToRoutePath('apps/x/routes/docs/[...slug]/page.sail')).toBe(
      '/docs/[...slug]',
    )
    expect(pathToRoutePath('apps/studio/routes/_layout.sail')).toBeNull()
    expect(pathToRoutePath('apps/x/src/a.sail')).toBeNull()
    expect(pathToRoutePath(null)).toBeNull()
  })

  it('routeSailTreeNodes builds route hierarchy with path labels', () => {
    const nodes = routeSailTreeNodes(
      [
        {
          path: 'apps/x/routes/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/edit/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/components/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/components/[tag]/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/sail/tutorial/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/_layout.sail',
          name: '_layout.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/src/a.sail',
          name: 'a.sail',
          group: 'src',
          checks: 0,
        },
      ],
      '',
    )
    // Top-level: index + first segments (not raw file paths)
    expect(nodes.map((n) => n.label)).toEqual([
      '/',
      '/components',
      '/edit',
      '/sail',
    ])
    expect(nodes[0]!.id).toBe('apps/x/routes/page.sail')
    expect(nodes[1]!.id).toBe('apps/x/routes/components/page.sail')
    expect(nodes[1]!.children![0]!.label).toBe('/components/[tag]')
    expect(nodes[1]!.children![0]!.id).toBe(
      'apps/x/routes/components/[tag]/page.sail',
    )
    // Intermediate segment without page.sail is expand-only meta id
    expect(nodes[3]!.label).toBe('/sail')
    expect(nodes[3]!.id).toBe('route:/sail')
    expect(nodes[3]!.children![0]!.label).toBe('/sail/tutorial')
    expect(nodes[3]!.children![0]!.id).toBe(
      'apps/x/routes/sail/tutorial/page.sail',
    )
  })

  it('routeSailTreeNodes dirty dot + filter by route path', () => {
    const files = [
      {
        path: 'apps/x/routes/page.sail',
        name: 'page.sail',
        group: 'routes' as const,
        checks: 0,
      },
      {
        path: 'apps/x/routes/edit/page.sail',
        name: 'page.sail',
        group: 'routes' as const,
        checks: 0,
      },
      {
        path: 'apps/x/routes/sail/tutorial/page.sail',
        name: 'page.sail',
        group: 'routes' as const,
        checks: 0,
      },
    ]
    const dirty = routeSailTreeNodes(files, '', {
      dirtyPaths: new Set(['apps/x/routes/edit/page.sail']),
    })
    expect(dirty.find((n) => n.id.endsWith('/edit/page.sail'))!.label).toBe(
      '● /edit',
    )
    expect(dirty.find((n) => n.label === '/')!.label).toBe('/')

    const filtered = routeSailTreeNodes(files, 'tutorial')
    expect(filtered.map((n) => n.label)).toEqual(['/sail'])
    expect(filtered[0]!.children![0]!.label).toBe('/sail/tutorial')
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

  it('writeSignalValue stores override and posts to registered live preview window', async () => {
    setOpsClient(cli)
    await openDocument(PAGE, { loadPreview: false })
    const { setLivePreviewWindow } = await import('../src/edit/store')
    const posts: { data: unknown; origin: string }[] = []
    const cw = {
      postMessage: (data: unknown, origin: string) => {
        posts.push({ data, origin })
      },
      location: { origin: window.location.origin },
    } as unknown as Window
    setLivePreviewWindow(cw)
    try {
      writeSignalValue('ticks', '42')
      expect(activeDoc()?.signalValues.ticks).toBe('42')
      expect(posts.length).toBe(1)
      expect(posts[0]!.data).toMatchObject({
        type: 'sail-studio-signal',
        name: 'ticks',
        value: '42',
      })
      expect(posts[0]!.origin).toBe(window.location.origin)
    } finally {
      setLivePreviewWindow(null)
    }
  }, 120_000)

  it('elementSailTreeNodes: short labels, layer badge in trailing (not label cram)', () => {
    const nodes = parseTreeNodes([
      {
        id: 'root',
        tag: 'column',
        layer: 'l1-keyword',
        children: ['btn', 'pack'],
        span: { line: 1, col: 1 },
      },
      {
        id: 'btn',
        tag: 'button',
        layer: 'l2-component',
        attrs: { name: 'save' },
        children: [],
        span: { line: 2, col: 3 },
      },
      {
        id: 'pack',
        tag: 'edit-explorer',
        layer: 'userland',
        children: ['cf'],
        span: { line: 3, col: 3 },
      },
      {
        id: 'cf',
        tag: 'template',
        layer: 'l1-keyword',
        control_flow: { kind: 'if', expr: 'open' },
        children: [],
        span: { line: 4, col: 5 },
      },
    ])
    const tree = elementSailTreeNodes(nodes, ['root'])
    expect(tree).toHaveLength(1)
    expect(tree[0]!.label).toBe('<column>')
    expect(tree[0]!.trailing).toBe('L1')
    expect(tree[0]!.label).not.toMatch(/L1|·/)
    const btn = tree[0]!.children!.find((c) => c.id === 'btn')!
    expect(btn.label).toBe('<button> save')
    expect(btn.trailing).toBe('L2')
    expect(btn.icon).toBe('component')
    const pack = tree[0]!.children!.find((c) => c.id === 'pack')!
    expect(pack.label).toBe('<edit-explorer>')
    expect(pack.trailing).toBe('L3')
    expect(pack.icon).toBe('box')
    const cf = pack.children![0]!
    expect(cf.label).toBe('<if> open')
    expect(cf.trailing).toBe('L1')
    // No badge/meta crammed into labels
    for (const n of [tree[0]!, btn, pack, cf]) {
      expect(n.label).not.toMatch(/·\s*L[123]/)
    }
  })

  it('fileSailTreeNodes: nested dirs, short labels, dirty in trailing (not label)', () => {
    const nodes = fileSailTreeNodes(
      [
        { path: 'apps/x/routes/page.sail', name: 'page.sail', group: 'routes', checks: 2 },
        {
          path: 'apps/x/routes/edit/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/sail/tutorial/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        { path: 'apps/x/src/a.sail', name: 'a.sail', group: 'src', checks: 0 },
      ],
      '',
      'apps/x/routes/page.sail',
      {
        dirtyPaths: new Set(['apps/x/routes/page.sail', 'apps/x/src/a.sail']),
      },
    )
    expect(nodes.map((n) => n.id)).toEqual(['group:routes', 'group:src'])
    // Group-relative nesting: edit/ and sail/ dirs + root page.sail
    const routes = nodes[0]!.children!
    const routeLabels = routes.map((n) => n.label)
    expect(routeLabels).toContain('edit')
    expect(routeLabels).toContain('sail')
    expect(routeLabels).toContain('page.sail')
    const rootPage = routes.find((n) => n.id === 'apps/x/routes/page.sail')!
    expect(rootPage.label).toBe('page.sail')
    expect(rootPage.trailing).toBe('●')
    // No size / checks meta crammed into labels
    expect(rootPage.label).not.toMatch(/KB|·|●/)
    const editDir = routes.find((n) => n.id === 'dir:routes/edit')!
    expect(editDir.label).toBe('edit')
    expect(editDir.children![0]!.id).toBe('apps/x/routes/edit/page.sail')
    expect(editDir.children![0]!.label).toBe('page.sail')
    expect(editDir.children![0]!.trailing).toBeUndefined()
    const sailDir = routes.find((n) => n.id === 'dir:routes/sail')!
    expect(sailDir.children![0]!.id).toBe('dir:routes/sail/tutorial')
    expect(sailDir.children![0]!.children![0]!.id).toBe(
      'apps/x/routes/sail/tutorial/page.sail',
    )
    const srcLeaf = nodes[1]!.children![0]!
    expect(srcLeaf.label).toBe('a.sail')
    expect(srcLeaf.trailing).toBe('●')
  })

  it('fileSailTreeNodes filter keeps path ancestors under group roots', () => {
    const files = [
      { path: 'apps/x/routes/page.sail', name: 'page.sail', group: 'routes', checks: 0 },
      {
        path: 'apps/x/routes/edit/page.sail',
        name: 'page.sail',
        group: 'routes',
        checks: 0,
      },
      {
        path: 'apps/x/routes/sail/tutorial/page.sail',
        name: 'page.sail',
        group: 'routes',
        checks: 0,
      },
      { path: 'apps/x/src/a.sail', name: 'a.sail', group: 'src', checks: 0 },
    ]
    const filtered = fileSailTreeNodes(files, 'tutorial', null)
    expect(filtered.map((n) => n.id)).toEqual(['group:routes'])
    const sail = filtered[0]!.children![0]!
    expect(sail.id).toBe('dir:routes/sail')
    expect(sail.children![0]!.id).toBe('dir:routes/sail/tutorial')
    expect(sail.children![0]!.children![0]!.id).toBe(
      'apps/x/routes/sail/tutorial/page.sail',
    )
  })

  it('fileSailTreeNodes disambiguates multi-page.sail via parent dir rows (never flat siblings)', () => {
    const nodes = fileSailTreeNodes(
      [
        { path: 'apps/x/routes/page.sail', name: 'page.sail', group: 'routes', checks: 0 },
        {
          path: 'apps/x/routes/edit/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
        {
          path: 'apps/x/routes/checks/page.sail',
          name: 'page.sail',
          group: 'routes',
          checks: 0,
        },
      ],
      '',
      'apps/x/routes/edit/page.sail',
    )
    const routes = nodes.find((n) => n.id === 'group:routes')!.children!
    // Nested dirs — not three bare page.sail siblings under the group.
    const topLabels = routes.map((n) => n.label).sort()
    expect(topLabels).toEqual(['checks', 'edit', 'page.sail'])
    const editPage = routes.find((n) => n.id === 'dir:routes/edit')!.children![0]!
    expect(editPage.id).toBe('apps/x/routes/edit/page.sail')
    expect(editPage.label).toBe('page.sail')
    // Path-relative display names under group for flat consumers.
    expect(fileDisplayName('apps/x/routes/edit/page.sail', 'routes')).toBe('edit/page.sail')
    expect(fileDisplayName('apps/x/routes/checks/page.sail', 'routes')).toBe(
      'checks/page.sail',
    )
  })

  it('fileSailTreeNodes nests non-sail under other (project-relative, not bare basename only)', () => {
    const nodes = fileSailTreeNodes(
      [
        { path: 'apps/x/package.json', name: 'package.json', group: 'other', checks: 0 },
        { path: 'apps/x/test/foo.ts', name: 'foo.ts', group: 'other', checks: 0 },
        { path: 'apps/x/test/bar.ts', name: 'bar.ts', group: 'other', checks: 0 },
        { path: 'apps/x/src/main.ts', name: 'main.ts', group: 'src', checks: 0 },
      ],
      '',
      null,
    )
    expect(nodes.map((n) => n.id).sort()).toEqual(['group:other', 'group:src'])
    const other = nodes.find((n) => n.id === 'group:other')!.children!
    expect(other.map((n) => n.label).sort()).toEqual(['package.json', 'test'])
    const testDir = other.find((n) => n.id === 'dir:other/test')!
    expect(testDir.children!.map((c) => c.label).sort()).toEqual(['bar.ts', 'foo.ts'])
    expect(testDir.children!.map((c) => c.id).sort()).toEqual([
      'apps/x/test/bar.ts',
      'apps/x/test/foo.ts',
    ])
    expect(fileDisplayName('apps/x/test/foo.ts', 'other')).toBe('test/foo.ts')
    expect(fileDisplayName('apps/x/package.json', 'other')).toBe('package.json')
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

describe('preview failure surfaces error + retry (no infinite opening)', () => {
  const path = 'crates/sail/render/tests/fixtures/paint.sail'

  afterEach(() => {
    __setPreviewTimeoutForTests(null)
  })

  function mockClient(
    screenshot: OpsClient['sailRenderScreenshot'],
  ): OpsClient {
    return {
      ...cli,
      sailRenderScreenshot: screenshot,
    }
  }

  it('openDocument with rejecting screenshot sets preview.error and not loading', async () => {
    setOpsClient(
      mockClient(async () => {
        throw new Error('render boom')
      }),
    )
    const doc = await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: true,
      viewport: { width: 400, height: 300 },
    })
    expect(doc).not.toBeNull()
    expect(doc!.preview.imagePng).toBeFalsy()
    expect(doc!.preview.loading).toBe(false)
    expect(doc!.preview.error).toMatch(/Could not render the audit snapshot/)
  }, 60_000)

  it('hanging screenshot times out — never leaves loading true', async () => {
    __setPreviewTimeoutForTests(80)
    setOpsClient(
      mockClient(
        () =>
          new Promise(() => {
            /* never settles */
          }),
      ),
    )
    const doc = await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: true,
      viewport: { width: 400, height: 300 },
    })
    expect(doc).not.toBeNull()
    expect(doc!.preview.loading).toBe(false)
    expect(doc!.preview.error).toBeTruthy()
    expect(doc!.preview.error).toMatch(/timed out|try again/i)
  }, 30_000)

  it('ensurePreviewOnce + retryPreviewOnce: fail sticky then reload on retry', async () => {
    let calls = 0
    setOpsClient(
      mockClient(async () => {
        calls++
        if (calls === 1) throw new Error('first fail')
        return {
          dynamics: [],
          imagePng: 'iVBORw0KGgo=',
          ms: 1,
          regions: [],
          theme: 'ship',
          viewport: { width: 400, height: 300 },
        }
      }),
    )
    // Open without preview; surface will load via ensure/refresh
    const doc = await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: false,
    })
    expect(doc).not.toBeNull()
    setView('preview')

    const flight: PreviewFlight = { inflight: null, failedPath: null }
    await new Promise<void>((resolve) => {
      ensurePreviewOnce(flight, resolve)
    })
    expect(activeDoc()?.preview.loading).toBe(false)
    expect(activeDoc()?.preview.error).toBeTruthy()
    expect(flight.failedPath).toBe(path)
    expect(calls).toBe(1)

    // Sticky: second ensure must not re-storm
    let secondDone = false
    ensurePreviewOnce(flight, () => {
      secondDone = true
    })
    expect(secondDone).toBe(false)
    expect(calls).toBe(1)

    await new Promise<void>((resolve) => {
      retryPreviewOnce(flight, resolve)
    })
    expect(calls).toBe(2)
    expect(activeDoc()?.preview.imagePng).toBeTruthy()
    expect(activeDoc()?.preview.error).toBeNull()
    expect(activeDoc()?.preview.loading).toBe(false)
    expect(flight.failedPath).toBeNull()
  }, 60_000)

  it('refreshPreview sets loading then settles with error on failure', async () => {
    setOpsClient(
      mockClient(async () => {
        throw new Error('nope')
      }),
    )
    await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: false,
    })
    const seenLoading: boolean[] = []
    const unsub = subscribeEdit((s) => {
      const p = s.docs.get(path)?.preview
      if (p) seenLoading.push(p.loading)
    })
    await refreshPreview({ viewport: { width: 400, height: 300 } })
    unsub()
    expect(seenLoading.some((v) => v === true)).toBe(true)
    expect(activeDoc()?.preview.loading).toBe(false)
    expect(activeDoc()?.preview.error).toBeTruthy()
  }, 60_000)

  it('structured edit ops are empty (rail hidden — no dead stubs)', () => {
    expect(uiStructuredOps()).toEqual([])
  })

  it('concurrent refreshPreview: only latest gen applies (stale settle ignored)', async () => {
    let resolveSlow: ((v: {
      dynamics: unknown[]
      imagePng: string
      ms: number
      regions: unknown[]
      theme: string
      viewport: unknown
    }) => void) | null = null
    let calls = 0
    setOpsClient(
      mockClient(() => {
        calls++
        if (calls === 1) {
          return new Promise((resolve) => {
            resolveSlow = resolve
          })
        }
        return Promise.resolve({
          dynamics: [],
          imagePng: 'fast-png',
          ms: 1,
          regions: [],
          theme: 'ship',
          viewport: { width: 400, height: 300 },
        })
      }),
    )
    await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: false,
    })
    const genBefore = __previewGenForTests()
    const slow = refreshPreview({ viewport: { width: 400, height: 300 } })
    const genSlow = __previewGenForTests()
    expect(genSlow).toBeGreaterThan(genBefore)
    // Supersede with a fast second refresh (retry path).
    const fastResult = await refreshPreview({ viewport: { width: 400, height: 300 } })
    expect(fastResult.applied).toBe(true)
    expect(activeDoc()?.preview.imagePng).toBe('fast-png')
    expect(activeDoc()?.preview.error).toBeNull()
    expect(activeDoc()?.preview.loading).toBe(false)
    const genFast = __previewGenForTests()
    expect(genFast).toBeGreaterThan(genSlow)
    // Late slow settle must not overwrite the fast result.
    resolveSlow!({
      dynamics: [],
      imagePng: 'slow-stale-png',
      ms: 99,
      regions: [],
      theme: 'ship',
      viewport: { width: 400, height: 300 },
    })
    const slowResult = await slow
    expect(slowResult.applied).toBe(false)
    expect(activeDoc()?.preview.imagePng).toBe('fast-png')
    expect(activeDoc()?.preview.imagePng).not.toBe('slow-stale-png')
    expect(__previewGenForTests()).toBe(genFast)
  }, 60_000)

  it('concurrent openDocument: superseded open does not clobber newer activePath', async () => {
    const pathA = path
    const pathB = PAGE
    let releaseA: (() => void) | null = null
    const gateA = new Promise<void>((r) => {
      releaseA = r
    })
    let aEntered = false
    setOpsClient({
      ...cli,
      sailInspectModel: async (args) => {
        if (args.path === pathA && !aEntered) {
          aEntered = true
          await gateA
        }
        return cli.sailInspectModel(args)
      },
    })
    const slowA = openDocument(pathA, {
      sourceText: readFileSync(join(REPO, pathA), 'utf8'),
      loadPreview: false,
    })
    // Let A block inside sailInspectModel
    await new Promise((r) => setTimeout(r, 20))
    const genDuringA = __previewGenForTests()
    const fastB = await openDocument(pathB, { loadPreview: false })
    expect(fastB).not.toBeNull()
    expect(getEditState().activePath).toBe(pathB)
    expect(__previewGenForTests()).toBeGreaterThan(genDuringA)

    releaseA!()
    const aResult = await slowA
    expect(aResult).toBeNull()
    expect(getEditState().activePath).toBe(pathB)
    expect(activeDoc()?.path).toBe(pathB)
  }, 120_000)

  it('concurrent openDocument: supersede after install does not refresh wrong path', async () => {
    const pathA = path
    const pathB = PAGE
    const shotPaths: string[] = []
    setOpsClient({
      ...cli,
      sailRenderScreenshot: async (args) => {
        shotPaths.push(String(args.path ?? getEditState().activePath ?? ''))
        return {
          dynamics: [],
          imagePng: `png-for-${args.path}`,
          ms: 1,
          regions: [],
          theme: 'ship',
          viewport: { width: 400, height: 300 },
        }
      },
    })

    // Re-enter open B from A’s install emit — bumps gen before A’s post-install
    // refreshPreview (which reads state.activePath and would target B).
    let bPromise: Promise<Awaited<ReturnType<typeof openDocument>>> | null = null
    let kicked = false
    const unsub = subscribeEdit(() => {
      if (kicked) return
      const st = getEditState()
      if (st.activePath === pathA && st.docs.has(pathA) && !st.loading) {
        kicked = true
        bPromise = openDocument(pathB, { loadPreview: false })
      }
    })

    const aResult = await openDocument(pathA, {
      sourceText: readFileSync(join(REPO, pathA), 'utf8'),
      loadPreview: true,
      viewport: { width: 400, height: 300 },
    })
    unsub()

    expect(kicked).toBe(true)
    expect(aResult).toBeNull()
    expect(bPromise).not.toBeNull()
    const bResult = await bPromise!
    expect(bResult).not.toBeNull()
    expect(getEditState().activePath).toBe(pathB)
    // A must not have started a screenshot for the stolen active path B.
    expect(shotPaths).not.toContain(pathB)
  }, 120_000)

  it('flight bookkeeping: superseded refresh does not sticky-fail or clear newer inflight', async () => {
    let resolveSlow: ((v: {
      dynamics: unknown[]
      imagePng: string
      ms: number
      regions: unknown[]
      theme: string
      viewport: unknown
    }) => void) | null = null
    let calls = 0
    setOpsClient(
      mockClient(() => {
        calls++
        if (calls === 1) {
          return new Promise((resolve) => {
            resolveSlow = resolve
          })
        }
        return Promise.resolve({
          dynamics: [],
          imagePng: 'winner-png',
          ms: 1,
          regions: [],
          theme: 'ship',
          viewport: { width: 400, height: 300 },
        })
      }),
    )
    await openDocument(path, {
      sourceText: readFileSync(join(REPO, path), 'utf8'),
      loadPreview: false,
    })
    setView('preview')
    const flight: PreviewFlight = { inflight: null, failedPath: null }

    const firstDone = new Promise<void>((resolve) => {
      ensurePreviewOnce(flight, resolve)
    })
    expect(flight.inflight).toBe(path)

    // Retry while first is in flight — clears then re-sets inflight for winner.
    const secondDone = new Promise<void>((resolve) => {
      retryPreviewOnce(flight, resolve)
    })
    expect(flight.failedPath).toBeNull()
    expect(flight.inflight).toBe(path)

    await secondDone
    expect(activeDoc()?.preview.imagePng).toBe('winner-png')
    expect(flight.failedPath).toBeNull()
    expect(flight.inflight).toBeNull()

    // Late first settle must not sticky-fail after success.
    resolveSlow!({
      dynamics: [],
      imagePng: 'stale-png',
      ms: 1,
      regions: [],
      theme: 'ship',
      viewport: { width: 400, height: 300 },
    })
    await firstDone
    expect(flight.failedPath).toBeNull()
    expect(activeDoc()?.preview.imagePng).toBe('winner-png')
  }, 60_000)
})
