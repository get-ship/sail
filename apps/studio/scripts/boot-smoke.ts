#!/usr/bin/env bun
/**
 * In-agent recipe for STUDIO boot-smoke:
 *
 *   cd apps/studio && bun run test:boot-smoke
 *
 * Starts `sail dev` for apps/studio when STUDIO_URL is not already up,
 * runs test/boot-smoke.e2e.test.ts, then shuts down any server this
 * script started. Never leaves orphan sail processes.
 *
 * Env:
 *   STUDIO_URL   default http://127.0.0.1:7891
 *   STUDIO_ADDR  bind addr when we start sail (default 127.0.0.1:7891)
 *   SAIL_BIN     path to sail binary (default: repo target/debug/sail, else cargo run)
 */
import { spawn } from 'node:child_process'
import http from 'node:http'
import { existsSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const studioRoot = resolve(here, '..')
const repoRoot = resolve(studioRoot, '../..')

const STUDIO_URL = process.env.STUDIO_URL || 'http://127.0.0.1:7891'
const STUDIO_ADDR = process.env.STUDIO_ADDR || '127.0.0.1:7891'

function studioUp(url = STUDIO_URL, timeoutMs = 1500) {
  return new Promise((resolveUp) => {
    try {
      const u = new URL(url)
      const req = http.request(
        {
          hostname: u.hostname,
          port: u.port || 80,
          path: '/',
          method: 'GET',
          timeout: timeoutMs,
        },
        (res) => {
          res.resume()
          resolveUp((res.statusCode ?? 500) < 500)
        },
      )
      req.on('error', () => resolveUp(false))
      req.on('timeout', () => {
        req.destroy()
        resolveUp(false)
      })
      req.end()
    } catch {
      resolveUp(false)
    }
  })
}

async function waitForStudio(maxMs = 180_000) {
  const start = Date.now()
  while (Date.now() - start < maxMs) {
    if (await studioUp()) return true
    await new Promise((r) => setTimeout(r, 500))
  }
  return false
}

function startSail() {
  const debugBin = join(repoRoot, 'target/debug/sail')
  const releaseBin = join(repoRoot, 'target/release/sail')
  const envBin = process.env.SAIL_BIN
  let cmd
  let args
  if (envBin && existsSync(envBin)) {
    cmd = envBin
    args = ['dev', '--project', 'apps/studio', '--addr', STUDIO_ADDR]
  } else if (existsSync(debugBin)) {
    cmd = debugBin
    args = ['dev', '--project', 'apps/studio', '--addr', STUDIO_ADDR]
  } else if (existsSync(releaseBin)) {
    cmd = releaseBin
    args = ['dev', '--project', 'apps/studio', '--addr', STUDIO_ADDR]
  } else {
    cmd = 'cargo'
    args = [
      'run',
      '-q',
      '-p',
      'deck-cli',
      '--bin',
      'sail',
      '--',
      'dev',
      '--project',
      'apps/studio',
      '--addr',
      STUDIO_ADDR,
    ]
  }
  const child = spawn(cmd, args, {
    cwd: repoRoot,
    stdio: ['ignore', 'pipe', 'pipe'],
    env: process.env,
  })
  child.stdout.on('data', (b) => process.stderr.write(`[sail] ${b}`))
  child.stderr.on('data', (b) => process.stderr.write(`[sail] ${b}`))
  return child
}

function killTree(child) {
  if (!child || child.killed || child.exitCode != null) return
  try {
    child.kill('SIGTERM')
  } catch {
    /* ignore */
  }
  setTimeout(() => {
    try {
      if (child.exitCode == null) child.kill('SIGKILL')
    } catch {
      /* ignore */
    }
  }, 2000).unref?.()
}

function runVitest() {
  return new Promise((resolveRun) => {
    const vitestBin = join(repoRoot, 'node_modules/.bin/vitest')
    const cmd = existsSync(vitestBin) ? vitestBin : 'bunx'
    const args = existsSync(vitestBin)
      ? ['run', 'test/boot-smoke.e2e.test.ts']
      : ['vitest', 'run', 'test/boot-smoke.e2e.test.ts']
    const child = spawn(cmd, args, {
      cwd: studioRoot,
      stdio: 'inherit',
      env: { ...process.env, STUDIO_URL },
    })
    child.on('exit', (code, signal) => {
      resolveRun(signal ? 1 : code ?? 1)
    })
    child.on('error', (err) => {
      console.error('[boot-smoke] vitest spawn failed:', err)
      resolveRun(1)
    })
  })
}

let sailChild = null
let startedByUs = false

async function main() {
  const already = await studioUp()
  if (!already) {
    console.error(`[boot-smoke] starting sail dev at ${STUDIO_ADDR} …`)
    sailChild = startSail()
    startedByUs = true
    sailChild.on('exit', (code, signal) => {
      if (startedByUs) {
        console.error(
          `[boot-smoke] sail exited early code=${code} signal=${signal ?? ''}`,
        )
      }
    })
    const ok = await waitForStudio()
    if (!ok) {
      console.error(
        `[boot-smoke] studio never became reachable at ${STUDIO_URL}`,
      )
      killTree(sailChild)
      process.exit(1)
    }
    console.error(`[boot-smoke] studio up at ${STUDIO_URL}`)
  } else {
    console.error(`[boot-smoke] reusing existing studio at ${STUDIO_URL}`)
  }

  const code = await runVitest()
  if (startedByUs) {
    console.error('[boot-smoke] shutting down sail we started')
    killTree(sailChild)
  }
  process.exit(code)
}

for (const sig of ['SIGINT', 'SIGTERM']) {
  process.on(sig, () => {
    if (startedByUs) killTree(sailChild)
    process.exit(130)
  })
}

main().catch((err) => {
  console.error('[boot-smoke] fatal:', err)
  if (startedByUs) killTree(sailChild)
  process.exit(1)
})
