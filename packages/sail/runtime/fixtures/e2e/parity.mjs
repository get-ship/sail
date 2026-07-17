#!/usr/bin/env node
// View parity harness — j2 template vs SAIL template structural comparison.
//
// Default: static source analysis (no daemon needed).
// Playwright mode (PARITY_PLAYWRIGHT=1): boots shipd, uses live rendering.
//
// Run: node packages/sail/runtime/fixtures/e2e/parity.mjs
// Output: packages/sail/runtime/fixtures/e2e/parity-baselines/parity-report.md

import { chromium } from 'playwright'
import { readFileSync, existsSync, mkdirSync, writeFileSync } from 'fs'
import { spawn } from 'child_process'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.resolve(__dirname, '../../../..')
const J2_ROOT = path.join(ROOT, '.ship/views')
const SUI_ROOT = path.join(ROOT, 'views')
const BASELINES = path.join(__dirname, 'parity-baselines')
const VIEWS_FILE = path.join(__dirname, 'parity-views.json')
const BASE = process.env.SUI_E2E_BASE_URL || 'http://127.0.0.1:9391'
const USE_PLAYWRIGHT = process.env.PARITY_PLAYWRIGHT === '1'

mkdirSync(BASELINES, { recursive: true })

const VIEWS = JSON.parse(readFileSync(VIEWS_FILE, 'utf8'))

// ── Template strippers ────────────────────────────────────────────────────

function stripJinja(src) {
  return src
    .replace(/{%-?[\s\S]*?-?%}/g, '')  // {% blocks %}
    .replace(/\{\{[\s\S]*?\}\}/g, 'DYN')  // {{ expressions }}
    .replace(/\{#[\s\S]*?#\}/g, '')  // {# comments #}
}

function stripSui(src) {
  return src
    .replace(/<data [^>]*\/?>/g, '')  // <data ...> directives
    .replace(/\{#[a-z][\w-]*[^}]*\}/g, '')  // {#if}, {#for}, {#while-*}
    .replace(/\{:[a-z][\w-]*[^}]*\}/g, '')  // {:else}, {:else if}
    .replace(/\{\/[a-z][\w-]*\}/g, '')  // {/if}, {/for}
    .replace(/\{[^}]+\}/g, 'DYN')  // {expressions}
}

// ── Content extraction ────────────────────────────────────────────────────

function extractContent(html) {
  const headings = []
  for (const m of html.matchAll(/<h([1-6])[^>]*>([\s\S]*?)<\/h[1-6]>/gi)) {
    const text = m[2].replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim()
    if (text && text !== 'DYN' && text.length > 1) headings.push({ level: +m[1], text })
  }

  // Section macro calls like section('Providers') used in j2 partials
  const sectionMacros = [...html.matchAll(/section\(['"]([^'"]{2,})['"]/g)].map(m => m[1])

  const ariaLabels = [...html.matchAll(/aria-label="([^"]+)"/gi)].map(m => m[1])
    .filter(v => v.length > 1 && !v.startsWith('DYN'))

  const slotTitles = []
  for (const m of html.matchAll(/slot="title"[^>]*>([\s\S]*?)<\//gi)) {
    const text = m[1].replace(/<[^>]+>/g, '').trim()
    if (text && text !== 'DYN') slotTitles.push(text)
  }

  return { headings, sectionMacros, ariaLabels, slotTitles }
}

// ── J2 loading (resolves includes) ───────────────────────────────────────

function loadJ2(name) {
  const main = path.join(J2_ROOT, name, 'index.html.j2')
  if (!existsSync(main)) return null
  let src = readFileSync(main, 'utf8')
  const seen = new Set([main])
  // BFS: re-scan after each append so nested includes are resolved.
  let changed = true
  while (changed) {
    changed = false
    for (const m of src.matchAll(/include\s+'([^'_][^']*\.j2)'/g)) {
      const rel = m[1]
      if (rel.startsWith('_')) continue  // skip system partials (_ship, _urls)
      const abs = path.join(J2_ROOT, rel)
      if (!seen.has(abs) && existsSync(abs)) {
        seen.add(abs)
        src += '\n' + readFileSync(abs, 'utf8')
        changed = true
      }
    }
  }
  return extractContent(stripJinja(src))
}

function loadSui(name) {
  const main = path.join(SUI_ROOT, name, 'page.sail')
  if (!existsSync(main)) return null
  return extractContent(stripSui(readFileSync(main, 'utf8')))
}

// ── Diff logic ────────────────────────────────────────────────────────────

function inSui(needle, haystack) {
  const n = needle.toLowerCase()
  return haystack.some(s => s.toLowerCase() === n)
}

function diffContent(j2, sail) {
  const missing = []
  for (const h of j2.headings) {
    if (!inSui(h.text, sail.headings.map(s => s.text))) {
      missing.push(`h${h.level}: "${h.text}"`)
    }
  }
  for (const s of j2.sectionMacros) {
    const allHeadings = [...sail.headings.map(h => h.text), ...sail.sectionMacros, ...sail.ariaLabels]
    if (!inSui(s, allHeadings)) {
      missing.push(`section: "${s}"`)
    }
  }
  for (const lbl of j2.ariaLabels) {
    if (!inSui(lbl, sail.ariaLabels)) {
      missing.push(`aria-label: "${lbl}"`)
    }
  }
  for (const t of j2.slotTitles) {
    if (!inSui(t, sail.slotTitles)) {
      missing.push(`slot[title]: "${t}"`)
    }
  }
  return missing
}

function elementCount(c) {
  return c.headings.length + c.sectionMacros.length + c.ariaLabels.length + c.slotTitles.length
}

// ── Playwright mode ───────────────────────────────────────────────────────

async function findShipd() {
  // The sail repo root (5 up from this fixture), then the sibling ship repo
  // where shipd is built (post-nest layout: <parent>/{sail,ship}). Derived from
  // this file's dir so it survives the checkout moving; SHIPD_BIN overrides.
  // (Note: the module-level ROOT is sail/packages, not the repo root — don't
  //  reuse it here.)
  const suiRoot = path.resolve(__dirname, '../../../../..')
  const candidates = [
    process.env.SHIPD_BIN,
    path.join(suiRoot, 'target/debug/shipd'),
    path.join(suiRoot, '../ship/target/debug/shipd'),
  ].filter(Boolean)
  return candidates.find(p => existsSync(p)) ?? null
}

async function startDaemon(bin) {
  const env = { ...process.env, SHIP_HOME: '/tmp/ship-parity-home', SHIP_DEV_PROJECT_DIR: ROOT }
  const proc = spawn(bin, ['--port', '9391', '--dev', '--workspace', ROOT],
    { env, detached: true, stdio: 'ignore' })
  proc.unref()
  for (let i = 0; i < 24; i++) {
    await new Promise(r => setTimeout(r, 500))
    try {
      const res = await fetch(`${BASE}/api/health`, { signal: AbortSignal.timeout(400) })
      if (res.ok) return proc.pid
    } catch { /* still starting */ }
  }
  return null
}

async function getDevToken() {
  const init = await fetch(`${BASE}/auth/pair/init`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ client_label: 'parity-test' }),
  }).then(r => r.json())
  if (!init.dev_mode) throw new Error('daemon not in dev mode')
  const status = await fetch(`${BASE}/auth/pair/status?claim_token=${init.claim_token}`)
    .then(r => r.json())
  if (status.status !== 'approved') throw new Error('dev-mode pair not approved')
  return status.session_token
}

async function capturePage(browser, url, authToken) {
  const ctx = await browser.newContext()
  const page = await ctx.newPage()
  if (authToken) await page.setExtraHTTPHeaders({ Authorization: `Bearer ${authToken}` })
  await page.goto(url, { waitUntil: 'networkidle', timeout: 15000 })
  const html = await page.evaluate(() => document.documentElement.outerHTML)
  const shot = await page.screenshot({ fullPage: true })
  await ctx.close()
  return { html, shot }
}

// ── Main ──────────────────────────────────────────────────────────────────

async function run() {
  const results = []
  let daemonPid = null
  let browser = null
  let authToken = null
  let playwright = USE_PLAYWRIGHT

  if (playwright) {
    const bin = await findShipd()
    if (!bin) {
      console.warn('no shipd binary found — falling back to static analysis')
      playwright = false
    } else {
      console.log(`starting daemon: ${bin}`)
      daemonPid = await startDaemon(bin)
      if (!daemonPid) {
        console.warn('daemon failed to start — falling back to static analysis')
        playwright = false
      } else {
        console.log(`daemon ready (pid ${daemonPid})`)
        browser = await chromium.launch()
        authToken = await getDevToken()
      }
    }
  }

  const mode = playwright ? 'playwright' : 'static'
  console.log(`mode: ${mode}`)

  for (const v of VIEWS) {
    process.stdout.write(`  ${v.name}... `)
    const row = { name: v.name, j2Count: 0, suiCount: 0, missing: [], error: null }

    try {
      let j2, sail
      if (playwright) {
        const j2Cap = await capturePage(browser, `${BASE}/api/v1/v/${v.j2}`, authToken)
        const suiCap = await capturePage(browser, `${BASE}/sail/${v.sail}`, null)
        j2 = extractContent(stripJinja(j2Cap.html))
        sail = extractContent(stripSui(suiCap.html))
        row.missing = diffContent(j2, sail)
        if (row.missing.length > 0) {
          writeFileSync(path.join(BASELINES, `${v.name}-j2.png`), j2Cap.shot)
          writeFileSync(path.join(BASELINES, `${v.name}-sail.png`), suiCap.shot)
        }
      } else {
        j2 = loadJ2(v.j2)
        sail = loadSui(v.sail)
        if (!j2) { row.error = 'j2 not found'; results.push(row); console.log('skip'); continue }
        if (!sail) { row.error = 'SAIL not found'; results.push(row); console.log('skip'); continue }
        row.missing = diffContent(j2, sail)
      }
      row.j2Count = elementCount(j2)
      row.suiCount = elementCount(sail)
      console.log(row.missing.length === 0 ? '✓' : `${row.missing.length} gap(s)`)
    } catch (e) {
      row.error = e.message
      console.log(`error: ${e.message}`)
    }
    results.push(row)
  }

  if (browser) await browser.close()
  if (daemonPid) {
    try { process.kill(daemonPid, 'SIGTERM') } catch { /* already gone */ }
  }

  // ── Report ─────────────────────────────────────────────────────────────
  const ok = results.filter(r => !r.error && r.missing.length === 0).length
  const gaps = results.filter(r => r.missing.length > 0)
  const errored = results.filter(r => r.error)

  let report = `# View Parity Report\n\n`
  report += `**Mode:** ${mode === 'playwright' ? 'Playwright (live rendering)' : 'Static source analysis'}  \n`
  report += `**Date:** ${new Date().toISOString().slice(0, 10)}  \n`
  report += `**Views:** ${results.length} compared, ${ok} at parity, ${gaps.length} with gaps`
  if (errored.length) report += `, ${errored.length} errored`
  report += `\n\n`

  report += `| View | j2 elements | SAIL elements | Missing in SAIL |\n`
  report += `|---|---|---|---|\n`
  for (const r of results) {
    if (r.error) {
      report += `| ${r.name} | — | — | ⚠ ${r.error} |\n`
    } else {
      const cell = r.missing.length > 0 ? r.missing.join('; ') : '(none) ✓'
      report += `| ${r.name} | ${r.j2Count} | ${r.suiCount} | ${cell} |\n`
    }
  }

  if (gaps.length > 0) {
    report += `\n## Gaps detail\n\n`
    for (const r of gaps) {
      report += `### ${r.name}\n\n`
      for (const g of r.missing) report += `- ${g}\n`
      report += '\n'
    }
  }

  const reportPath = path.join(BASELINES, 'parity-report.md')
  writeFileSync(reportPath, report)
  console.log(`\nreport: ${reportPath}`)
  console.log(`${ok}/${results.length} views at parity`)
}

run().catch(e => {
  console.error('parity harness fatal:', e.message)
  process.exit(1)
})
