/**
 * Generated indexes over filtered views. Reference run artifacts — never copy.
 */

import { existsSync, mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { RunStore, type RunRow } from '../store/db.ts'
import { reportsDir, storePath } from '../naming/paths.ts'

export interface ReportResult {
  batchId: string
  reportDir: string
  indexPath: string
  runCount: number
  /** Always 0 — we never copy artifact bytes. */
  copiedBytes: number
}

/**
 * Regenerate batch index HTML + SUMMARY.md.
 * Screenshots and sources are referenced via relative hrefs into the run tree.
 */
export function writeBatchReport(lab: string, batchRef: string): ReportResult {
  const store = RunStore.openExisting(storePath(lab))
  const batchId = store.resolveBatchId(batchRef)
  const batch = store.getBatch(batchId)
  if (!batch) throw new Error(`batch not found: ${batchId}`)
  const runs = store.runsForBatch(batchId)
  const reportDir = batch.report_dir ?? reportsDir(lab, batchId)
  mkdirSync(reportDir, { recursive: true })

  const summary = buildSummaryMd(batchId, batch, runs)
  writeFileSync(join(reportDir, 'SUMMARY.md'), summary)

  const index = buildIndexHtml(batchId, batch, runs, lab)
  const indexPath = join(reportDir, 'index.html')
  writeFileSync(indexPath, index)

  // Pointers only — reaffirm zero-copy
  writeFileSync(
    join(reportDir, 'run_dirs.txt'),
    runs.map((r) => r.run_dir).join('\n') + (runs.length ? '\n' : ''),
  )

  store.close()
  return {
    batchId,
    reportDir,
    indexPath,
    runCount: runs.length,
    copiedBytes: 0,
  }
}

function buildSummaryMd(
  batchId: string,
  batch: { name: string; status: string; profile_hash: string; harness_ver: string },
  runs: RunRow[],
): string {
  const lines = [
    `# Batch ${batchId}`,
    '',
    `- name: ${batch.name}`,
    `- status: ${batch.status}`,
    `- profile_hash: ${batch.profile_hash}`,
    `- harness_ver: ${batch.harness_ver}`,
    `- runs: ${runs.length}`,
    '',
    '| run_id | task | arm | model | gate | render | wall_ms | status | flags |',
    '|---|---|---|---|---:|---:|---:|---|---|',
  ]
  for (const r of runs) {
    const flags = safeJsonArray(r.flags_json).join(',') || '—'
    lines.push(
      `| ${r.run_id} | ${r.task_id} | ${r.arm} | ${r.model_slug} | ${fmtNull(r.gate_ok)} | ${fmtNull(r.render_ok)} | ${fmtNull(r.wall_ms)} | ${r.status} | ${flags} |`,
    )
  }
  lines.push('', '## Paths (reference only — no copies)', '')
  for (const r of runs) {
    lines.push(`- \`${r.run_id}\`: \`${r.run_dir}\``)
  }
  lines.push('')
  return lines.join('\n')
}

function buildIndexHtml(
  batchId: string,
  batch: { name: string; status: string },
  runs: RunRow[],
  lab: string,
): string {
  const cards = runs
    .map((r) => {
      const renderPath = join(r.run_dir, 'render.png')
      const hasRender = existsSync(renderPath)
      const relFromLab = r.run_dir.startsWith(lab)
        ? r.run_dir.slice(lab.length).replace(/^\//, '')
        : r.run_dir
      const img = hasRender
        ? `<img src="/${relFromLab}/render.png" alt="render ${r.run_id}" loading="lazy" />`
        : `<div class="missing">render missing${reasonSuffix(r)}</div>`
      return `<article class="run" data-run-id="${esc(r.run_id)}">
  <h2>${esc(r.run_id)} <small>${esc(r.task_id)} / ${esc(r.arm)} / ${esc(r.model_slug)}</small></h2>
  <div class="shot">${img}</div>
  <dl>
    <dt>status</dt><dd>${esc(r.status)}</dd>
    <dt>gate_ok</dt><dd>${fmtNull(r.gate_ok)}</dd>
    <dt>render_ok</dt><dd>${fmtNull(r.render_ok)}</dd>
    <dt>wall_ms</dt><dd>${fmtNull(r.wall_ms)}</dd>
    <dt>flags</dt><dd>${esc(safeJsonArray(r.flags_json).join(', ') || '—')}</dd>
    <dt>run_dir</dt><dd><code>${esc(r.run_dir)}</code></dd>
    <dt>source</dt><dd><a href="/${esc(relFromLab)}/sandbox/">sandbox/</a></dd>
    <dt>manifest</dt><dd><a href="/${esc(relFromLab)}/manifest.json">manifest.json</a></dd>
  </dl>
</article>`
    })
    .join('\n')

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<title>batch ${esc(batchId)}</title>
<style>
  :root { font-family: system-ui, sans-serif; color: #111; background: #fafafa; }
  body { margin: 0 auto; max-width: 1100px; padding: 1.5rem; }
  h1 { font-size: 1.4rem; }
  .run { background: #fff; border: 1px solid #ddd; border-radius: 10px; padding: 1rem; margin: 1rem 0; }
  .shot img { max-width: 100%; border: 1px solid #eee; border-radius: 6px; }
  .missing { background: #fee; color: #900; padding: 2rem; text-align: center; border-radius: 6px; }
  dl { display: grid; grid-template-columns: 8rem 1fr; gap: 0.25rem 0.75rem; font-size: 0.9rem; }
  dt { color: #666; } code { font-size: 0.8rem; word-break: break-all; }
  small { color: #666; font-weight: normal; }
</style>
</head>
<body>
<header>
  <h1>Batch <code>${esc(batchId)}</code></h1>
  <p>${esc(batch.name)} — status: <strong>${esc(batch.status)}</strong> — ${runs.length} run(s)</p>
  <p><em>Generated index — artifacts referenced in place, zero copied bytes.</em></p>
</header>
<main>
${cards || '<p>No runs in this batch.</p>'}
</main>
</body>
</html>
`
}

function reasonSuffix(r: RunRow): string {
  try {
    const m = JSON.parse(r.missing_reasons_json || '{}') as Record<string, string>
    if (m.render) return ` (${m.render})`
  } catch {
    /* ignore */
  }
  return ''
}

function fmtNull(v: number | null | undefined): string {
  return v === null || v === undefined ? 'NULL' : String(v)
}

function safeJsonArray(s: string | null): string[] {
  if (!s) return []
  try {
    const v = JSON.parse(s)
    return Array.isArray(v) ? v.map(String) : []
  } catch {
    return []
  }
}

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}
