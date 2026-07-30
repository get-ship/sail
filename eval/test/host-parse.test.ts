import { describe, expect, it } from 'vitest'
import { parseJsonStdout } from '../src/host.ts'

describe('parseJsonStdout', () => {
  it('parses plain JSON', () => {
    expect(parseJsonStdout('{"ok":true}')).toEqual({ ok: true })
  })

  it('takes the last JSON object when logs precede it', () => {
    const stdout = `info: loading {cache}
warn: path {/tmp/x}
{"ok":true,"count":1}
`
    expect(parseJsonStdout(stdout)).toEqual({ ok: true, count: 1 })
  })

  it('takes trailing array when that is the last value', () => {
    const stdout = `log {start}
[{"rule":"overflow"}]
`
    expect(parseJsonStdout(stdout)).toEqual([{ rule: 'overflow' }])
  })

  it('throws when no JSON present', () => {
    expect(() => parseJsonStdout('no braces here')).toThrow(/no JSON/)
  })
})
