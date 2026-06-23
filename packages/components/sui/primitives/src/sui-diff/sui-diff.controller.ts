import { controller, bindHtml } from '@ship/sui-runtime'
import type { Scope } from './sui-diff.sui'

type Line = { kind: 'add' | 'del' | 'ctx' | 'hunk'; text: string }

function escape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function diffLines(a: string, b: string): Line[] {
  const A = a.split('\n')
  const B = b.split('\n')
  const n = A.length
  const m = B.length
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0))
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i]![j] = A[i] === B[j] ? dp[i + 1]![j + 1]! + 1 : Math.max(dp[i + 1]![j]!, dp[i]![j + 1]!)
    }
  }
  const out: Line[] = []
  let i = 0
  let j = 0
  while (i < n && j < m) {
    if (A[i] === B[j]) { out.push({ kind: 'ctx', text: A[i]! }); i++; j++ }
    else if (dp[i + 1]![j]! >= dp[i]![j + 1]!) { out.push({ kind: 'del', text: A[i]! }); i++ }
    else { out.push({ kind: 'add', text: B[j]! }); j++ }
  }
  while (i < n) out.push({ kind: 'del', text: A[i++]! })
  while (j < m) out.push({ kind: 'add', text: B[j++]! })
  return out
}

function parseUnified(value: string): Line[] {
  return value.split('\n').map((line) => {
    if (line.startsWith('@@')) return { kind: 'hunk', text: line }
    if (line.startsWith('+')) return { kind: 'add', text: line.slice(1) }
    if (line.startsWith('-')) return { kind: 'del', text: line.slice(1) }
    return { kind: 'ctx', text: line.startsWith(' ') ? line.slice(1) : line }
  })
}

function renderLines(lines: Line[]): string {
  return lines
    .map((l) => {
      const sign = l.kind === 'add' ? '+' : l.kind === 'del' ? '-' : l.kind === 'hunk' ? '@' : ' '
      const kindLabel = l.kind === 'add' ? 'added' : l.kind === 'del' ? 'removed' : l.kind === 'hunk' ? 'hunk' : 'context'
      return `<div class="line ${l.kind}" part="line" data-line-kind="${l.kind}" role="listitem" aria-label="${kindLabel}: ${escape(l.text)}"><span class="sign" aria-hidden="true">${sign}</span><span>${escape(l.text)}</span></div>`
    })
    .join('')
}

export default controller<Scope>({
  mount(state, host) {

    if (!host.hasAttribute('role')) host.setAttribute('role', 'region')
    if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'diff')

    const el = state.$signals.contentEl() as HTMLElement | null
    if (!el) return () => {}

    return bindHtml(el, () => {
      const value = state.$signals.value()
      const from = state.$signals.from()
      const to = state.$signals.to()
      let lines: Line[]
      if (value) {
        lines = parseUnified(value)
      } else if (from || to) {
        lines = diffLines(from, to)
      } else {
        lines = []
      }
      return renderLines(lines)
    })
  },
})
