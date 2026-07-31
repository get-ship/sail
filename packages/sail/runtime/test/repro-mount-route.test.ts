// Contract: ambient `mount` vs host `mount` collision at compile time.
//
// A route-level page controller that imports `mount` from `@ship/sail-runtime`
// shadows the ambient lifecycle hook with the F1 host route-mounting entry.
// That mis-call throws at runtime inside `seedRouteParams`. The compiler
// refuses it (`sail-mount-ambient-shadow`) so authors get a hard error, not
// a silent page failure.
//
// Ambient (no import) mount+cleanup: callback receives the host; cleanup
// runs on dispose (no interval survives unmount/nav).
import { execFileSync } from 'node:child_process'
import { mkdirSync, readFileSync, writeFileSync, existsSync, copyFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { describe, expect, it, beforeAll, vi } from 'vitest'
import { mount, type RoutePayload } from '../src/mount'
import { route } from '../src/route'

const REPO_ROOT = resolve(__dirname, '../../../..')
const SUI_COMPILER_BIN = resolve(REPO_ROOT, 'target/debug/sail-compiler')
const FIXTURES = resolve(__dirname, 'fixtures')
const COMPILE_OUT = resolve(__dirname, '.compiled-repro')

function ensureBinary(): void {
  if (existsSync(SUI_COMPILER_BIN)) return
  execFileSync('cargo', ['build', '--bin', 'sail-compiler', '--quiet'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
}

/** Compile a fixture (given as a repo-relative path under fixtures/) and
 *  return the emitted .mjs path, or throw the compiler's stderr on failure. */
function compileFixture(relFile: string): string {
  ensureBinary()
  const outDir = join(COMPILE_OUT, ...relFile.split('/').slice(0, -1))
  mkdirSync(outDir, { recursive: true })
  execFileSync(SUI_COMPILER_BIN, ['build', join(FIXTURES, relFile), '--out', outDir], {
    stdio: 'pipe',
  })
  const stem = relFile.replace(/\.sail$/, '').split('/').pop()!
  const jsPath = join(outDir, `${stem}.js`)
  const sailDst = join(outDir, `${stem}.sail`)
  if (!existsSync(sailDst)) copyFileSync(join(FIXTURES, relFile), sailDst)
  const js = readFileSync(jsPath, 'utf8')
  const mjsPath = join(outDir, `${stem}.mjs`)
  writeFileSync(mjsPath, js)
  return mjsPath
}

describe('ambient mount() in a route-level page controller', () => {
  beforeAll(() => {
    ensureBinary()
  }, 120_000)

  it('RED→refused: explicitly importing `mount` from @ship/sail-runtime is a compile-time error, not a runtime pageerror', () => {
    expect(() => compileFixture('repro-route-bad/page.sail')).toThrowError(
      /sail-mount-ambient-shadow/,
    )
    // The diagnostic names both `mount`s and says what to do — not just "no".
    try {
      compileFixture('repro-route-bad/page.sail')
      expect.unreachable()
    } catch (err) {
      const message = String((err as { stderr?: Buffer }).stderr ?? err)
      expect(message).toContain('view-mounting entry point')
      expect(message).toContain('ambient lifecycle hook')
    }
  }, 60_000)

  it('GREEN: ambient mount() (no import) fires with the host element and cleans up on dispose', async () => {
    const path = compileFixture('repro-route/page.sail')
    const mod = await import(`${path}?t=${Date.now()}`)

    const target = document.createElement('div')
    document.body.appendChild(target)

    // The same real, live RouteSignals a router hands a page — proves the
    // fix works with actual route plumbing present, not just its absence.
    const routeSignals = route({ url: new URL('http://localhost/word') })

    const payload: RoutePayload = {
      name: 'repro-route-page',
      html: '',
      css: '',
      scope: {},
      moduleUrl: '',
      params: {},
      clients: [],
    } as unknown as RoutePayload

    const handle = await mount(target, payload, {
      module: mod as never,
      route: routeSignals,
      navigate: () => {},
      cssScope: 'adjacent',
    })

    const div = target.querySelector('[data-testid="word"]') as HTMLElement
    expect(div).toBeTruthy()
    // The ambient callback ran with the host element (mount.ts's `target`).
    expect(target.getAttribute('data-mounted')).toBe('true')

    const before = div.textContent
    await vi.waitFor(() => {
      expect(div.textContent).not.toBe(before)
    })

    // Dispose = "unmount across a nav." Cleanup must run, and the interval
    // must not keep firing afterward (the leak the mission asks us to guard).
    handle.dispose()
    expect(target.getAttribute('data-cleaned-up')).toBe('true')
    const afterDispose = div.textContent
    await new Promise((r) => setTimeout(r, 40))
    expect(div.textContent).toBe(afterDispose)
  }, 60_000)
})
