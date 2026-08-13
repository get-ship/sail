#!/usr/bin/env node
// PERF-7: sample steady-state RSS of `sail dev` + Studio tab.
// Does not change product flags. Does not invent numbers if boot is blocked.
//
//   bun tests/perf/scripts/profile-sail-dev-ram.mjs
//   bun tests/perf/scripts/profile-sail-dev-ram.mjs --out tests/perf/baselines/custom-ram.json
//
// Samples after HTTP serve (not during cargo regen), then after 1 and 10 HMR
// edits of a tiny Studio route. Browser renderer is sampled only if Studio serves.

import { spawn, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import http from 'node:http'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(__dirname, '../../..')
const DEFAULT_OUT = resolve(REPO_ROOT, 'tests/perf/baselines/2026-08-13-ram.json')
const PROJECT = 'apps/studio'
const ADDR = '127.0.0.1:18767'
const STUDIO_PATH = '/tasks'
const HMR_REL = 'apps/studio/routes/tasks/page.sail'
const HMR_EDITS = 10
const BOOT_MS = 180_000
const REGEN_BOOT_MS = 480_000
const HMR_MS = 90_000
const SETTLE_MS = 2_000

const outIdx = process.argv.indexOf('--out')
const out = outIdx >= 0 ? resolve(process.argv[outIdx + 1]) : DEFAULT_OUT

const sailBin = resolve(REPO_ROOT, 'target/debug/sail')
const hmrFile = resolve(REPO_ROOT, HMR_REL)
const studioUrl = `http://${ADDR}${STUDIO_PATH}`

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms))
}

function gitSha() {
  const r = spawnSync('git', ['rev-parse', '--short', 'HEAD'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
  })
  return r.status === 0 ? r.stdout.trim() : 'unknown'
}

function today() {
  const r = spawnSync('date', ['+%Y-%m-%d'], { encoding: 'utf8' })
  return r.status === 0 ? r.stdout.trim() : 'unknown'
}

function httpOk(url, timeoutMs = 2000) {
  return new Promise((resolveOk) => {
    try {
      const u = new URL(url)
      const req = http.request(
        {
          hostname: u.hostname,
          port: u.port || 80,
          path: u.pathname || '/',
          method: 'GET',
          timeout: timeoutMs,
        },
        (res) => {
          res.resume()
          resolveOk((res.statusCode ?? 500) < 500)
        },
      )
      req.on('error', () => resolveOk(false))
      req.on('timeout', () => {
        req.destroy()
        resolveOk(false)
      })
      req.end()
    } catch {
      resolveOk(false)
    }
  })
}

function readStatus(pid) {
  const text = readFileSync(`/proc/${pid}/status`, 'utf8')
  const field = (name) => {
    const line = text.split('\n').find((l) => l.startsWith(`${name}:`))
    return line ? line.slice(name.length + 1).trim() : ''
  }
  const kb = (name) => {
    const rest = field(name)
    const n = parseInt(rest, 10)
    return Number.isFinite(n) ? n : null
  }
  return {
    pid,
    comm: field('Name') || null,
    vm_rss_kb: kb('VmRSS'),
    vm_swap_kb: kb('VmSwap'),
    ppid: kb('PPid'),
  }
}

function readCmdline(pid) {
  try {
    return readFileSync(`/proc/${pid}/cmdline`, 'utf8').replace(/\u0000/g, ' ').trim()
  } catch {
    return ''
  }
}

function childrenOf(pid) {
  const kids = new Set()
  try {
    const taskDir = `/proc/${pid}/task`
    for (const tid of readdirSync(taskDir)) {
      try {
        const raw = readFileSync(`${taskDir}/${tid}/children`, 'utf8').trim()
        for (const tok of raw.split(/\s+/)) {
          const n = Number(tok)
          if (Number.isFinite(n) && n > 0) kids.add(n)
        }
      } catch {
        /* task vanished */
      }
    }
  } catch {
    /* process vanished */
  }
  return [...kids]
}

function descendants(rootPid) {
  const outPids = []
  const seen = new Set()
  const stack = [rootPid]
  while (stack.length) {
    const pid = stack.pop()
    if (seen.has(pid)) continue
    seen.add(pid)
    outPids.push(pid)
    for (const c of childrenOf(pid)) stack.push(c)
  }
  return outPids
}

function classifyRole(cmdline, comm, { browserRoot }) {
  const cmd = cmdline.toLowerCase()
  if (cmd.includes('--type=renderer')) return 'chrome-renderer'
  if (cmd.includes('--type=gpu-process')) return 'chrome-gpu'
  if (cmd.includes('--type=utility')) return 'chrome-utility'
  if (cmd.includes('--type=zygote')) return 'chrome-zygote'
  if (cmd.includes('tsgo') || comm === 'tsgo') return 'tsgo'
  if (/\bcargo\b/.test(cmd) || comm === 'cargo') return 'cargo'
  if (/\brustc\b/.test(cmd) || comm === 'rustc') return 'rustc'
  if (cmd.includes('rolldown') && !cmd.includes('sail')) return 'rolldown'
  if (browserRoot && (cmd.includes('chrome') || comm === 'chrome' || comm === 'chromium')) {
    return 'chrome-browser'
  }
  if (cmd.includes('/sail') || /(^|\/)sail(\s|$)/.test(cmd) || comm === 'sail') {
    return 'sail-daemon'
  }
  return comm || 'other'
}

function snapshotPids(pids, { browserRoot }) {
  const processes = []
  for (const pid of pids) {
    try {
      const st = readStatus(pid)
      const cmdline = readCmdline(pid)
      const role = classifyRole(cmdline, st.comm, { browserRoot })
      processes.push({
        pid,
        ppid: st.ppid,
        role,
        comm: st.comm,
        cmdline: cmdline.slice(0, 240),
        vm_rss_kb: st.vm_rss_kb,
        vm_swap_kb: st.vm_swap_kb,
      })
    } catch {
      /* gone */
    }
  }
  processes.sort((a, b) => a.pid - b.pid)
  const sum_rss_kb = processes.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0)
  const sum_swap_kb = processes.reduce((s, p) => s + (p.vm_swap_kb ?? 0), 0)
  return { processes, sum_rss_kb, sum_swap_kb }
}

function findListenerPid(port) {
  const ss = spawnSync('ss', ['-ltnp', `sport = :${port}`], { encoding: 'utf8' })
  if (ss.status === 0 && ss.stdout) {
    const m = ss.stdout.match(/pid=(\d+)/)
    if (m) return Number(m[1])
  }
  // Fallback: scan /proc/*/cmdline for this addr.
  const needle = `--addr ${ADDR}`
  try {
    for (const ent of readdirSync('/proc')) {
      if (!/^\d+$/.test(ent)) continue
      const cmd = readCmdline(Number(ent))
      if (cmd.includes(needle) || cmd.includes(ADDR)) {
        const comm = (() => {
          try {
            return readStatus(Number(ent)).comm
          } catch {
            return ''
          }
        })()
        if (comm === 'sail' || cmd.includes('/sail')) return Number(ent)
      }
    }
  } catch {
    /* ignore */
  }
  return null
}

function treeHasRole(processes, role) {
  return processes.some((p) => p.role === role)
}

function logLooksLikeLintOrRegenFail(log) {
  const t = log.toLowerCase()
  if (t.includes('bindingpatternkind')) return 'sail-lint compile failed (BindingPatternKind)'
  if (t.includes('sail-lint') && (t.includes('error[') || t.includes('could not compile'))) {
    return 'sail-lint / framework regen failed to compile'
  }
  const crateFail = log.match(/error: could not compile `([^`]+)`/)
  if (crateFail) {
    const rustc = log.match(
      /error\[E[0-9]+\]: ([^\n]+)\n\s+-->\s+(\S+:\d+:\d+)/,
    )
    const bits = [`framework regen: cargo could not compile \`${crateFail[1]}\``]
    if (rustc) bits.push(rustc[2], rustc[1].trim())
    return bits.join(' — ')
  }
  if (t.includes('are still stale after an automatic')) {
    return 'framework regen still stale after automatic rebuild'
  }
  if (t.includes('automatic build-assets (staleness regen) failed')) {
    const parse = log.match(/target failed target=([^\s]+)[\s\S]*?message: "([^"]+)"/)
    if (parse) {
      return `framework regen (build-assets) failed — ${parse[1]}: ${parse[2]}`
    }
    return 'framework regen (build-assets) failed'
  }
  if (t.includes('initial app build failed')) return 'sail dev: initial app build failed'
  return null
}

function logLooksLikeRegen(log) {
  return (
    log.includes('STALE framework') ||
    log.includes('regenerating automatically') ||
    log.includes('rebuilding + restarting with the fresh bake')
  )
}

async function waitForServe(child, logBuf) {
  const started = Date.now()
  let regen = false
  let lastFail = null
  while (Date.now() - started < (regen ? REGEN_BOOT_MS : BOOT_MS)) {
    const log = logBuf.text()
    if (!regen && logLooksLikeRegen(log)) regen = true
    lastFail = logLooksLikeLintOrRegenFail(log)
    if (lastFail && child.exitCode != null) {
      return { ok: false, regen, reason: lastFail }
    }
    const served = log.includes('sail dev: serving') && (await httpOk(`http://${ADDR}/`))
    if (served) {
      return { ok: true, regen, reason: null }
    }
    if (child.exitCode != null) {
      return {
        ok: false,
        regen,
        reason:
          lastFail ||
          `sail exited before serve (code=${child.exitCode}). ${log.slice(-800)}`,
      }
    }
    await sleep(400)
  }
  return {
    ok: false,
    regen,
    reason:
      lastFail ||
      `timed out waiting for sail dev to serve on ${ADDR} (${regen ? 'regen window' : 'boot window'})`,
  }
}

async function waitForHmr(logBuf, beforeCount) {
  const started = Date.now()
  while (Date.now() - started < HMR_MS) {
    const log = logBuf.text()
    const hits = log.split('sail dev: rebuilt app (HMR)').length - 1
    if (hits > beforeCount) return { ok: true, hits }
    if (log.includes('sail dev: rebuild failed')) {
      return { ok: false, hits, reason: 'HMR rebuild failed' }
    }
    await sleep(200)
  }
  return { ok: false, hits: beforeCount, reason: 'timed out waiting for HMR rebuild' }
}

function hmrCount(log) {
  return log.split('sail dev: rebuilt app (HMR)').length - 1
}

function applyHmrMarker(src, n) {
  const marker = `<!-- perf7-hmr ${n} -->`
  if (src.includes('<!-- perf7-hmr ')) {
    return src.replace(/<!-- perf7-hmr \d+ -->/, marker)
  }
  if (!src.endsWith('\n')) return `${src}\n${marker}\n`
  return `${src}${marker}\n`
}

function killTree(child) {
  if (!child) return
  const root = child.pid
  let pids = []
  try {
    pids = descendants(root)
  } catch {
    pids = root ? [root] : []
  }
  const extra = findListenerPid(Number(ADDR.split(':')[1]))
  if (extra && !pids.includes(extra)) pids.push(extra)
  for (const pid of pids) {
    try {
      process.kill(pid, 'SIGTERM')
    } catch {
      /* already gone */
    }
  }
  const deadline = Date.now() + 2500
  while (Date.now() < deadline) {
    let alive = false
    for (const pid of pids) {
      try {
        process.kill(pid, 0)
        alive = true
      } catch {
        /* gone */
      }
    }
    if (!alive) return
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50)
  }
  for (const pid of pids) {
    try {
      process.kill(pid, 'SIGKILL')
    } catch {
      /* gone */
    }
  }
}

function writeReport(report) {
  mkdirSync(dirname(out), { recursive: true })
  writeFileSync(out, `${JSON.stringify(report, null, 2)}\n`)
  console.log(`wrote ${out}`)
}

function blockedReport({ reason, log, regen }) {
  return {
    captured: today(),
    label: 'perf-7-ram',
    git_sha: gitSha(),
    status: 'blocked',
    blocked_reason: reason,
    project: PROJECT,
    addr: ADDR,
    sail_bin: sailBin,
    cargo_regen_observed: Boolean(regen),
    ram: {
      sail_dev: {
        attempted: true,
        served: false,
        rss_kb: null,
        note: reason,
        snapshots: {},
      },
      browser: {
        attempted: false,
        served: false,
        rss_kb: null,
        note: 'not sampled — sail dev did not serve',
        snapshots: {},
      },
    },
    table: [],
    verdict: {
      claim_1gb: 'blocked',
      under_500mb: null,
      dominant: null,
      follow_up: reason,
    },
    log_tail: (log || '').slice(-2000),
  }
}

async function collectSnapshot(label, { spawnPid, browserPid }) {
  const port = Number(ADDR.split(':')[1])
  const listener = findListenerPid(port) || spawnPid
  const sailPids = new Set(descendants(listener))
  if (spawnPid) {
    for (const p of descendants(spawnPid)) sailPids.add(p)
  }
  // Prefer the process that actually bound the port as the daemon root.
  const daemonTree = new Set(descendants(listener))
  const browserTree = browserPid ? new Set(descendants(browserPid)) : new Set()
  const all = new Set([...daemonTree, ...browserTree])
  const snap = snapshotPids([...all], {
    browserRoot: browserPid || null,
  })
  // Drop cargo/rustc from the ARC sum — those are regen, not steady state.
  const steady = snap.processes.filter((p) => p.role !== 'cargo' && p.role !== 'rustc')
  const sail = steady.filter((p) => p.role === 'sail-daemon' || p.role === 'rolldown' || p.role === 'tsgo')
  const renderer = steady.filter((p) => p.role === 'chrome-renderer')
  const browser = steady.filter((p) => String(p.role).startsWith('chrome-'))
  return {
    label,
    at_ms: Date.now(),
    listener_pid: listener,
    processes: snap.processes,
    sum_rss_kb: snap.sum_rss_kb,
    sum_swap_kb: snap.sum_swap_kb,
    sum_steady_rss_kb: steady.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0),
    sum_sail_rss_kb: sail.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0),
    sum_browser_rss_kb: browser.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0),
    sum_renderer_rss_kb: renderer.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0),
    has_cargo: treeHasRole(snap.processes, 'cargo') || treeHasRole(snap.processes, 'rustc'),
  }
}

function tableFromSnapshots(snaps) {
  const rows = []
  for (const snap of snaps) {
    for (const p of snap.processes) {
      if (p.role === 'cargo' || p.role === 'rustc') continue
      rows.push({
        phase: snap.label,
        pid: p.pid,
        role: p.role,
        vm_rss_kb: p.vm_rss_kb,
        vm_swap_kb: p.vm_swap_kb,
      })
    }
  }
  return rows
}

function verdictFrom(boot, after10) {
  const bootSum = boot?.sum_steady_rss_kb ?? 0
  const afterSum = after10?.sum_steady_rss_kb ?? 0
  const peak = Math.max(bootSum, afterSum)
  const peakSnap = afterSum >= bootSum ? after10 : boot
  let dominant = null
  if (peakSnap) {
    const ranked = [...peakSnap.processes]
      .filter((p) => p.role !== 'cargo' && p.role !== 'rustc')
      .sort((a, b) => (b.vm_rss_kb ?? 0) - (a.vm_rss_kb ?? 0))
    if (ranked[0]) {
      dominant = {
        pid: ranked[0].pid,
        role: ranked[0].role,
        vm_rss_kb: ranked[0].vm_rss_kb,
      }
    }
  }
  const under500 = peak < 500 * 1024
  const over1g = peak >= 1024 * 1024
  return {
    sum_rss_kb_boot: bootSum,
    sum_rss_kb_after_10_hmr: afterSum,
    peak_sum_rss_kb: peak,
    claim_1gb: over1g ? 'named' : under500 ? 'killed' : 'below_1gb_above_500mb',
    under_500mb: under500,
    dominant: over1g || !under500 ? dominant : dominant,
    follow_up: over1g
      ? `sum ≥ 1GB; dominant pid ${dominant?.pid} (${dominant?.role}, ${dominant?.vm_rss_kb} KB)`
      : under500
        ? null
        : `sum ${peak} KB is under 1GB but over the 500MB ARC; leftover is ${dominant?.role} pid ${dominant?.pid}`,
  }
}

async function main() {
  if (!existsSync(sailBin)) {
    writeReport(
      blockedReport({
        reason: 'target/debug/sail missing — RSS of sail-dev not sampled',
        log: '',
        regen: false,
      }),
    )
    process.exit(2)
  }
  if (!existsSync(hmrFile)) {
    writeReport(
      blockedReport({
        reason: `${HMR_REL} missing — cannot drive HMR edits`,
        log: '',
        regen: false,
      }),
    )
    process.exit(2)
  }

  const originalHmr = readFileSync(hmrFile, 'utf8')
  const logChunks = []
  const logBuf = {
    text: () => logChunks.join(''),
    push: (buf) => {
      const s = String(buf)
      logChunks.push(s)
      process.stderr.write(s)
    },
  }

  let child = null
  let browser = null
  let startedByUs = true
  let exitCode = 0

  const cleanup = async () => {
    try {
      writeFileSync(hmrFile, originalHmr)
    } catch {
      /* restore best-effort */
    }
    if (browser) {
      try {
        await browser.close()
      } catch {
        /* ignore */
      }
      browser = null
    }
    if (startedByUs) killTree(child)
  }

  for (const sig of ['SIGINT', 'SIGTERM']) {
    process.on(sig, () => {
      cleanup().finally(() => process.exit(130))
    })
  }

  try {
    console.error(`[perf-7] starting ${sailBin} dev --project ${PROJECT} --addr ${ADDR}`)
    child = spawn(sailBin, ['dev', '--project', PROJECT, '--addr', ADDR], {
      cwd: REPO_ROOT,
      stdio: ['ignore', 'pipe', 'pipe'],
      env: process.env,
    })
    child.stdout.on('data', (b) => logBuf.push(b))
    child.stderr.on('data', (b) => logBuf.push(b))
    child.on('exit', (code, signal) => {
      console.error(`[perf-7] sail pid=${child.pid} exited code=${code} signal=${signal ?? ''}`)
    })

    const boot = await waitForServe(child, logBuf)
    if (!boot.ok) {
      writeReport(blockedReport({ reason: boot.reason, log: logBuf.text(), regen: boot.regen }))
      exitCode = 2
      return
    }

    // Leave cargo-regen (if any) before sampling. Serving means the new
    // binary is up; rustc/cargo children should drain. Do not treat their
    // peak as steady state.
    const settleDeadline = Date.now() + 30_000
    let spawnPid = child.pid
    while (Date.now() < settleDeadline) {
      const listener = findListenerPid(Number(ADDR.split(':')[1])) || spawnPid
      const procs = snapshotPids(descendants(listener), {
        browserRoot: null,
      }).processes
      if (!treeHasRole(procs, 'cargo') && !treeHasRole(procs, 'rustc')) break
      await sleep(400)
    }
    await sleep(SETTLE_MS)

    if (!(await httpOk(`http://${ADDR}/`))) {
      writeReport(
        blockedReport({
          reason: 'HTTP dropped after boot settle — not sampling',
          log: logBuf.text(),
          regen: boot.regen,
        }),
      )
      exitCode = 2
      return
    }

    let browserPid = null
    let browserNote = 'Studio served; Playwright Chromium opened the Studio tab'
    let page = null
    try {
      const { chromium } = await import('playwright')
      browser = await chromium.launch({ headless: true })
      const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } })
      page = await ctx.newPage()
      await page.goto(studioUrl, { waitUntil: 'domcontentloaded', timeout: 60_000 })
      await sleep(1500)
      browserPid = browser.process()?.pid ?? null
      if (!browserPid) browserNote = 'Playwright launched but browser.process() pid was null'
    } catch (e) {
      browserNote = `Studio served; browser RSS not sampled: ${e.message}`
      browser = null
      browserPid = null
    }

    const snapBoot = await collectSnapshot('boot', { spawnPid, browserPid })
    if (snapBoot.has_cargo) {
      writeReport(
        blockedReport({
          reason: 'cargo/rustc still in the tree after serve — refusing to treat regen as steady state',
          log: logBuf.text(),
          regen: true,
        }),
      )
      exitCode = 2
      return
    }

    let snap1 = null
    let snap10 = null
    const hmrResults = []
    try {
      for (let i = 1; i <= HMR_EDITS; i++) {
        const before = hmrCount(logBuf.text())
        writeFileSync(hmrFile, applyHmrMarker(originalHmr, i))
        const hmr = await waitForHmr(logBuf, before)
        hmrResults.push({ edit: i, ...hmr })
        if (!hmr.ok) {
          const snaps = [snapBoot, snap1].filter(Boolean)
          writeReport({
            captured: today(),
            label: 'perf-7-ram',
            git_sha: gitSha(),
            status: 'blocked',
            blocked_reason: `HMR edit ${i}/${HMR_EDITS} failed: ${hmr.reason}`,
            project: PROJECT,
            addr: ADDR,
            sail_bin: sailBin,
            hmr_file: HMR_REL,
            hmr_edits: i - 1,
            hmr_results: hmrResults,
            cargo_regen_observed: boot.regen,
            studio_url: studioUrl,
            ram: {
              sail_dev: {
                attempted: true,
                served: true,
                rss_kb: snapBoot.sum_sail_rss_kb,
                note: `Boot sampled. HMR stopped at edit ${i}: ${hmr.reason}`,
                snapshots: {
                  boot: slimSnap(snapBoot, 'sail'),
                  after_1_hmr: snap1 ? slimSnap(snap1, 'sail') : null,
                  after_10_hmr: null,
                },
              },
              browser: {
                attempted: Boolean(browserPid),
                served: Boolean(browserPid),
                rss_kb: snapBoot.sum_renderer_rss_kb,
                url: studioUrl,
                note: browserNote,
                snapshots: {
                  boot: slimSnap(snapBoot, 'browser'),
                  after_1_hmr: snap1 ? slimSnap(snap1, 'browser') : null,
                  after_10_hmr: null,
                },
              },
            },
            table: tableFromSnapshots(snaps),
            verdict: {
              claim_1gb: 'blocked',
              under_500mb: null,
              dominant: null,
              follow_up: `HMR edit ${i} failed after a successful boot — boot RSS is real; after-10 not invented`,
              sum_rss_kb_boot: snapBoot.sum_steady_rss_kb,
            },
            log_tail: logBuf.text().slice(-2000),
          })
          exitCode = 2
          return
        }
        await sleep(SETTLE_MS)
        if (i === 1) snap1 = await collectSnapshot('after_1_hmr', { spawnPid, browserPid })
      }
      snap10 = await collectSnapshot('after_10_hmr', { spawnPid, browserPid })
    } finally {
      writeFileSync(hmrFile, originalHmr)
    }

    const snaps = [snapBoot, snap1, snap10].filter(Boolean)
    const table = tableFromSnapshots(snaps)
    const v = verdictFrom(snapBoot, snap10)

    const report = {
      captured: today(),
      label: 'perf-7-ram',
      git_sha: gitSha(),
      status: 'ok',
      blocked_reason: null,
      project: PROJECT,
      addr: ADDR,
      sail_bin: sailBin,
      hmr_file: HMR_REL,
      hmr_edits: HMR_EDITS,
      hmr_results: hmrResults,
      cargo_regen_observed: boot.regen,
      studio_url: studioUrl,
      ram: {
        sail_dev: {
          attempted: true,
          served: true,
          rss_kb: snap10.sum_sail_rss_kb,
          note:
            'Steady-state VmRSS of the serving sail daemon + in-tree children (Rolldown is in-process; tsgo only if still live). Cargo/rustc excluded. Boot is post-HTTP-serve, not the cargo-regen peak.',
          snapshots: {
            boot: slimSnap(snapBoot, 'sail'),
            after_1_hmr: snap1 ? slimSnap(snap1, 'sail') : null,
            after_10_hmr: slimSnap(snap10, 'sail'),
          },
        },
        browser: {
          attempted: Boolean(browserPid) || browserNote.includes('Playwright'),
          served: Boolean(browserPid),
          rss_kb: snap10.sum_renderer_rss_kb,
          url: studioUrl,
          note: browserNote,
          snapshots: {
            boot: slimSnap(snapBoot, 'browser'),
            after_1_hmr: snap1 ? slimSnap(snap1, 'browser') : null,
            after_10_hmr: slimSnap(snap10, 'browser'),
          },
        },
      },
      table,
      verdict: v,
    }
    writeReport(report)
    printTable(table, v)
  } catch (err) {
    writeReport(
      blockedReport({
        reason: `sampler crashed: ${err.message}`,
        log: logBuf.text(),
        regen: false,
      }),
    )
    exitCode = 1
  } finally {
    await cleanup()
  }
  process.exit(exitCode)
}

function slimSnap(snap, kind) {
  const roles =
    kind === 'sail'
      ? new Set(['sail-daemon', 'rolldown', 'tsgo'])
      : new Set(
          snap.processes.filter((p) => String(p.role).startsWith('chrome-')).map((p) => p.role),
        )
  const processes = snap.processes.filter((p) =>
    kind === 'sail' ? roles.has(p.role) : String(p.role).startsWith('chrome-'),
  )
  return {
    listener_pid: snap.listener_pid,
    processes,
    sum_rss_kb: processes.reduce((s, p) => s + (p.vm_rss_kb ?? 0), 0),
    sum_swap_kb: processes.reduce((s, p) => s + (p.vm_swap_kb ?? 0), 0),
  }
}

function printTable(table, v) {
  console.log('\nphase\tpid\trole\tVmRSS_KB\tVmSwap_KB')
  for (const r of table) {
    console.log(`${r.phase}\t${r.pid}\t${r.role}\t${r.vm_rss_kb}\t${r.vm_swap_kb}`)
  }
  console.log(
    `\nverdict peak=${v.peak_sum_rss_kb} KB claim_1gb=${v.claim_1gb} under_500mb=${v.under_500mb}`,
  )
  if (v.dominant) {
    console.log(`dominant pid=${v.dominant.pid} role=${v.dominant.role} rss=${v.dominant.vm_rss_kb} KB`)
  }
}

main()
