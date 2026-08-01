/**
 * collect phase — post-run metrics + event pointers. Blobs stay on disk.
 */

import { existsSync, readFileSync, statSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import {
  CONTRACT_VERSION,
  type PhaseResult,
  type RunManifest,
  emptyPhaseResult,
} from '../contract/types.ts'
import { fileSha256 } from '../config/hash.ts'

function readEvents(path: string): { lines: number; sessionId: string | null; toolCalls: number } {
  if (!existsSync(path)) {
    return { lines: 0, sessionId: null, toolCalls: 0 }
  }
  const text = readFileSync(path, 'utf8')
  let sessionId: string | null = null
  let toolCalls = 0
  let lines = 0
  for (const line of text.split('\n')) {
    if (!line.trim()) continue
    lines++
    try {
      const row = JSON.parse(line) as { event?: Record<string, unknown> }
      const ev = row.event ?? {}
      sessionId = sessionId ?? (ev.sessionId as string | undefined) ?? null
      const name = String(ev.hookEventName ?? ev.hook_event_name ?? '').toLowerCase()
      if (name.includes('tool') || ev.toolName) toolCalls++
    } catch {
      /* count as line anyway */
    }
  }
  return { lines, sessionId, toolCalls }
}

function tryGrokTrace(sessionId: string | null): {
  usage: Record<string, unknown> | null
  reason?: string
} {
  if (!sessionId) return { usage: null, reason: 'no_session_id' }
  const which = spawnSync('which', ['grok'], { encoding: 'utf8' })
  if (which.status !== 0) return { usage: null, reason: 'grok_not_on_path' }
  const r = spawnSync('grok', ['trace', '--local', '--json', sessionId], {
    encoding: 'utf8',
    timeout: 60_000,
  })
  if (r.status !== 0) {
    return { usage: null, reason: `grok_trace_exit_${r.status}` }
  }
  try {
    const parsed = JSON.parse(r.stdout) as Record<string, unknown>
    return { usage: parsed }
  } catch {
    return { usage: null, reason: 'grok_trace_unparseable' }
  }
}

export async function collectPhase(manifest: RunManifest): Promise<PhaseResult> {
  const t0 = Date.now()
  const result = emptyPhaseResult('collect')
  result.contract_version = CONTRACT_VERSION

  const eventsRel = manifest.paths.events
  const eventsPath = join(manifest.run_dir, eventsRel)

  // Ensure events file exists (empty is ok — explicit)
  if (!existsSync(eventsPath)) {
    writeFileSync(eventsPath, '')
  }

  const ev = readEvents(eventsPath)
  const trace = tryGrokTrace(ev.sessionId)

  const flags: string[] = []
  const missing: Record<string, string> = {}
  let tokensIn: number | null = null
  let tokensOut: number | null = null

  if (trace.usage) {
    const u = trace.usage as {
      contextTokensUsed?: number
      tokensIn?: number
      tokensOut?: number
      input_tokens?: number
      output_tokens?: number
      signals?: { contextTokensUsed?: number }
    }
    tokensIn =
      u.tokensIn ??
      u.input_tokens ??
      u.contextTokensUsed ??
      u.signals?.contextTokensUsed ??
      null
    tokensOut = u.tokensOut ?? u.output_tokens ?? null
  } else {
    flags.push('missing_usage')
    missing.tokens_in = trace.reason ?? 'usage_unavailable'
    missing.tokens_out = trace.reason ?? 'usage_unavailable'
  }

  if (tokensIn == null) {
    flags.push('missing_tokens_in')
    missing.tokens_in = missing.tokens_in ?? 'null'
  }
  if (tokensOut == null) {
    flags.push('missing_tokens_out')
    missing.tokens_out = missing.tokens_out ?? 'null'
  }
  if (!ev.sessionId) {
    flags.push('missing_session_id')
    missing.session_id = 'not_in_events'
  }

  // turns not collected in v1 — explicit NULL + named reason (never bare NULL)
  const turns: number | null = null
  flags.push('missing_turns')
  missing.turns = 'not_collected_v1'

  const eventsStat = existsSync(eventsPath) ? statSync(eventsPath) : null
  const collectBody = {
    session_id: ev.sessionId,
    events_rel: eventsRel,
    events_lines: ev.lines,
    tool_calls: ev.toolCalls,
    tokens_in: tokensIn,
    tokens_out: tokensOut,
    turns,
    usage: trace.usage,
    source_rel: manifest.paths.source,
    collected_at: new Date().toISOString(),
  }
  writeFileSync(join(manifest.run_dir, 'collect.json'), JSON.stringify(collectBody, null, 2) + '\n')

  result.ok = true
  result.verdict = 'pass'
  result.flags = flags
  result.missing = missing
  result.metrics = {
    tokens_in: tokensIn,
    tokens_out: tokensOut,
    turns,
    tool_calls: ev.toolCalls,
    events_lines: ev.lines,
    session_id: ev.sessionId,
  }
  result.detail = collectBody
  result.artifacts = [
    {
      name: 'events.jsonl',
      rel_path: eventsRel,
      blocking: false,
      status: eventsStat && eventsStat.size > 0 ? 'present' : 'degraded',
      bytes: eventsStat?.size ?? 0,
      sha256: eventsStat ? fileSha256(readFileSync(eventsPath)) : null,
      reason: eventsStat && eventsStat.size > 0 ? null : 'empty_events',
    },
    {
      name: 'collect.json',
      rel_path: 'collect.json',
      blocking: false,
      status: 'present',
    },
  ]
  result.wall_ms = Date.now() - t0
  return result
}
