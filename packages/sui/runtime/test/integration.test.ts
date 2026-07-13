// Integration test: compile a real .sui file via the Rust `sui-compiler`
// binary, load the produced JS in jsdom, and exercise the resulting
// custom element. This is the test that proves the compiler+runtime
// contract end-to-end. Substring assertions in the compiler's emit
// tests can pass while the actual JS is broken; this can't.
//
// Fixtures live in test/fixtures/ — one .sui per
// component under test. Components compile with shadow DOM (the CLI
// default for component mode), so DOM queries go via el.shadowRoot.

import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, writeFileSync, existsSync, rmSync, copyFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { describe, expect, it, beforeAll } from 'vitest'

const REPO_ROOT = resolve(__dirname, '../../../..')
const SUI_COMPILER_BIN = resolve(REPO_ROOT, 'target/debug/sui-compiler')
const FIXTURES = resolve(__dirname, 'fixtures')
// Compile output sits inside the project tree so vitest's resolver
// can find the dynamic-imported file. Cleared per test run.
const COMPILE_OUT = resolve(__dirname, '.compiled')

function ensureBinary() {
  if (existsSync(SUI_COMPILER_BIN)) return
  execFileSync('cargo', ['build', '--bin', 'sui-compiler', '--quiet'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
}

/** Compile a fixture and return the path to the .mjs output. */
function compileFixture(file: string): string {
  ensureBinary()
  mkdirSync(COMPILE_OUT, { recursive: true })
  execFileSync(
    SUI_COMPILER_BIN,
    ['build', '--mode', 'component', join(FIXTURES, file), '--out', COMPILE_OUT],
    { stdio: 'pipe' },
  )
  // Output filename = input stem (e.g. ship-set-counter.sui → ship-set-counter.js).
  // The component tag registered by the runtime is also the stem.
  const stem = file.replace(/\.sui$/, '')
  const jsPath = join(COMPILE_OUT, `${stem}.js`)

  // The compiled JS imports its controller as the virtual
  // `./<stem>.sui.controller.ts`, resolved against the module's own directory.
  // The harness serves that module by extracting the `<controller>` block from
  // the sibling `.sui` — so the source has to sit next to the output.
  const suiDst = join(COMPILE_OUT, file)
  if (!existsSync(suiDst)) copyFileSync(join(FIXTURES, file), suiDst)

  // Copy as .mjs so vitest treats it as a module unambiguously.
  const js = readFileSync(jsPath, 'utf8')
  const mjsPath = join(COMPILE_OUT, `${stem}.mjs`)
  writeFileSync(mjsPath, js)
  return mjsPath
}

/** Compile, import (registers the custom element), and return the tag name. */
async function loadCompiled(file: string): Promise<string> {
  const path = compileFixture(file)
  // Cache-bust per test run with a query param; vitest dedupes by URL.
  await import(`${path}?t=${Date.now()}`)
  // Tag name = stem (all fixture names already include the required hyphen).
  return file.replace(/\.sui$/, '')
}

/** Query helper — light DOM (the default) renders into the host itself;
 *  shadow components (the `shadow: true` opt-in) render into the shadow root. */
function q(el: Element, selector: string): Element | null {
  return (el.shadowRoot ?? el).querySelector(selector)
}

function qAll(el: Element, selector: string): NodeListOf<Element> {
  return (el.shadowRoot ?? el).querySelectorAll(selector)
}

describe('integration: real compile → run', () => {
  beforeAll(() => {
    ensureBinary()
    // Clean stale compile output so each run is deterministic.
    if (existsSync(COMPILE_OUT)) rmSync(COMPILE_OUT, { recursive: true })
  }, 120_000)

  it('counter compiles and ticks', async () => {
    const tag = await loadCompiled('ship-set-counter.sui')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const count = q(el, '[data-testid="count"]')!
    const doubled = q(el, '[data-testid="doubled"]')!
    const inc = q(el, '[data-testid="inc"]') as HTMLButtonElement
    const reset = q(el, '[data-testid="reset"]') as HTMLButtonElement

    expect(count.textContent).toBe('0')
    expect(doubled.textContent).toBe('0')

    inc.click()
    inc.click()
    inc.click()
    expect(count.textContent).toBe('3')
    expect(doubled.textContent).toBe('6')

    reset.click()
    expect(count.textContent).toBe('0')
    expect(doubled.textContent).toBe('0')

    el.remove()
  }, 60_000)

  it('search filters with derived + if/else', async () => {
    const tag = await loadCompiled('ship-set-search.sui')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const query = q(el, '[data-testid="query"]') as HTMLInputElement
    const caseBox = q(el, '[data-testid="case"]') as HTMLInputElement

    // Initial: no query → all 6 items.
    expect(qAll(el, '[data-testid="result"]').length).toBe(6)
    expect(q(el, '[data-testid="empty"]')).toBeNull()

    // Filter: "b" → banana, blueberry.
    query.value = 'b'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    const results = Array.from(qAll(el, '[data-testid="result"]'))
    expect(results.map((r) => r.textContent)).toEqual(['banana', 'blueberry'])

    // Empty branch: query that matches nothing.
    query.value = 'zzz'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    expect(qAll(el, '[data-testid="result"]').length).toBe(0)
    const empty = q(el, '[data-testid="empty"]')!
    expect(empty.textContent).toContain('zzz')

    // Case-sensitivity flips a derived dep.
    // ITEMS = ['apple', 'apricot', 'banana', 'blueberry', 'cherry', 'grape']
    // 'AP' case-insensitive → apple + apricot + grape (all contain 'ap').
    query.value = 'AP'
    query.dispatchEvent(new Event('input', { bubbles: true }))
    expect(qAll(el, '[data-testid="result"]').length).toBe(3) // apple + apricot + grape
    caseBox.click()
    expect(qAll(el, '[data-testid="result"]').length).toBe(0) // case-sensitive: none contain 'AP'

    el.remove()
  }, 60_000)

  it('note-display props + expand toggle + list', async () => {
    const tag = await loadCompiled('note-display.sui')
    const el = document.createElement(tag)
    el.setAttribute('title', 'My Note')
    el.setAttribute('body', 'Hello world')
    el.setAttribute('tags', '["a","b"]')
    document.body.appendChild(el)

    expect(q(el, '[data-testid="title"]')!.textContent).toBe('My Note')
    // Initially collapsed.
    expect(q(el, '[data-testid="body"]')).toBeNull()
    expect(q(el, '[data-testid="tags"]')).toBeNull()

    // Expand.
    ;(q(el, '[data-testid="toggle"]') as HTMLButtonElement).click()
    const body = q(el, '[data-testid="body"]')!
    expect(body.textContent).toBe('Hello world')
    const tags = q(el, '[data-testid="tags"]')!
    expect(tags.querySelectorAll('li').length).toBe(2)

    // Title prop change flows through.
    el.setAttribute('title', 'Renamed')
    expect(q(el, '[data-testid="title"]')!.textContent).toBe('Renamed')

    el.remove()
  }, 60_000)

  it('modal: visibility, prop-driven open/close, backdrop close', async () => {
    const tag = await loadCompiled('ship-set-modal.sui')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    // open=false (default) → data-open="false".
    const root = q(el, '.modal-root')!
    expect(root.getAttribute('data-open')).toBe('false')

    // Open: attribute flows through prop signal → bindAttr re-evaluates.
    el.setAttribute('open', '')
    expect(root.getAttribute('data-open')).toBe('true')

    // Click backdrop closes.
    const backdrop = q(el, '[data-testid="backdrop"]') as HTMLElement
    backdrop.click()
    expect(root.getAttribute('data-open')).toBe('false')

    el.remove()
  }, 60_000)

  it('todo list adds, toggles, removes', async () => {
    const tag = await loadCompiled('ship-set-todo.sui')
    const el = document.createElement(tag)
    document.body.appendChild(el)

    const draft = q(el, '[data-testid="draft"]') as HTMLInputElement
    const add = q(el, '[data-testid="add"]') as HTMLButtonElement
    const list = q(el, '[data-testid="list"]')!
    const count = q(el, '[data-testid="count"]')!

    // Initial render: 2 items.
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(2)
    expect(count.textContent).toBe('2 item(s)')

    // Add one.
    draft.value = 'feed cat'
    draft.dispatchEvent(new Event('input', { bubbles: true }))
    add.click()
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(3)
    expect(count.textContent).toBe('3 item(s)')

    // Toggle the first.
    const firstToggle = list.querySelectorAll('[data-testid="toggle"]')[0] as HTMLButtonElement
    expect(firstToggle.textContent).toBe('O')
    firstToggle.click()
    expect(firstToggle.textContent).toBe('X')

    // Remove the second.
    const removeBtns = list.querySelectorAll('[data-testid="remove"]')
    ;(removeBtns[1] as HTMLButtonElement).click()
    expect(list.querySelectorAll('[data-testid="item"]').length).toBe(2)
    expect(count.textContent).toBe('2 item(s)')

    el.remove()
  }, 60_000)
})
