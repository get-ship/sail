/**
 * Flatten highlight spans into escaped `<span class="tok-*">` HTML
 * for `<code-block>` / `use:syntaxHighlight`.
 */

import { SCOPE_TO_TOK, type SailScope } from './scopes'

export interface TokSpan {
  start: number
  end: number
  scope: SailScope
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function tokClass(scope: SailScope): string {
  return SCOPE_TO_TOK[scope] ?? ''
}

/**
 * Non-overlapping segments. Overlaps pick the shortest covering span
 * (innermost capture). Adjacent same-class runs merge.
 */
export function toTokHtml(src: string, spans: readonly TokSpan[]): string {
  if (!src) return ''
  const cuts = new Set<number>([0, src.length])
  for (const s of spans) {
    const lo = Math.max(0, Math.min(s.start, src.length))
    const hi = Math.max(lo, Math.min(s.end, src.length))
    if (hi > lo) {
      cuts.add(lo)
      cuts.add(hi)
    }
  }
  const points = [...cuts].sort((a, b) => a - b)
  let out = ''
  let runCls = ''
  let runText = ''
  const flush = (): void => {
    if (!runText) return
    if (runCls) out += `<span class="tok-${runCls}">${esc(runText)}</span>`
    else out += esc(runText)
    runText = ''
  }
  for (let i = 0; i < points.length - 1; i++) {
    const start = points[i]!
    const end = points[i + 1]!
    const covering = spans.filter((s) => s.start <= start && s.end >= end && s.end > s.start)
    let cls = ''
    if (covering.length > 0) {
      let best = covering[0]!
      for (const s of covering) {
        if (s.end - s.start < best.end - best.start) best = s
      }
      cls = tokClass(best.scope)
    }
    if (cls !== runCls) {
      flush()
      runCls = cls
    }
    runText += src.slice(start, end)
  }
  flush()
  return out
}
