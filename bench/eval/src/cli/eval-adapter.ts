#!/usr/bin/env bun
/**
 * eval-adapter <phase> — RUN CONTRACT v1 evaluation pack entry.
 * stdin: RunManifest JSON
 * stdout: PhaseResult JSON
 * exit 2 = contract error; exit 1 = internal crash; exit 0 = completed (check ok/verdict)
 */

import { checkContractVersion, isPhase, type Phase } from '../contract/types.ts'
import { validateManifest } from '../contract/validate.ts'
import { runPhase } from '../adapters/index.ts'

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = []
  for await (const chunk of process.stdin) {
    chunks.push(typeof chunk === 'string' ? Buffer.from(chunk) : chunk)
  }
  return Buffer.concat(chunks).toString('utf8')
}

async function main(): Promise<void> {
  const phaseArg = process.argv[2]
  if (!phaseArg || phaseArg === '-h' || phaseArg === '--help') {
    console.error('usage: eval-adapter <prepare|gate|render|collect>')
    process.exit(2)
  }
  if (!isPhase(phaseArg)) {
    console.error(JSON.stringify({
      contract_version: 'run-contract/1',
      phase: phaseArg,
      ok: false,
      verdict: 'error',
      error: { code: 'UNKNOWN_PHASE', message: `unknown phase: ${phaseArg}` },
    }))
    process.exit(2)
  }
  const phase = phaseArg as Phase

  let raw: string
  try {
    raw = await readStdin()
  } catch (e) {
    console.error(e)
    process.exit(1)
  }

  let parsed: unknown
  try {
    parsed = JSON.parse(raw || '{}')
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    process.stdout.write(
      JSON.stringify({
        contract_version: 'run-contract/1',
        phase,
        ok: false,
        verdict: 'error',
        wall_ms: 0,
        artifacts: [],
        metrics: {},
        flags: [],
        missing: {},
        detail: {},
        error: { code: 'MALFORMED_STDIN', message: msg },
      }) + '\n',
    )
    process.exit(2)
  }

  const v = validateManifest(parsed)
  if (!v.ok) {
    process.stdout.write(
      JSON.stringify({
        contract_version: 'run-contract/1',
        phase,
        ok: false,
        verdict: 'error',
        wall_ms: 0,
        artifacts: [],
        metrics: {},
        flags: [],
        missing: {},
        detail: {},
        error: { code: 'INVALID_MANIFEST', message: v.error },
      }) + '\n',
    )
    process.exit(2)
  }

  const verErr = checkContractVersion(v.value.contract_version)
  if (verErr) {
    process.stdout.write(
      JSON.stringify({
        contract_version: 'run-contract/1',
        phase,
        ok: false,
        verdict: 'error',
        wall_ms: 0,
        artifacts: [],
        metrics: {},
        flags: [],
        missing: {},
        detail: {},
        error: { code: 'VERSION', message: verErr },
      }) + '\n',
    )
    process.exit(2)
  }

  try {
    const result = await runPhase(phase, { ...v.value, phase })
    process.stdout.write(JSON.stringify(result) + '\n')
    // exit 0 even on ok:false — core reads body for phase policy
    process.exit(0)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error(msg)
    process.stdout.write(
      JSON.stringify({
        contract_version: 'run-contract/1',
        phase,
        ok: false,
        verdict: 'error',
        wall_ms: 0,
        artifacts: [],
        metrics: {},
        flags: ['adapter_crash'],
        missing: {},
        detail: {},
        error: { code: 'ADAPTER_CRASH', message: msg },
      }) + '\n',
    )
    process.exit(1)
  }
}

main()
