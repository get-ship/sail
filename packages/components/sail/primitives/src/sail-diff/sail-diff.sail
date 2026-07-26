<!--
  <sail-diff> — unified diff renderer. Accepts from/to (LCS diff computed) or a
  raw unified-diff string via value.

  Prop parity
  ----------------------------------------------------------------
  from  (string, reflect)   original text
  to    (string, reflect)   revised text
  value (string, reflect)   raw unified-diff (wins over from/to)
  ----------------------------------------------------------------
  (no events, no slots)
  ARIA: host role=region + aria-label="diff" (controller sets if absent);
        each line role=listitem + aria-label="added|removed|hunk|context: …"

  The controller computes `lines()` as data; the template renders them with
  {#for}/{#match}, one static sx read per line kind.

  LANG-GAP: white-space control is not sx vocabulary — code lines must not
  collapse space runs, so the text rides a <pre> (pre, not the pre-wrap +
  break-word this component actually wants).
-->

<scope>
  props: {
    from = ''
    to = ''
    value = ''
  }
</scope>

<controller>
import { controller, derived } from '@ship/sail-runtime'
import type { Scope } from './sail-diff.sail'

type Kind = 'add' | 'del' | 'ctx' | 'hunk'
interface Line {
  id: number
  key: string
  kind: Kind
  text: string
  sign: string
  label: string
}

function diffKinds(a: string, b: string): Array<{ kind: Kind; text: string }> {
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
  const out: Array<{ kind: Kind; text: string }> = []
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

function parseUnified(value: string): Array<{ kind: Kind; text: string }> {
  return value.split('\n').map((line) => {
    if (line.startsWith('@@')) return { kind: 'hunk' as Kind, text: line }
    if (line.startsWith('+')) return { kind: 'add' as Kind, text: line.slice(1) }
    if (line.startsWith('-')) return { kind: 'del' as Kind, text: line.slice(1) }
    return { kind: 'ctx' as Kind, text: line.startsWith(' ') ? line.slice(1) : line }
  })
}

const SIGNS: Record<Kind, string> = { add: '+', del: '-', hunk: '@', ctx: ' ' }
const LABELS: Record<Kind, string> = { add: 'added', del: 'removed', hunk: 'hunk', ctx: 'context' }

export default controller(({ from, to, value }: Scope) => {
    const lines = derived((): Line[] => {
      const v = value()
      const f = from()
      const t = to()
      const raw = v ? parseUnified(v) : f || t ? diffKinds(f, t) : []
      return raw.map((l, i) => ({
        id: i,
        key: `${i}:${l.kind}`,
        kind: l.kind,
        text: l.text,
        sign: SIGNS[l.kind],
        label: `${LABELS[l.kind]}: ${l.text}`,
      }))
    })

    mount((host) => {
      if (!host.hasAttribute('role')) host.setAttribute('role', 'region')
      if (!host.hasAttribute('aria-label')) host.setAttribute('aria-label', 'diff')
    })

    return { lines }
  })
</controller>

<column role="list" gap="0" sx={style.bg(muted).borderWidth(1).border(border).radius(md).overflow(hidden).fontFamily(mono).fontSize(sm).leading(normal)}>
  <for as="line" of={lines()} key={line.key}>
    <match on={line.kind}>
      <case of={'add'}>
        <row role="listitem" data-line-kind="add" aria-label={line.label} gap="0" sx={style.padX(2).bg(hover-bg)}>
          <small aria-hidden="true" sx={style.w(4).fg(success)}>{line.sign}</small>
          <pre grow sx={style.fontSize(sm)}>{line.text}</pre>
        </row>
      </case><case of={'del'}>
        <row role="listitem" data-line-kind="del" aria-label={line.label} gap="0" sx={style.padX(2).bg(hover-bg)}>
          <small aria-hidden="true" sx={style.w(4).fg(destructive)}>{line.sign}</small>
          <pre grow sx={style.fontSize(sm)}>{line.text}</pre>
        </row>
      </case><case of={'hunk'}>
        <row role="listitem" data-line-kind="hunk" aria-label={line.label} gap="0" sx={style.padX(2).bg(hover-bg)}>
          <small aria-hidden="true" sx={style.w(4).fg(info)}>{line.sign}</small>
          <pre grow sx={style.fontSize(sm).fg(info)}>{line.text}</pre>
        </row>
      </case><default>
        <row role="listitem" data-line-kind="ctx" aria-label={line.label} gap="0" sx={style.padX(2)}>
          <small aria-hidden="true" sx={style.w(4).fg(muted-foreground)}>{line.sign}</small>
          <pre grow sx={style.fontSize(sm)}>{line.text}</pre>
        </row>
    </default></match>
  </for>
</column>
