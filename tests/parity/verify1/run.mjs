#!/usr/bin/env node
/**
 * VERIFY-1 proof program — taffy/takumi layout trust.
 *
 * Work items:
 *   1. Enumerate layout vocabulary (vocab.mjs)
 *   2. Seeded corpus: taffy ↔ Chromium rect-by-rect, per-dimension rates
 *   3. Text measurement proof
 *   4. Error-class precision/recall (100% precision bar for Error)
 *   5. Takumi paint truth on paint corpus
 *
 * Invoke:
 *   SAIL_BIN=target/release/sail node tests/parity/verify1/run.mjs
 *   node tests/parity/verify1/run.mjs --seed 42 --geometry 2000
 *   node tests/parity/verify1/run.mjs --quick   # small corpus for smoke
 *   node tests/parity/verify1/run.mjs --phase vocab|geometry|text|errors|paint|all
 *
 * Artifacts: tests/parity/verify1/out/
 * Verdict is written by the agent to mission scratch from out/summary.json.
 */

import {
  mkdirSync,
  writeFileSync,
  readFileSync,
  existsSync,
  rmSync,
} from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { tmpdir } from 'node:os'
import { VOCAB, vocabSummary } from './vocab.mjs'
import { generateCorpus, mulberry32 } from './generator.mjs'
import {
  ensureOut,
  outDir,
  repoRoot,
  SAIL_BIN,
  CHROME,
  CHROME_SOURCE,
  VIEWPORT,
  PAINT_VIEWPORT,
  TOL,
  layoutCheckAll,
  compileBatch,
  writeMultiCaseHtml,
  measureBrowserCases,
  diffBoxes,
  classifyDiag,
  browserErrorTruth,
  takeRepro,
  writeJson,
  sailRender,
  measureBrowserStyles,
  paintDiff,
  entryHtml,
  entryCss,
} from './shared.mjs'

const __dirname = dirname(fileURLToPath(import.meta.url))

const args = process.argv.slice(2)
function flag(name) {
  return args.includes(name)
}
function opt(name, def) {
  const i = args.indexOf(name)
  return i >= 0 ? args[i + 1] : def
}

const seed = Number(opt('--seed', process.env.VERIFY1_SEED || 42))
const quick = flag('--quick')
const phase = opt('--phase', 'all')
const geometryN = Number(opt('--geometry', quick ? 80 : 2000))
const textN = Number(opt('--text', quick ? 40 : 240))
const errorsPerClass = Number(opt('--errors', quick ? 20 : 80))
const paintN = Number(opt('--paint', quick ? 12 : 120))
const batchSize = Number(opt('--batch', 80))

function log(...a) {
  console.log(...a)
}

async function phaseVocab() {
  log('== WI-1 vocabulary ==')
  const summary = vocabSummary()
  writeJson(join(outDir, 'vocab.json'), { ...VOCAB, summary })
  log(
    `primitives=${summary.primitive_count} dims=${summary.dimension_ids.length} gaps=${summary.taffy_gaps.length}`,
  )
  log(`taffy_gaps: ${summary.taffy_gaps.join(', ')}`)
  return { vocab: summary }
}

async function phaseGeometry(corpus) {
  log(`== WI-2 geometry corpus (${geometryN} cases, seed=${seed}) ==`)
  const cases = corpus.cases.filter((c) => c.family === 'geometry')
  log(`LayoutCheck × ${cases.length} (parallel)…`)
  const t0 = Date.now()
  const taffy = await layoutCheckAll(cases, {
    concurrency: 8,
    onProgress: (n, t) => {
      if (n % 200 === 0 || n === t) process.stdout.write(`\r  taffy ${n}/${t}`)
    },
  })
  process.stdout.write('\n')
  log(`  taffy done in ${((Date.now() - t0) / 1000).toFixed(1)}s`)

  // Compile in batches for lowered HTML + injected sx CSS
  const artifactById = new Map()
  let compileOk = 0
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const stage = join(tmpdir(), `v1-geom-batch-${i}`)
    const res = compileBatch(slice, stage)
    if (!res.ok && res.compiled === 0) {
      log(`  compile batch ${i} fail: ${res.error?.slice(0, 120)}`)
    } else {
      for (const [id, entry] of res.htmlById) {
        artifactById.set(id, entry)
        compileOk++
      }
    }
    process.stdout.write(`\r  compile ${Math.min(i + batchSize, cases.length)}/${cases.length}`)
  }
  process.stdout.write('\n')
  log(`  compiled html+css for ${artifactById.size} cases (ok≈${compileOk})`)

  // Browser measure in batches
  const browserById = new Map()
  let browserSkipped = null
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const pairs = slice
      .map((c) => {
        const e = artifactById.get(c.id)
        return { id: c.id, html: entryHtml(e), css: entryCss(e) }
      })
      .filter((p) => p.html)
    if (!pairs.length) continue
    const serve = join(tmpdir(), `v1-geom-serve-${i}`)
    writeMultiCaseHtml(serve, pairs)
    const measured = await measureBrowserCases(
      serve,
      pairs.map((p) => p.id),
      VIEWPORT,
    )
    if (measured.skipped) {
      browserSkipped = measured.reason
      break
    }
    for (const [id, boxes] of measured.map) browserById.set(id, boxes)
    process.stdout.write(
      `\r  browser ${Math.min(i + batchSize, cases.length)}/${cases.length}`,
    )
  }
  process.stdout.write('\n')
  if (browserSkipped) log(`  browser SKIPPED: ${browserSkipped}`)
  else log(`  browser measured ${browserById.size} cases`)

  // Per-dimension stats + overall + repros
  const dimStats = {} // dim -> { total, size_ok, size_fail, structure_ok }
  function bump(dim, key) {
    if (!dim) return
    if (!dimStats[dim]) dimStats[dim] = { total: 0, size_ok: 0, size_fail: 0, structure_ok: 0 }
    dimStats[dim][key]++
  }

  const repros = []
  let overall = {
    cases: cases.length,
    taffy_ok: 0,
    browser_ok: 0,
    compared_cases: 0,
    size_ok_cases: 0,
    structure_ok_cases: 0,
    size_match_boxes: 0,
    size_fail_boxes: 0,
    compared_boxes: 0,
  }

  const byCase = []
  for (let i = 0; i < cases.length; i++) {
    const c = cases[i]
    const t = taffy[i]
    const row = { id: c.id, dims: c.dims }
    if (!t?.ok) {
      row.taffy_ok = false
      row.error = t?.error
      byCase.push(row)
      continue
    }
    overall.taffy_ok++
    row.taffy_ok = true
    row.taffy_boxes = t.boxes.length
    row.diagnostics = (t.diagnostics || []).length

    const bb = browserById.get(c.id)
    if (!bb) {
      row.browser_ok = false
      byCase.push(row)
      continue
    }
    overall.browser_ok++
    row.browser_ok = true
    const diff = diffBoxes(t.boxes, bb)
    row.diff = {
      compared: diff.compared,
      size_match: diff.size_match,
      size_fail: diff.size_fail,
      structure_ok: diff.structure_ok,
      size_ok: diff.size_ok,
    }
    overall.compared_cases++
    overall.compared_boxes += diff.compared
    overall.size_match_boxes += diff.size_match
    overall.size_fail_boxes += diff.size_fail
    if (diff.structure_ok) overall.structure_ok_cases++
    if (diff.size_ok) overall.size_ok_cases++

    // dimension tagging: every dim key on the case
    const dimKeys = Object.keys(c.dims || {})
    for (const dk of dimKeys) {
      const dimId = dk.startsWith('primitive') ? 'primitive' : dk
      const val = c.dims[dk]
      const key = `${dimId}=${val}`
      if (!dimStats[key]) dimStats[key] = { total: 0, size_ok: 0, size_fail: 0, structure_ok: 0 }
      dimStats[key].total++
      if (diff.size_ok) dimStats[key].size_ok++
      else dimStats[key].size_fail++
      if (diff.structure_ok) dimStats[key].structure_ok++
    }
    // also roll up by dimension family
    for (const dk of dimKeys) {
      const dimId = dk.startsWith('primitive') ? 'primitive' : dk
      if (!dimStats[dimId]) dimStats[dimId] = { total: 0, size_ok: 0, size_fail: 0, structure_ok: 0 }
      dimStats[dimId].total++
      if (diff.size_ok) dimStats[dimId].size_ok++
      else dimStats[dimId].size_fail++
      if (diff.structure_ok) dimStats[dimId].structure_ok++
    }

    if (!diff.size_ok && repros.length < 60) {
      const classification = classifyGeomDivergence(diff, c)
      repros.push(
        takeRepro(c, {
          classification,
          diff: row.diff,
          sample_diffs: diff.diffs.slice(0, 4),
        }),
      )
    }
    byCase.push(row)
  }

  // rates
  const dimRates = {}
  for (const [k, v] of Object.entries(dimStats)) {
    dimRates[k] = {
      ...v,
      size_match_rate: v.total ? v.size_ok / v.total : null,
      structure_match_rate: v.total ? v.structure_ok / v.total : null,
    }
  }

  const result = {
    seed,
    viewport: VIEWPORT,
    tol: TOL,
    overall: {
      ...overall,
      case_size_match_rate: overall.compared_cases
        ? overall.size_ok_cases / overall.compared_cases
        : null,
      case_structure_match_rate: overall.compared_cases
        ? overall.structure_ok_cases / overall.compared_cases
        : null,
      box_size_match_rate: overall.compared_boxes
        ? overall.size_match_boxes / overall.compared_boxes
        : null,
      browser_skipped: browserSkipped,
    },
    per_dimension: dimRates,
    repros,
    sample_cases: byCase.slice(0, 20),
  }
  writeJson(join(outDir, 'geometry.json'), result)
  writeJson(join(outDir, 'repros', 'geometry-repros.json'), repros)
  log(
    `  size match cases: ${overall.size_ok_cases}/${overall.compared_cases} (${pct(result.overall.case_size_match_rate)})`,
  )
  log(
    `  box size match: ${overall.size_match_boxes}/${overall.compared_boxes} (${pct(result.overall.box_size_match_rate)})`,
  )
  log(`  repros: ${repros.length}`)
  return result
}

function classifyGeomDivergence(diff, c) {
  // Heuristic classification for verdict
  const hasText = (c.dims?.text_script && c.dims.text_script !== 'latin') || false
  const hasTextAny = !!c.dims?.text_script
  const maxD = Math.max(
    0,
    ...diff.diffs.filter((d) => d.dw != null).map((d) => Math.max(d.dw || 0, d.dh || 0)),
  )
  if (diff.structure_ok === false && diff.tag_diverge > 0) return 'structure/tag-pairing'
  if (hasText && maxD < 20) return 'text-delta'
  if (c.dims?.overflow) return 'overflow/scroll-modeling'
  if (c.dims?.primitive === 'grid' || c.dims?.grid_cols) return 'grid-track-modeling'
  if (maxD > 50) return 'large-delta-investigate'
  if (hasTextAny) return 'text-delta-or-mapping'
  return 'sail-mapping-or-taffy-edge'
}

async function phaseText(corpus) {
  log(`== WI-3 text measurement (${textN} cases) ==`)
  const cases = corpus.cases.filter((c) => c.family === 'text')
  const taffy = await layoutCheckAll(cases, { concurrency: 8 })

  const artifactById = new Map()
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const stage = join(tmpdir(), `v1-text-batch-${i}`)
    const res = compileBatch(slice, stage)
    for (const [id, entry] of res.htmlById) artifactById.set(id, entry)
  }

  const browserById = new Map()
  let browserSkipped = null
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const pairs = slice
      .map((c) => {
        const e = artifactById.get(c.id)
        return { id: c.id, html: entryHtml(e), css: entryCss(e) }
      })
      .filter((p) => p.html)
    if (!pairs.length) continue
    const serve = join(tmpdir(), `v1-text-serve-${i}`)
    writeMultiCaseHtml(serve, pairs)
    const measured = await measureBrowserCases(
      serve,
      pairs.map((p) => p.id),
      VIEWPORT,
    )
    if (measured.skipped) {
      browserSkipped = measured.reason
      break
    }
    for (const [id, boxes] of measured.map) browserById.set(id, boxes)
  }

  const metrics = {
    intrinsic_height: { match: 0, fail: 0, total: 0, deltas: [] },
    intrinsic_width: { match: 0, fail: 0, total: 0, deltas: [] },
    by_script: {},
    by_family: {},
    by_size: {},
  }
  const repros = []
  const textTol = Math.max(TOL, 3) // text heights need slightly looser tol

  for (let i = 0; i < cases.length; i++) {
    const c = cases[i]
    const t = taffy[i]
    if (!t?.ok) continue
    const bb = browserById.get(c.id)
    if (!bb?.length || !t.boxes?.length) continue

    // Compare leaf span-ish: last non-empty box often the text host
    const tLeaf =
      [...t.boxes].reverse().find((b) => (b.height || 0) > 0) || t.boxes[t.boxes.length - 1]
    const bLeaf =
      [...bb].reverse().find((b) => (b.height || 0) > 0 && b.tag !== 'section') ||
      bb[bb.length - 1]
    if (!tLeaf || !bLeaf) continue

    const dh = Math.abs((tLeaf.height || 0) - (bLeaf.height || 0))
    const dw = Math.abs((tLeaf.width || 0) - (bLeaf.width || 0))
    metrics.intrinsic_height.total++
    metrics.intrinsic_width.total++
    if (dh <= textTol) metrics.intrinsic_height.match++
    else {
      metrics.intrinsic_height.fail++
      metrics.intrinsic_height.deltas.push(dh)
    }
    if (dw <= textTol) metrics.intrinsic_width.match++
    else {
      metrics.intrinsic_width.fail++
      metrics.intrinsic_width.deltas.push(dw)
    }

    const script = c.dims?.text_script || 'unknown'
    const fam = c.dims?.text_family || 'unknown'
    const size = c.dims?.text_size || 'unknown'
    for (const [bucket, key] of [
      [metrics.by_script, script],
      [metrics.by_family, fam],
      [metrics.by_size, size],
    ]) {
      if (!bucket[key]) bucket[key] = { match: 0, total: 0 }
      bucket[key].total++
      if (dh <= textTol && dw <= textTol) bucket[key].match++
    }

    if ((dh > textTol || dw > textTol) && repros.length < 40) {
      const verdict =
        script === 'cjk' || script === 'emoji' || script === 'mixed'
          ? 'font-config-or-shaping-delta'
          : dh > 20
            ? 'unexplained-or-lineheight'
            : 'shaping-delta'
      repros.push(
        takeRepro(c, {
          taffy: { w: tLeaf.width, h: tLeaf.height },
          browser: { w: bLeaf.width, h: bLeaf.height },
          dh,
          dw,
          verdict,
        }),
      )
    }
  }

  function rate(m) {
    return m.total ? m.match / m.total : null
  }

  const result = {
    seed,
    textTol,
    browser_skipped: browserSkipped,
    metrics: {
      intrinsic_height: {
        ...metrics.intrinsic_height,
        match_rate: rate(metrics.intrinsic_height),
        mean_fail_delta: mean(metrics.intrinsic_height.deltas),
      },
      intrinsic_width: {
        ...metrics.intrinsic_width,
        match_rate: rate(metrics.intrinsic_width),
        mean_fail_delta: mean(metrics.intrinsic_width.deltas),
      },
      by_script: Object.fromEntries(
        Object.entries(metrics.by_script).map(([k, v]) => [
          k,
          { ...v, match_rate: v.total ? v.match / v.total : null },
        ]),
      ),
      by_family: Object.fromEntries(
        Object.entries(metrics.by_family).map(([k, v]) => [
          k,
          { ...v, match_rate: v.total ? v.match / v.total : null },
        ]),
      ),
      by_size: Object.fromEntries(
        Object.entries(metrics.by_size).map(([k, v]) => [
          k,
          { ...v, match_rate: v.total ? v.match / v.total : null },
        ]),
      ),
    },
    repros,
    notes: [
      'sail-layout uses baked ASCII metrics table (latin primary); non-ASCII → fallback_width_px',
      'No kerning/ligatures/BiDi in measure path',
      'Same woff2 assets loaded in browser via tokens CSS when available',
    ],
  }
  writeJson(join(outDir, 'text.json'), result)
  log(
    `  height match: ${metrics.intrinsic_height.match}/${metrics.intrinsic_height.total} (${pct(rate(metrics.intrinsic_height))})`,
  )
  log(
    `  width match: ${metrics.intrinsic_width.match}/${metrics.intrinsic_width.total} (${pct(rate(metrics.intrinsic_width))})`,
  )
  return result
}

function mean(arr) {
  if (!arr?.length) return null
  return arr.reduce((a, b) => a + b, 0) / arr.length
}

async function phaseErrors(corpus) {
  log(`== WI-4 error-class precision/recall ==`)
  const cases = corpus.cases.filter((c) => c.family === 'error')
  const taffy = await layoutCheckAll(cases, { concurrency: 8 })

  // Browser truth: compile when possible; Error-severity positives ship hand-lowered HTML
  // because `sail compile` fails the build on layout Errors (by design).
  const artifactById = new Map()
  for (const c of cases) {
    if (c.html) {
      // Minimal sx CSS for hand-lowered silent-clip cases
      artifactById.set(c.id, {
        html: c.html,
        css: `@layer sail {
.sx-h0{height:0}.sx-h8{height:var(--space-8)}.sx-h12{height:var(--space-12)}.sx-h16{height:var(--space-16)}.sx-h20{height:var(--space-20)}.sx-h24{height:var(--space-24)}
.sx-wfull{width:100%}.sx-overflowhidden{overflow:hidden}.sx-shrink0{flex-shrink:0}
.sail-col{display:flex;flex-direction:column;min-width:0}.sail-ai-stretch{align-items:stretch}
}`,
      })
    }
  }
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize).filter((c) => !artifactById.has(c.id))
    if (!slice.length) continue
    const stage = join(tmpdir(), `v1-err-batch-${i}`)
    const res = compileBatch(slice, stage)
    for (const [id, entry] of res.htmlById) artifactById.set(id, entry)
  }
  const browserById = new Map()
  let browserSkipped = null
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const pairs = slice
      .map((c) => {
        const e = artifactById.get(c.id)
        return { id: c.id, html: entryHtml(e), css: entryCss(e) }
      })
      .filter((p) => p.html)
    if (!pairs.length) continue
    const serve = join(tmpdir(), `v1-err-serve-${i}`)
    writeMultiCaseHtml(serve, pairs)
    const measured = await measureBrowserCases(
      serve,
      pairs.map((p) => p.id),
      VIEWPORT,
    )
    if (measured.skipped) {
      browserSkipped = measured.reason
      break
    }
    for (const [id, boxes] of measured.map) browserById.set(id, boxes)
  }

  const classes = ['silent-clip', 'child-overflows-parent', 'height-chain']
  const stats = Object.fromEntries(
    classes.map((id) => [
      id,
      {
        // vs labeled expected (generator ground truth for what SHOULD fire)
        tp: 0,
        fp: 0,
        tn: 0,
        fn: 0,
        // vs browser heuristic truth
        browser_tp: 0,
        browser_fp: 0,
        browser_tn: 0,
        browser_fn: 0,
        fp_repros: [],
        fn_repros: [],
      },
    ]),
  )

  for (let i = 0; i < cases.length; i++) {
    const c = cases[i]
    const t = taffy[i]
    const classId = c.class_id
    if (!stats[classId]) continue
    const diags = t?.ok ? t.diagnostics || [] : []
    const fired = diags.some((d) => classifyDiag(d) === classId)
    // Severity of fired diags for this class
    const errorSeverity = diags.some((d) => {
      if (classifyDiag(d) !== classId) return false
      const sev = (d.severity || '').toLowerCase()
      return sev === 'error' || sev === 'Error'
    })

    const expected = !!c.expected
    if (fired && expected) stats[classId].tp++
    else if (fired && !expected) {
      stats[classId].fp++
      if (stats[classId].fp_repros.length < 15) {
        stats[classId].fp_repros.push(
          takeRepro(c, { fired, expected, diags: diags.map((d) => d.message?.slice(0, 80)) }),
        )
      }
    } else if (!fired && !expected) stats[classId].tn++
    else {
      stats[classId].fn++
      if (stats[classId].fn_repros.length < 15) {
        stats[classId].fn_repros.push(takeRepro(c, { fired, expected }))
      }
    }

    // Browser truth comparison
    const bb = browserById.get(c.id)
    if (bb) {
      const truth = browserErrorTruth(bb)
      const bExpected = !!truth[classId]
      if (fired && bExpected) stats[classId].browser_tp++
      else if (fired && !bExpected) stats[classId].browser_fp++
      else if (!fired && !bExpected) stats[classId].browser_tn++
      else stats[classId].browser_fn++
    }

    // track default severity for ruling
    stats[classId]._saw_error_sev = stats[classId]._saw_error_sev || errorSeverity
  }

  const rulings = {}
  for (const id of classes) {
    const s = stats[id]
    const precision = s.tp + s.fp ? s.tp / (s.tp + s.fp) : null
    const recall = s.tp + s.fn ? s.tp / (s.tp + s.fn) : null
    const bPrec =
      s.browser_tp + s.browser_fp ? s.browser_tp / (s.browser_tp + s.browser_fp) : null
    const bRec =
      s.browser_tp + s.browser_fn ? s.browser_tp / (s.browser_tp + s.browser_fn) : null

    // 100% precision bar for Error severity
    let ruling
    const meta = VOCAB.error_classes.find((e) => e.id === id)
    const defaultSev = meta?.default_severity || 'Warning'
    if (defaultSev === 'Error') {
      if (precision === 1 && s.fp === 0 && s.tp > 0) {
        ruling = 'Error — keep (100% precision on labeled corpus)'
      } else if (precision != null && precision < 1) {
        ruling = 'DROP or demote — false positives on labeled corpus (fails 100% precision bar)'
      } else if (s.tp === 0) {
        ruling = 'INCONCLUSIVE — no true positives fired; investigate detector'
      } else {
        ruling = 'KEEP as Error only if precision stays 1.0; else warning'
      }
    } else {
      if (precision === 1 && s.tp > 0) {
        ruling =
          'Eligible for Error promotion under layout:error profile ONLY if browser precision also 1.0; default stays Warning'
      } else if (precision != null && precision < 1) {
        ruling = 'Stay Warning or drop — precision below Error bar'
      } else {
        ruling = 'Stay Warning — insufficient evidence for Error'
      }
    }

    rulings[id] = {
      labeled: {
        tp: s.tp,
        fp: s.fp,
        tn: s.tn,
        fn: s.fn,
        precision,
        recall,
      },
      browser_heuristic: {
        tp: s.browser_tp,
        fp: s.browser_fp,
        tn: s.browser_tn,
        fn: s.browser_fn,
        precision: bPrec,
        recall: bRec,
      },
      default_severity: defaultSev,
      ruling,
      fp_repros: s.fp_repros,
      fn_repros: s.fn_repros,
    }
  }

  const result = {
    seed,
    browser_skipped: browserSkipped,
    cases: cases.length,
    classes: rulings,
    bar: '100% precision required for Error severity',
  }
  writeJson(join(outDir, 'errors.json'), result)
  for (const id of classes) {
    const r = rulings[id].labeled
    log(
      `  ${id}: P=${pct(r.precision)} R=${pct(r.recall)} (tp=${r.tp} fp=${r.fp} fn=${r.fn}) → ${rulings[id].ruling.slice(0, 60)}`,
    )
  }
  return result
}

async function phasePaint(corpus) {
  log(`== WI-5 takumi paint (${paintN} cases) ==`)
  const cases = corpus.cases.filter((c) => c.family === 'paint')
  const paintDir = join(outDir, 'paint-corpus')
  rmSync(paintDir, { recursive: true, force: true })
  mkdirSync(paintDir, { recursive: true })

  // 1) takumi render all
  const regionsById = new Map()
  let casesTotal = 0
  for (const c of cases) {
    const path = join(paintDir, `${c.id}.sail`)
    writeFileSync(path, c.source)
    casesTotal++
    const rend = sailRender(path, PAINT_VIEWPORT)
    if (rend.ok) regionsById.set(c.id, rend.regions)
    process.stdout.write(`\r  takumi ${casesTotal}/${cases.length}`)
  }
  process.stdout.write('\n')
  log(`  takumi rendered ${regionsById.size}/${cases.length}`)

  // 2) compile all + one multi-case browser measure (single Chrome launch)
  const pairs = []
  for (let i = 0; i < cases.length; i += batchSize) {
    const slice = cases.slice(i, i + batchSize)
    const stage = join(tmpdir(), `v1-paint-batch-${i}`)
    const comp = compileBatch(slice, stage)
    for (const c of slice) {
      const e = comp.htmlById.get(c.id)
      const html = entryHtml(e)
      if (html) pairs.push({ id: c.id, html, css: entryCss(e) })
    }
  }
  const serve = join(tmpdir(), `v1-paint-serve-all`)
  writeMultiCaseHtml(serve, pairs, { width: PAINT_VIEWPORT.width })
  // One Chrome launch: L1 host computed styles per case
  let styleMap = new Map()
  const one = await measureBrowserStylesMulti(serve, pairs.map((p) => p.id), PAINT_VIEWPORT)
  if (!one.skipped) styleMap = one.map
  else log(`  style measure skip: ${one.reason}`)

  let propsMatch = 0
  let propsCompared = 0
  let casesOk = 0
  const repros = []
  const byVariant = {}

  for (const c of cases) {
    const regions = regionsById.get(c.id)
    const hosts = styleMap.get(c.id) || []
    if (!regions || !hosts.length) continue
    const diff = paintDiff(regions, hosts)
    propsMatch += diff.props_match
    propsCompared += diff.props_compared
    if (diff.props_compared > 0 && diff.props_fail === 0) casesOk++
    const variant = c.dims?.paint_variant || 'unknown'
    if (!byVariant[variant]) byVariant[variant] = { match: 0, total: 0, props_m: 0, props_c: 0 }
    byVariant[variant].total++
    byVariant[variant].props_m += diff.props_match
    byVariant[variant].props_c += diff.props_compared
    if (diff.props_fail === 0 && diff.props_compared > 0) byVariant[variant].match++
    if (diff.props_fail > 0 && repros.length < 30) {
      repros.push(takeRepro(c, { paint: diff }))
    }
  }

  const result = {
    seed,
    cases: casesTotal,
    cases_style_ok: casesOk,
    props_match: propsMatch,
    props_compared: propsCompared,
    prop_match_rate: propsCompared ? propsMatch / propsCompared : null,
    case_match_rate: casesTotal ? casesOk / casesTotal : null,
    by_variant: byVariant,
    repros,
    browser_skipped: one.skipped ? one.reason : null,
  }
  writeJson(join(outDir, 'paint.json'), result)
  log(
    `  props: ${propsMatch}/${propsCompared} (${pct(result.prop_match_rate)}); cases ${casesOk}/${casesTotal}`,
  )
  return result
}

/** One Chrome launch: per-case L1 host computed styles (oklch→rgba). */
async function measureBrowserStylesMulti(serveDir, caseIds, viewport = PAINT_VIEWPORT) {
  if (!CHROME) return { skipped: true, reason: 'no chrome', map: new Map() }
  let chromium
  try {
    const mod = await import(
      '/home/madvib/dev/Ship/worktrees/d-lang-g6/node_modules/playwright/index.js'
    )
    chromium = mod.chromium
  } catch (e) {
    return { skipped: true, reason: e.message, map: new Map() }
  }
  const { createServer } = await import('node:http')
  const { createReadStream, existsSync, statSync } = await import('node:fs')
  const { join } = await import('node:path')
  const net = await import('node:net')
  const port = await new Promise((resolveP, reject) => {
    const s = net.createServer()
    s.listen(0, '127.0.0.1', () => {
      const { port: p } = s.address()
      s.close(() => resolveP(p))
    })
    s.on('error', reject)
  })
  const server = await new Promise((resolveP, reject) => {
    const srv = createServer((req, res) => {
      let urlPath = decodeURIComponent((req.url || '/').split('?')[0])
      if (urlPath === '/') urlPath = '/index.html'
      const file = join(serveDir, urlPath.replace(/^\//, ''))
      if (!file.startsWith(serveDir) || !existsSync(file) || statSync(file).isDirectory()) {
        res.writeHead(404)
        res.end('not found')
        return
      }
      const ext = file.split('.').pop()
      const types = {
        html: 'text/html',
        css: 'text/css',
        js: 'application/javascript',
        woff2: 'font/woff2',
      }
      res.writeHead(200, { 'Content-Type': types[ext] || 'application/octet-stream' })
      createReadStream(file).pipe(res)
    })
    srv.listen(port, '127.0.0.1', () => resolveP(srv))
    srv.on('error', reject)
  })
  const map = new Map()
  try {
    const browser = await chromium.launch({
      headless: true,
      executablePath: CHROME,
      args: ['--disable-dev-shm-usage', '--no-sandbox'],
      timeout: 60000,
    })
    try {
      const page = await browser.newPage({
        viewport: { width: viewport.width, height: viewport.height },
      })
      await page.goto(`http://127.0.0.1:${port}/`, {
        waitUntil: 'domcontentloaded',
        timeout: 60000,
      })
      await page.waitForTimeout(200)
      const measured = await page.evaluate((ids) => {
        const L1 = new Set([
          'button',
          'card',
          'field',
          'input',
          'switch',
          'badge',
          'navbar',
          'sidebar',
          'dialog',
          'table',
        ])
        const toRgba = (c) => {
          if (!c || c === 'transparent') return 'rgba(0, 0, 0, 0)'
          try {
            const canvas = document.createElement('canvas')
            canvas.width = 1
            canvas.height = 1
            const ctx = canvas.getContext('2d', { willReadFrequently: true })
            ctx.clearRect(0, 0, 1, 1)
            ctx.fillStyle = c
            ctx.fillRect(0, 0, 1, 1)
            const [r, g, b, a] = ctx.getImageData(0, 0, 1, 1).data
            return `rgba(${r}, ${g}, ${b}, ${(a / 255).toFixed(3)})`
          } catch {
            return c
          }
        }
        const out = {}
        for (const id of ids) {
          const sec = document.querySelector(`.v1-case[data-case="${id}"]`)
          if (!sec) {
            out[id] = []
            continue
          }
          const hosts = []
          const walk = (el) => {
            if (el.nodeType !== 1) return
            const tag = el.tagName.toLowerCase()
            if (L1.has(tag) || el.hasAttribute('data-sail-host')) {
              const cs = getComputedStyle(el)
              const r = el.getBoundingClientRect()
              hosts.push({
                tag,
                x: r.x,
                y: r.y,
                w: r.width,
                h: r.height,
                backgroundColor: toRgba(cs.backgroundColor),
                color: toRgba(cs.color),
                borderRadius: cs.borderRadius,
                borderTopWidth: cs.borderTopWidth,
                borderTopColor: toRgba(cs.borderTopColor),
                opacity: cs.opacity,
              })
            }
            for (const c of el.children) walk(c)
          }
          walk(sec)
          out[id] = hosts
        }
        return out
      }, caseIds)
      for (const id of caseIds) map.set(id, measured[id] || [])
    } finally {
      await browser.close().catch(() => {})
    }
  } catch (e) {
    return { skipped: true, reason: String(e.message || e), map }
  } finally {
    await new Promise((r) => server.close(() => r()))
  }
  return { skipped: false, map }
}

function pct(x) {
  if (x == null || Number.isNaN(x)) return 'n/a'
  return `${(x * 100).toFixed(1)}%`
}

function bottomLine(geom, text, errors, paint, vocab) {
  const gaps = vocab?.taffy_gaps || VOCAB.taffy_gaps.map((g) => g.id)
  const sizeRate = geom?.overall?.case_size_match_rate
  const boxRate = geom?.overall?.box_size_match_rate
  const textH = text?.metrics?.intrinsic_height?.match_rate
  const textW = text?.metrics?.intrinsic_width?.match_rate
  const sc = errors?.classes?.['silent-clip']?.labeled
  const paintRate = paint?.prop_match_rate

  const constraints = []
  if (gaps.includes('text-shaping') || (textH != null && textH < 0.95)) {
    constraints.push('text: structure-only or browser-measured caching for non-latin / wrap-critical UI')
  }
  if (gaps.includes('flex-basis')) constraints.push('no flex-basis in closed vocab')
  if (gaps.includes('order')) constraints.push('no CSS order mapping')
  if (gaps.includes('position-fixed-sticky')) constraints.push('fixed/sticky not modeled as browser')
  if (gaps.includes('grid-var-tracks')) constraints.push('custom grid track vars auto-place only')
  if (sizeRate != null && sizeRate < 0.85) {
    constraints.push(`geometry case size match only ${pct(sizeRate)} — not byte-faithful`)
  }
  if (sc && sc.precision != null && sc.precision < 1) {
    constraints.push('silent-clip Error detector has false positives — demote or tighten')
  }

  let positioning
  if (
    sizeRate != null &&
    sizeRate >= 0.98 &&
    boxRate != null &&
    boxRate >= 0.98 &&
    textH != null &&
    textH >= 0.95 &&
    sc?.precision === 1
  ) {
    positioning = 'stands-as-is'
  } else if (sizeRate != null && sizeRate >= 0.7) {
    positioning = 'stands-with-constraints'
  } else if (sizeRate == null) {
    positioning = 'incomplete-run'
  } else {
    positioning = 'reconsider'
  }

  return {
    positioning,
    constraints,
    headline_numbers: {
      geometry_case_size_match_rate: sizeRate,
      geometry_box_size_match_rate: boxRate,
      text_height_match_rate: textH,
      text_width_match_rate: textW,
      silent_clip_precision: sc?.precision ?? null,
      silent_clip_recall: sc?.recall ?? null,
      paint_prop_match_rate: paintRate,
      taffy_gap_count: gaps.length,
    },
  }
}

async function main() {
  if (!existsSync(SAIL_BIN)) {
    console.error(`error: missing sail binary at ${SAIL_BIN}`)
    process.exit(1)
  }
  ensureOut()
  log(`VERIFY-1 seed=${seed} sail=${SAIL_BIN}`)
  log(`chrome=${CHROME || 'null'} (${CHROME_SOURCE}) tol=${TOL}px`)
  log(
    `sizes: geometry=${geometryN} text=${textN} errors/class=${errorsPerClass} paint=${paintN}${quick ? ' (quick)' : ''}`,
  )

  const corpus = generateCorpus(seed, {
    geometry: geometryN,
    text: textN,
    errorsPerClass,
    paint: paintN,
  })
  writeJson(join(outDir, 'corpus-meta.json'), {
    seed,
    sizes: corpus.sizes,
    case_count: corpus.cases.length,
    families: {
      geometry: corpus.cases.filter((c) => c.family === 'geometry').length,
      text: corpus.cases.filter((c) => c.family === 'text').length,
      error: corpus.cases.filter((c) => c.family === 'error').length,
      paint: corpus.cases.filter((c) => c.family === 'paint').length,
    },
  })
  // Persist a sample of sources for repro (not full thousands in git)
  const sampleDir = join(outDir, 'corpus-sample')
  mkdirSync(sampleDir, { recursive: true })
  for (const c of corpus.cases.slice(0, 30)) {
    writeFileSync(join(sampleDir, `${c.id}.sail`), c.source)
  }

  let vocabR, geomR, textR, errR, paintR
  if (phase === 'all' || phase === 'vocab') vocabR = await phaseVocab()
  if (phase === 'all' || phase === 'geometry') geomR = await phaseGeometry(corpus)
  if (phase === 'all' || phase === 'text') textR = await phaseText(corpus)
  if (phase === 'all' || phase === 'errors') errR = await phaseErrors(corpus)
  if (phase === 'all' || phase === 'paint') paintR = await phasePaint(corpus)

  // Load from disk if phase-partial
  if (!vocabR && existsSync(join(outDir, 'vocab.json'))) {
    vocabR = { vocab: vocabSummary() }
  }
  if (!geomR && existsSync(join(outDir, 'geometry.json'))) {
    geomR = JSON.parse(readFileSync(join(outDir, 'geometry.json'), 'utf8'))
  }
  if (!textR && existsSync(join(outDir, 'text.json'))) {
    textR = JSON.parse(readFileSync(join(outDir, 'text.json'), 'utf8'))
  }
  if (!errR && existsSync(join(outDir, 'errors.json'))) {
    errR = JSON.parse(readFileSync(join(outDir, 'errors.json'), 'utf8'))
  }
  if (!paintR && existsSync(join(outDir, 'paint.json'))) {
    paintR = JSON.parse(readFileSync(join(outDir, 'paint.json'), 'utf8'))
  }

  const bl = bottomLine(geomR, textR, errR, paintR, vocabR?.vocab)
  const summary = {
    generated_at: new Date().toISOString(),
    seed,
    quick,
    phase,
    sail: SAIL_BIN,
    chrome: { path: CHROME, source: CHROME_SOURCE },
    tol: TOL,
    vocab: vocabR?.vocab || vocabSummary(),
    geometry: geomR?.overall || null,
    geometry_per_dimension_top: geomR?.per_dimension
      ? Object.fromEntries(
          Object.entries(geomR.per_dimension)
            .filter(([k]) => !k.includes('='))
            .slice(0, 40),
        )
      : null,
    text: textR?.metrics || null,
    errors: errR?.classes
      ? Object.fromEntries(
          Object.entries(errR.classes).map(([k, v]) => [
            k,
            { labeled: v.labeled, ruling: v.ruling, default_severity: v.default_severity },
          ]),
        )
      : null,
    paint: paintR
      ? {
          prop_match_rate: paintR.prop_match_rate,
          case_match_rate: paintR.case_match_rate,
          props_match: paintR.props_match,
          props_compared: paintR.props_compared,
        }
      : null,
    bottom_line: bl,
    artifact_dir: outDir,
  }
  writeJson(join(outDir, 'summary.json'), summary)
  log('')
  log('== BOTTOM LINE ==')
  log(`  positioning: ${bl.positioning}`)
  log(`  numbers: ${JSON.stringify(bl.headline_numbers)}`)
  log(`  constraints (${bl.constraints.length}):`)
  for (const c of bl.constraints) log(`    - ${c}`)
  log(`summary → ${join(outDir, 'summary.json')}`)
}

main().catch((e) => {
  console.error(e)
  process.exit(1)
})
