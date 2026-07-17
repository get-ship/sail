#!/usr/bin/env node
// Thin shim — published as the `sail-eval` binary. Loads the
// compiled CLI when present; otherwise re-execs Node with TypeScript
// stripping turned on so the source `.ts` works in dev.

import { existsSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawn } from 'node:child_process'

const here = dirname(fileURLToPath(import.meta.url))
const compiled = resolve(here, '..', 'dist', 'cli.js')
const source = resolve(here, '..', 'src', 'cli.ts')

if (existsSync(compiled)) {
  const mod = await import(compiled)
  const code = await mod.main(process.argv.slice(2))
  process.exit(code)
}

// Dev path — Node strips types if invoked with the right flag, but
// must be re-exec'd because the flag is parsed before module init.
if (process.env['SHIP_SET_EVAL_REEXEC'] === '1') {
  const mod = await import(source)
  const code = await mod.main(process.argv.slice(2))
  process.exit(code)
}

const child = spawn(
  process.execPath,
  [
    '--experimental-strip-types',
    '--no-warnings=ExperimentalWarning',
    fileURLToPath(import.meta.url),
    ...process.argv.slice(2),
  ],
  { stdio: 'inherit', env: { ...process.env, SHIP_SET_EVAL_REEXEC: '1' } },
)
child.on('exit', (code) => process.exit(code ?? 0))
