// E2E test for the ported git view at `views/git/`. Drives hydrate
// against the real template + controller bundle equivalent (we
// dynamic-import the source ts module and pass its default export).
//
// Stress test for: <data live>, multi-derived controller, useFilteredList
// composed in `use:`, <use-kbd> shortcut handlers, tab-driven {#if}
// branches.

import { afterEach, describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { compileAndMount as hydrate } from '../src/test-utils/compile-sui'
import {
  signal,
  type LiveSignal,
  type LiveSource,
  type LiveState,
} from '@ship/sui-runtime'
import gitView from '../../../../views/git/controller'
import type { GitOverview } from '@ship/sdk-client'

const containers: HTMLElement[] = []
const mkContainer = (): HTMLElement => {
  const c = document.createElement('div')
  document.body.appendChild(c)
  containers.push(c)
  return c
}
afterEach(() => {
  for (const c of containers.splice(0)) c.remove()
})
const flush = () => new Promise((r) => setTimeout(r, 0))

const TEMPLATE = readFileSync(
  resolve(__dirname, '../../../../views/git/page.sui'),
  'utf8',
)

const fakeOverview = (over: Partial<GitOverview> = {}): GitOverview => ({
  workspace_id: null,
  branch: 'main',
  head_short: 'abc1234',
  clean: true,
  total_changed: 0,
  staged: [],
  modified: [],
  untracked: [],
  commits: [],
  branches: [],
  ahead: 0,
  behind: 0,
  ...over,
})

const fakeLive = <T>(value: T): LiveSource<T> => ({
  subscribe: (sig) => {
    sig.set({ kind: 'ok', value } as LiveState<T>)
    return () => {}
  },
})

const fakeShip = (over: GitOverview) => ({
  git: { overview: () => fakeLive(over) },
})

describe('views/git — clean tree', () => {
  it('renders the clean-tree hero with branch name', async () => {
    const container = mkContainer()
    const ship = fakeShip(fakeOverview({ branch: 'feat/ship-set', head_short: '61be7ee7' }))
    const handle = hydrate(TEMPLATE, gitView.setup, { container, ship })
    await flush()

    expect(container.querySelector('.git-title')!.textContent).toBe('Git')
    expect(container.querySelector('.git-hero-title')!.textContent).toBe(
      'Working tree is clean.',
    )
    // `branch()` derived rendered into the hero subtitle's <span>
    const sub = container.querySelector('.git-hero-sub')!.textContent ?? ''
    expect(sub).toContain('feat/ship-set')

    handle.dispose()
  })

  it('shows recent commits + branches in the hero columns', async () => {
    const container = mkContainer()
    const ship = fakeShip(
      fakeOverview({
        branch: 'main',
        commits: [
          { sha: 'a1', short_sha: 'a1', subject: 'Initial', author: 'Ada', date: '1d', additions: 10, deletions: 0 },
          { sha: 'b2', short_sha: 'b2', subject: 'Add docs', author: 'Bea', date: '2h', additions: 5, deletions: 1 },
        ],
        branches: [
          { name: 'main', active: true, ahead: 0, behind: 0 },
          { name: 'feat/x', active: false, upstream: 'origin/feat/x', ahead: 2, behind: 0 },
        ],
      }),
    )
    const handle = hydrate(TEMPLATE, gitView.setup, { container, ship })
    await flush()

    const titles = Array.from(container.querySelectorAll('.git-row-title')).map(
      (n) => n.textContent ?? '',
    )
    expect(titles).toContain('Initial')
    expect(titles).toContain('Add docs')
    expect(titles).toContain('main')
    expect(titles).toContain('feat/x')

    handle.dispose()
  })
})

describe('views/git — non-clean tree', () => {
  it('renders staged / modified / untracked file groups', async () => {
    const container = mkContainer()
    const ship = fakeShip(
      fakeOverview({
        clean: false,
        total_changed: 3,
        staged: [{ path: 'a.ts' }],
        modified: [{ path: 'b.ts' }],
        untracked: [{ path: 'c.ts' }],
      }),
    )
    const handle = hydrate(TEMPLATE, gitView.setup, { container, ship })
    await flush()

    const rows = Array.from(
      container.querySelectorAll<HTMLButtonElement>('[data-test="git-file-row"]'),
    )
    // The row is `<span class="git-file-status">glyph</span>{f.path}` — the
    // path is the text node that comes right after the status span, not
    // the full textContent (which would include the glyph and surrounding
    // whitespace from the template's indentation).
    const lines = rows.map((n) => {
      const span = n.querySelector('.git-file-status')
      const after = span?.nextSibling
      return after?.nodeType === 3 ? (after.textContent ?? '').trim() : ''
    })
    expect(lines).toEqual(['a.ts', 'b.ts', 'c.ts'])
    handle.dispose()
  })
})

describe('views/git — tab switching via on.*', () => {
  it('switching to history shows the history panel + commit filter', async () => {
    const container = mkContainer()
    const ship = fakeShip(
      fakeOverview({
        commits: [
          { sha: 'a', short_sha: 'a', subject: 'feat: add', author: 'Ada', date: '1h', additions: 1, deletions: 0 },
          { sha: 'b', short_sha: 'b', subject: 'fix: bug', author: 'Bea', date: '2h', additions: 1, deletions: 1 },
        ],
      }),
    )
    const handle = hydrate(TEMPLATE, gitView.setup, { container, ship })
    await flush()

    // No history panel yet — tab is 'changes'
    expect(container.querySelector('.git-history')).toBeNull()

    // <use-kbd> wires '2' to showHistory action — dispatch keydown.
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '2' }))
    await flush()

    expect(container.querySelector('.git-history')).not.toBeNull()
    // Both commits visible; count text "2 of 2".
    expect(
      container.querySelector('.git-history-count')!.textContent,
    ).toContain('2 of 2')

    // Type into the search; useFilteredList narrows.
    const input = container.querySelector('.git-search') as HTMLInputElement
    input.value = 'fix'
    input.dispatchEvent(new Event('input', { bubbles: true }))
    await flush()
    expect(
      container.querySelector('.git-history-count')!.textContent,
    ).toContain('1 of 2')

    handle.dispose()
  })

  it('keyboard shortcut "3" switches to branches', async () => {
    const container = mkContainer()
    const ship = fakeShip(
      fakeOverview({
        branches: [
          { name: 'main', active: true, ahead: 0, behind: 0 },
          { name: 'feat/y', active: false, ahead: 1, behind: 0 },
        ],
      }),
    )
    const handle = hydrate(TEMPLATE, gitView.setup, { container, ship })
    await flush()

    expect(container.querySelector('.git-branches')).toBeNull()
    document.dispatchEvent(new KeyboardEvent('keydown', { key: '3' }))
    await flush()
    expect(container.querySelector('.git-branches')).not.toBeNull()
    handle.dispose()
  })
})
