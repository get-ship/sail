import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import { existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'
import {
  __resetSelectionForTests,
  getSelection,
  getShellState,
  __resetShellForTests,
} from '@ship/sail-studio-shell'
import { createCliOpsClient } from '../src/edit/ops-adapter-cli'
import {
  APPLY_FIX_READY,
  __resetChecksForTests,
  applyFix,
  applyFixReview,
  applyFixStub,
  applyRuleFixes,
  autoFixOnSave,
  bothViewportsRan,
  buildFixReviewItems,
  editorDeepLink,
  getChecksState,
  groupByRule,
  groupBySeverity,
  isValidEditorDeepLink,
  loadFindings,
  openFixReview,
  openSourceUrlFor,
  parseCheckFindings,
  parseEditorSetting,
  recheckFindings,
  selectFinding,
  selectInApp,
  setAutoFixMode,
  setChecksOpsClient,
  setEditorSetting,
  setFixReviewAccepted,
  setProjectRoot,
  sourceForSave,
  uiEmptyCheckedSummary,
  uiFindingRows,
  uiFixableCount,
  uiRuleGroups,
  uiSeverityGroups,
  type ChecksOpsClient,
} from '../src/checks'
import {
  __resetEditForTests,
  getEditState,
  seedSourceText,
  setOpsClient as setEditOpsClient,
} from '../src/edit'
import type { OpsClient } from '../src/edit/types'

const REPO = join(dirname(fileURLToPath(import.meta.url)), '../../..')
const SAIL_SITE_PAGE = 'apps/sail-site/routes/page.sail'

let cli: ChecksOpsClient

beforeAll(() => {
  cli = createCliOpsClient(REPO) as ChecksOpsClient
}, 120_000)

afterEach(() => {
  __resetChecksForTests()
  __resetEditForTests()
  __resetSelectionForTests()
  __resetShellForTests()
})

describe('findings match SailAuthoring/Check 1:1 (sail-site)', () => {
  // Heavy: sailAuthoringCheck + loadFindings on sail-site page; needs headroom under full-suite load
  it.skipIf(!existsSync(join(REPO, SAIL_SITE_PAGE)))(
    'loadFindings for sail-site page equals Check diagnostics',
    async () => {
      setChecksOpsClient(cli)
      setProjectRoot(REPO)
      const chk = await cli.sailAuthoringCheck({ path: SAIL_SITE_PAGE })
      const findings = await loadFindings(SAIL_SITE_PAGE)
      expect(findings.length).toBe(chk.count)
      expect(findings.length).toBe((chk.diagnostics ?? []).length)

      const fromParse = parseCheckFindings(chk.diagnostics ?? [], {
        defaultPath: SAIL_SITE_PAGE,
      })
      // Same rule/line/message set
      const key = (f: { rule: string; line: number; message: string }) =>
        `${f.rule}|${f.line}|${f.message}`
      expect(findings.map(key).sort()).toEqual(fromParse.map(key).sort())
    },
    240_000,
  )

  it('inline source with participation violation surfaces 1:1', async () => {
    setChecksOpsClient(cli)
    const source = '<div sx={style.grow(1)}>x</div>\n'
    const chk = await cli.sailAuthoringCheck({ source })
    expect(chk.count).toBeGreaterThan(0)
    // Use a synthetic path + source so load path works without disk write
    const path = 'inline://participation.sail'
    const mock: ChecksOpsClient = {
      sailAuthoringCheck: async (args) => {
        if (args.source) return cli.sailAuthoringCheck(args)
        return cli.sailAuthoringCheck({ source })
      },
      sailAuthoringLayoutCheck: async () => ({
        ok: true,
        viewports: [
          { viewport: { width: 390, height: 844 } },
          { viewport: { width: 1440, height: 900 } },
        ],
        diagnostics: [],
      }),
    }
    setChecksOpsClient(mock)
    const findings = await loadFindings(path, { source })
    expect(findings.length).toBe(chk.count)
    expect(findings.some((f) => f.rule === 'sail-participation-style')).toBe(
      true,
    )
    expect(findings.some((f) => f.hasFix)).toBe(true)
    const rows = uiFindingRows()
    expect(rows.some((r) => r.showFixChip)).toBe(true)
    expect(rows.every((r) => r.showApply === (r.hasFix && APPLY_FIX_READY))).toBe(
      true,
    )
  }, 120_000)
})

describe('both viewports badge', () => {
  it('bothViewportsRan true only when ≥2 viewports without error', () => {
    expect(
      bothViewportsRan({
        viewports: [
          { viewport: { width: 390, height: 844 }, box_count: 1 },
          { viewport: { width: 1440, height: 900 }, box_count: 1 },
        ],
      }),
    ).toBe(true)
    expect(
      bothViewportsRan({
        viewports: [{ viewport: { width: 390, height: 844 } }],
      }),
    ).toBe(false)
    expect(
      bothViewportsRan({
        viewports: [
          { viewport: { width: 390, height: 844 } },
          { viewport: { width: 1440, height: 900 }, error: 'taffy boom' },
        ],
      }),
    ).toBe(false)
    expect(bothViewportsRan({ viewports: [] })).toBe(false)
    expect(bothViewportsRan(null)).toBe(false)
  })

  it('layout finding gets both-viewports only when LayoutCheck ran both', async () => {
    const layoutSrc = `<column gap="3" sx={style.h(0)}>
  <span>a</span>
  <div>b</div>
</column>
`
    const path = 'inline://layout.sail'

    // Both viewports OK
    setChecksOpsClient({
      sailAuthoringCheck: (args) => cli.sailAuthoringCheck(args),
      sailAuthoringLayoutCheck: async () => ({
        ok: true,
        viewports: [
          { viewport: { width: 390, height: 844 }, box_count: 3 },
          { viewport: { width: 1440, height: 900 }, box_count: 3 },
        ],
        diagnostics: [],
      }),
    })
    let findings = await loadFindings(path, { source: layoutSrc })
    const layoutOk = findings.filter((f) => f.isLayout)
    expect(layoutOk.length).toBeGreaterThan(0)
    expect(layoutOk.every((f) => f.bothViewports)).toBe(true)
    expect(uiFindingRows().some((r) => r.showBothViewports)).toBe(true)

    __resetChecksForTests()

    // Only one viewport / error → no badge
    setChecksOpsClient({
      sailAuthoringCheck: (args) => cli.sailAuthoringCheck(args),
      sailAuthoringLayoutCheck: async () => ({
        ok: true,
        viewports: [
          { viewport: { width: 390, height: 844 } },
          { error: 'failed' },
        ],
        diagnostics: [],
      }),
    })
    findings = await loadFindings(path, { source: layoutSrc })
    const layoutBad = findings.filter((f) => f.isLayout)
    expect(layoutBad.length).toBeGreaterThan(0)
    expect(layoutBad.every((f) => f.bothViewports === false)).toBe(true)
    expect(uiFindingRows().every((r) => !r.showBothViewports || !r.isLayout)).toBe(
      true,
    )
  }, 120_000)
})

describe('editor deep-link', () => {
  it('builds vscode/cursor/zed URLs at line:col', () => {
    const vscode = parseEditorSetting('vscode')
    const url = editorDeepLink(
      vscode,
      '/tmp/repo/apps/studio/routes/page.sail',
      88,
      3,
    )
    expect(url).toBe(
      'vscode://file//tmp/repo/apps/studio/routes/page.sail:88:3',
    )
    expect(isValidEditorDeepLink(url)).toBe(true)

    const cursor = editorDeepLink(
      parseEditorSetting('cursor'),
      '/tmp/x.sail',
      12,
      1,
    )
    expect(cursor).toBe('cursor://file//tmp/x.sail:12:1')

    const zed = editorDeepLink(parseEditorSetting('zed'), '/tmp/x.sail', 1, 1)
    expect(zed).toBe('zed://file//tmp/x.sail:1:1')

    const custom = parseEditorSetting({
      url: 'vscode://file/{path}:{line}:{col}',
    })
    expect(editorDeepLink(custom, '/a.sail', 2, 4)).toBe(
      'vscode://file//a.sail:2:4',
    )
  })

  it('openSourceUrlFor uses project root + selected finding span', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'style.grow — use grow attr\n  suggest: use grow',
            line: 34,
            col: 5,
            path: 'apps/studio/routes/page.sail',
          },
        ],
      }),
    })
    setProjectRoot(REPO)
    setEditorSetting('vscode')
    await loadFindings('apps/studio/routes/page.sail')
    selectFinding(getChecksState().findings[0]!.id)
    const url = openSourceUrlFor()
    expect(url).toContain('vscode://file/')
    expect(url).toContain('page.sail:34:5')
    expect(isValidEditorDeepLink(url)).toBe(true)
  })
})

describe('auto-fix off leaves disk untouched', () => {
  it('autoFixOnSave off/suggest never rewrites', () => {
    const src = '<div sx={style.grow(1)}>x</div>\n'
    const findings = parseCheckFindings(
      [
        {
          severity: 'error',
          rule: 'sail-participation-style',
          message: 'style.grow\n  suggest: use grow attr',
          line: 1,
          col: 1,
        },
      ],
      { defaultPath: 't.sail' },
    )
    expect(autoFixOnSave(src, findings, 'off')).toEqual({
      text: src,
      changed: false,
      applied: 0,
    })
    expect(autoFixOnSave(src, findings, 'suggest')).toEqual({
      text: src,
      changed: false,
      applied: 0,
    })
    const applied = autoFixOnSave(src, findings, 'apply')
    expect(applied.changed).toBe(true)
    expect(applied.text).toContain('grow')
    expect(applied.text).not.toContain('style.grow')
  })

  it('sourceForSave with mode off does not alter bytes on a temp file path', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'tool6-autofix-'))
    const file = join(dir, 't.sail')
    const original = '<div sx={style.grow(1)}>keep-me</div>\n'
    writeFileSync(file, original, 'utf8')
    try {
      setChecksOpsClient(cli)
      setAutoFixMode('off')
      await loadFindings(file, { source: original })
      const before = readFileSync(file, 'utf8')
      const result = sourceForSave(file, before)
      expect(result.changed).toBe(false)
      expect(result.text).toBe(before)
      // Disk still original (we never wrote)
      expect(readFileSync(file, 'utf8')).toBe(original)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  }, 60_000)
})

describe('apply fix ready', () => {
  it('APPLY_FIX_READY is true and applyFixStub retained for compatibility', () => {
    expect(APPLY_FIX_READY).toBe(true)
    const f = parseCheckFindings(
      [
        {
          severity: 'error',
          rule: 'sail-participation-style',
          message: 'x\n  suggest: y',
          line: 1,
          col: 1,
        },
      ],
      { defaultPath: 't.sail' },
    )[0]!
    const stub = applyFixStub(f)
    expect(stub.ok).toBe(false)
    expect(stub.status).toBe('deferred')
  })

  it('applyFix on store applies participation-style rewrite', async () => {
    const src = '<div sx={style.grow(1)}>x</div>'
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow\n  suggest: use attr',
            line: 1,
            col: 1,
            path: 't.sail',
          },
        ],
      }),
      readSource: async () => src,
    })
    await loadFindings('t.sail', { source: src })
    const ret = applyFix()
    expect(ret.ok).toBe(true)
    if (ret.ok) {
      expect(ret.message).toMatch(/participation/i)
      expect((ret as { text?: string }).text).toContain('grow')
    }
  })
})

describe('select in app', () => {
  it('selectInApp sets shared selection when nodeId present', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'warning',
            rule: 'sail-layout',
            message: 'collapsed',
            line: 2,
            col: 1,
            path: 't.sail',
            node_id: 'node:abc',
          },
        ],
      }),
      sailAuthoringLayoutCheck: async () => ({
        viewports: [{}, {}],
      }),
    })
    await loadFindings('t.sail')
    selectInApp(getChecksState().findings[0]!)
    expect(getSelection().nodeId).toBe('node:abc')
    expect(getShellState().checksBadge).toBeGreaterThanOrEqual(0)
  })
})

describe('empty vs error (no client)', () => {
  it('loadFindings without client is neutral empty (not error)', async () => {
    setChecksOpsClient(null)
    const out = await loadFindings('apps/studio/routes/edit/page.sail')
    expect(out).toEqual([])
    expect(getChecksState().error).toBeNull()
    expect(getChecksState().loading).toBe(false)
  })

  it('checks surface has error strip + retry, not only empty-state failure', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/checks-surface/checks-surface.sail'),
      'utf8',
    )
    expect(surface).toMatch(/data-checks-error/)
    expect(surface).toMatch(/data-retry-checks/)
    expect(surface).toMatch(/Retry/)
    // Empty-state only when !hasChecksError
    expect(surface).toMatch(/hasChecksError/)
  })
})

describe('header counts / status', () => {
  it('error badge updates shell checksBadge', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 2,
        diagnostics: [
          {
            severity: 'error',
            rule: 'a',
            message: 'e1',
            line: 1,
            col: 1,
            path: 't.sail',
          },
          {
            severity: 'warning',
            rule: 'b',
            message: 'w1',
            line: 2,
            col: 1,
            path: 't.sail',
          },
        ],
      }),
    })
    await loadFindings('t.sail')
    const s = getChecksState()
    expect(s.errorCount).toBe(1)
    expect(s.warningCount).toBe(1)
    expect(s.statusNote).toMatch(/1 error.*block the build/)
    expect(getShellState().checksBadge).toBe(1)
  })
})

describe('severity groups + expandable suggest', () => {
  it('groupBySeverity orders errors before warnings with file:line rows', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 3,
        diagnostics: [
          {
            severity: 'warning',
            rule: 'sail-layout',
            message: 'warn first',
            line: 4,
            col: 1,
            path: 'apps/demo/page.sail',
          },
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'style.grow — use grow attr\n  suggest: use grow attribute',
            line: 2,
            col: 3,
            path: 'apps/demo/page.sail',
          },
          {
            severity: 'error',
            rule: 'sail-no-raw-container',
            message: 'raw div\n  suggest: use column',
            line: 8,
            col: 1,
            path: 'apps/demo/other.sail',
          },
        ],
      }),
      readSource: async () => '<div sx={style.grow(1)}>x</div>\n',
    })
    await loadFindings('apps/demo/page.sail', {
      source: '<div sx={style.grow(1)}>x</div>\n',
    })
    const groups = groupBySeverity(getChecksState().findings)
    expect(groups.map((g) => g.severity)).toEqual(['error', 'warning'])
    expect(groups[0]!.count).toBe(2)
    expect(groups[0]!.items.every((f) => f.fileSpan.includes(':'))).toBe(true)
    expect(groups[0]!.items.some((f) => f.suggest?.includes('grow'))).toBe(true)
    const ui = uiSeverityGroups()
    expect(ui[0]!.label).toBe('Errors')
    expect(uiFixableCount()).toBeGreaterThan(0)
  })
})

describe('Review fixes dialog flow', () => {
  it('openFixReview builds accept/skip diffs; applyFixReview rewrites source + edit buffer', async () => {
    const src = '<div sx={style.grow(1)}>x</div>'
    const path = 't.sail'
    let written: { path: string; text: string } | null = null
    seedSourceText(path, src)
    setEditOpsClient({
      writeSource: async (p, content) => {
        written = { path: p, text: content }
      },
    } as OpsClient)
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow\n  suggest: use grow attr',
            line: 1,
            col: 1,
            path,
          },
        ],
      }),
      readSource: async () => src,
    })
    await loadFindings(path, { source: src })
    openFixReview()
    let s = getChecksState()
    expect(s.reviewOpen).toBe(true)
    expect(s.reviewItems.length).toBe(1)
    expect(s.reviewItems[0]!.machine).toBe(true)
    expect(s.reviewItems[0]!.accepted).toBe(true)
    expect(s.reviewItems[0]!.before).toMatch(/style\.grow|grow/)
    expect(s.reviewItems[0]!.after.length).toBeGreaterThan(0)

    setFixReviewAccepted(s.reviewItems[0]!.findingId, false)
    expect(getChecksState().reviewItems[0]!.accepted).toBe(false)
    setFixReviewAccepted(s.reviewItems[0]!.findingId, true)

    const { applied } = applyFixReview()
    expect(applied).toBe(1)
    s = getChecksState()
    expect(s.reviewOpen).toBe(false)
    expect(s.sourceByPath.get(path)).toContain('grow')
    expect(s.sourceByPath.get(path)).not.toContain('style.grow')
    expect(s.findings.length).toBe(0)
    expect(s.reviewNote).toMatch(/Applied 1 fix/)
    // Durable: edit document buffer updated + dirty
    const doc = getEditState().docs.get(path)
    expect(doc?.source.text).toBeDefined()
    expect(doc!.source.text).not.toContain('style.grow')
    expect(doc!.source.dirty).toBe(true)
    // sourceForSave prefers applied text even when autoFixMode is off
    setAutoFixMode('off')
    const saved = sourceForSave(path, src)
    expect(saved.changed).toBe(true)
    expect(saved.text).not.toContain('style.grow')
    // writeSource fired (async fire-and-forget — wait a tick)
    await new Promise((r) => setTimeout(r, 10))
    expect(written?.text).toBeDefined()
    expect(written!.text).not.toContain('style.grow')
  })

  it('skip all → applied 0 and findings retained', async () => {
    const src = '<div sx={style.grow(1)}>x</div>'
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow\n  suggest: use grow attr',
            line: 1,
            col: 1,
            path: 't.sail',
          },
        ],
      }),
    })
    await loadFindings('t.sail', { source: src })
    openFixReview()
    const id = getChecksState().reviewItems[0]!.findingId
    setFixReviewAccepted(id, false)
    const { applied } = applyFixReview()
    expect(applied).toBe(0)
    expect(getChecksState().findings).toHaveLength(1)
    expect(getChecksState().reviewNote).toMatch(/skipped/i)
    expect(getChecksState().sourceByPath.get('t.sail')).toBe(src)
  })

  it('suggest-only findings are excluded from Review fixes', () => {
    const findings = parseCheckFindings(
      [
        {
          severity: 'error',
          rule: 'sail-no-raw-container',
          message: 'raw div\n  suggest: use column',
          line: 1,
          col: 1,
          path: 't.sail',
        },
        {
          severity: 'error',
          rule: 'sail-participation-style',
          message: 'x\n  suggest: y',
          line: 1,
          col: 1,
          path: 't.sail',
        },
      ],
      { defaultPath: 't.sail' },
    )
    const srcMap = new Map([['t.sail', '<div sx={style.grow(1)}>x</div>']])
    const items = buildFixReviewItems(findings, srcMap)
    // only participation-style rewrites source; suggest-only raw-container excluded
    expect(items).toHaveLength(1)
    expect(items[0]!.rule).toBe('sail-participation-style')
    expect(items[0]!.machine).toBe(true)
    expect(items[0]!.accepted).toBe(true)
    expect(items[0]!.fileSpan).toMatch(/t\.sail:1/)
  })

  it('single-row applyFix updates edit buffer and drops finding', async () => {
    const src = '<div sx={style.grow(1)}>x</div>'
    const path = 'row.sail'
    seedSourceText(path, src)
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow\n  suggest: use attr',
            line: 1,
            col: 1,
            path,
          },
        ],
      }),
    })
    await loadFindings(path, { source: src })
    const ret = applyFix(getChecksState().findings[0]!.id)
    expect(ret.ok).toBe(true)
    expect(getChecksState().findings).toHaveLength(0)
    expect(getChecksState().reviewNote).toMatch(/Applied 1 fix/)
    expect(getEditState().docs.get(path)?.source.text).not.toContain('style.grow')
    // Attribution recorded (rule + before/after + timestamp)
    expect(getChecksState().attributions.length).toBeGreaterThan(0)
    expect(getChecksState().attributions[0]!.rule).toBe('sail-participation-style')
    expect(getChecksState().attributions[0]!.before).toMatch(/grow/)
    expect(getChecksState().attributions[0]!.at).toBeGreaterThan(0)
  })
})

describe('STUDIO-3: rule groups + autofix + re-check + empty', () => {
  it('groupByRule sorts by count desc with severity badge counts', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 4,
        diagnostics: [
          {
            severity: 'warning',
            rule: 'sail-layout',
            message: 'collapsed',
            line: 1,
            col: 1,
            path: 'a.sail',
          },
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow a',
            line: 2,
            col: 1,
            path: 'a.sail',
          },
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow b',
            line: 3,
            col: 1,
            path: 'a.sail',
          },
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow c',
            line: 4,
            col: 1,
            path: 'b.sail',
          },
        ],
      }),
    })
    await loadFindings('a.sail')
    const groups = groupByRule(getChecksState().findings)
    expect(groups[0]!.rule).toBe('sail-participation-style')
    expect(groups[0]!.count).toBe(3)
    expect(groups[0]!.errorCount).toBe(3)
    expect(groups[1]!.rule).toBe('sail-layout')
    expect(groups[1]!.warningCount).toBe(1)
    const ui = uiRuleGroups()
    expect(ui[0]!.rule).toBe('sail-participation-style')
    expect(ui.map((g) => g.rule)).toEqual([
      'sail-participation-style',
      'sail-layout',
    ])
  })

  it('why is verbatim compiler message (never paraphrased)', async () => {
    const why =
      'style.grow — use grow attr\n  suggest: use grow attribute on the host'
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: why,
            line: 1,
            col: 1,
            path: 't.sail',
          },
        ],
      }),
    })
    await loadFindings('t.sail', {
      source: '<div sx={style.grow(1)}>x</div>',
    })
    const f = getChecksState().findings[0]!
    expect(f.why).toBe(why)
    expect(f.message).toBe('style.grow — use grow attr')
    expect(f.suggest).toMatch(/grow attribute/)
  })

  it('applyRuleFixes applies all machine fixes for one rule + attributes', async () => {
    const src = '<div sx={style.grow(1)}>x</div>'
    const path = 'multi.sail'
    seedSourceText(path, src)
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 1,
        diagnostics: [
          {
            severity: 'error',
            rule: 'sail-participation-style',
            message: 'grow\n  suggest: use attr',
            line: 1,
            col: 1,
            path,
          },
        ],
      }),
    })
    await loadFindings(path, { source: src })
    const { applied } = applyRuleFixes('sail-participation-style')
    expect(applied).toBe(1)
    expect(getChecksState().findings).toHaveLength(0)
    expect(getChecksState().attributions[0]!.rule).toBe(
      'sail-participation-style',
    )
  })

  it('setAutoFixMode + recheckFindings updates lastRunAt and empty summary', async () => {
    setChecksOpsClient({
      sailAuthoringCheck: async () => ({
        ok: true,
        count: 0,
        diagnostics: [],
      }),
      sailAuthoringLayoutCheck: async () => ({
        ok: true,
        viewports: [{}, {}],
        diagnostics: [],
      }),
    })
    setAutoFixMode('suggest', 'studio checks')
    await loadFindings('clean.sail', { source: '<column></column>\n' })
    const first = getChecksState().lastRunAt
    expect(first).toBeTruthy()
    expect(getChecksState().lastCheckedPath).toBe('clean.sail')
    expect(getChecksState().lastLayoutChecked).toBe(true)
    expect(getChecksState().autoFixMode).toBe('suggest')
    const summary = uiEmptyCheckedSummary()
    expect(summary).toMatch(/clean\.sail|Checked/)
    expect(summary).toMatch(/compiler \+ layout/)
    expect(summary).toMatch(/autofix suggest/)
    await recheckFindings()
    expect(getChecksState().lastRunAt).toBeGreaterThanOrEqual(first!)
    expect(getChecksState().findings).toHaveLength(0)
  })

  it('surface markers: rule groups, sail-diff, two exits, autofix, re-check', () => {
    const surface = readFileSync(
      join(REPO, 'apps/studio/src/checks-surface/checks-surface.sail'),
      'utf8',
    )
    const vm = readFileSync(
      join(REPO, 'apps/studio/src/checks-surface/checks-surface-vm.ts'),
      'utf8',
    )
    expect(surface).toMatch(/data-rule-group/)
    expect(surface).toMatch(/data-findings-table/)
    expect(surface).toMatch(/Open in editor/)
    expect(surface).toMatch(/Select element/)
    expect(surface).toMatch(/data-finding-action="open-source"/)
    expect(surface).toMatch(/data-finding-action="select-in-app"/)
    expect(surface).toMatch(/<sail-diff\b/)
    expect(surface).toMatch(/data-autofix-mode/)
    expect(surface).toMatch(/data-recheck/)
    expect(surface).toMatch(/data-empty-checked/)
    expect(surface).toMatch(/data-apply-rule/)
    expect(surface).toMatch(/Here's what was checked|emptyChecked/)
    expect(vm).toMatch(/createTable/)
    expect(vm).toMatch(/grouping:\s*\[\s*['"]rule['"]\s*\]/)
  })
})
