import { describe, it, expect } from 'vitest'
import { resolveLayoutChain, type LayoutShape } from '../src/layouts'

const L = (pathPrefix: string, tag: string): LayoutShape => ({ pathPrefix, tag })

describe('resolveLayoutChain', () => {
  it('returns [] when no layouts are registered', () => {
    expect(resolveLayoutChain('/studio/themes', [])).toEqual([])
  })

  it('matches a single prefix layout', () => {
    const layouts = [L('/studio', 'sui-studio-chrome')]
    expect(resolveLayoutChain('/studio/themes', layouts)).toEqual(layouts)
  })

  it('orders nested matches deepest-first (innermost chrome at index 0)', () => {
    const studio = L('/studio', 'sui-studio-chrome')
    const agents = L('/studio/agents', 'sui-agents-chrome')
    // Registration order intentionally shallow-first to prove the sort.
    const chain = resolveLayoutChain('/studio/agents/42', [studio, agents])
    expect(chain.map((l) => l.tag)).toEqual(['sui-agents-chrome', 'sui-studio-chrome'])
  })

  it("a '/' layout matches every path", () => {
    const root = L('/', 'sui-root-chrome')
    expect(resolveLayoutChain('/anything/here', [root])).toEqual([root])
  })

  it('excludes prefixes that do not match the pathname', () => {
    expect(resolveLayoutChain('/home', [L('/studio', 'x')])).toEqual([])
  })

  it('matches an exact pathname (prefix === pathname)', () => {
    expect(resolveLayoutChain('/studio', [L('/studio', 'x')])).toHaveLength(1)
  })

  it('respects segment boundaries (/studio does not match /studiox)', () => {
    expect(resolveLayoutChain('/studiox', [L('/studio', 'x')])).toEqual([])
  })

  it('accepts a URL object', () => {
    const chain = resolveLayoutChain(new URL('http://localhost/studio/themes'), [L('/studio', 'x')])
    expect(chain).toHaveLength(1)
  })

  it('strips query and hash before matching', () => {
    expect(resolveLayoutChain('/studio/themes?q=1#frag', [L('/studio', 'x')])).toHaveLength(1)
  })

  it('skips empty or malformed prefixes', () => {
    const valid = L('/studio', 'ok')
    // @ts-expect-error — exercising the runtime guard against bad entries
    const chain = resolveLayoutChain('/studio', [L('', 'bad'), null, valid])
    expect(chain).toEqual([valid])
  })
})
