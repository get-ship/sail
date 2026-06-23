// #164 slice 1 — real-browser SES smoke. The jsdom blocker (#133 slice 5) was
// jsdom's non-standard ArrayBuffer tripping SES lockdown(). This page runs the
// SAME confinement path in a REAL browser engine and reports the result on
// `window.__sesSmoke`, which the playwright harness reads back. If lockdown +
// harden + a confined eval all succeed here, the real-browser SES verification
// path #165 depends on is proven.

// Relative import — the harness isn't a published consumer; this just reaches
// the SES runner to exercise it in a real browser.
import { confineController, lockdownRealm } from '../../host/src/index.ts'

interface SmokeResult {
  lockdown: 'ok' | string
  harden: 'ok' | string
  confined: unknown
  ambientDenied: boolean
}

function run(): SmokeResult {
  const result: SmokeResult = { lockdown: '', harden: '', confined: null, ambientDenied: false }

  try {
    lockdownRealm()
    result.lockdown = 'ok'
  } catch (e) {
    result.lockdown = e instanceof Error ? e.message : String(e)
    return result
  }

  try {
    // harden is only functional after lockdown — exercise it.
    const frozen = harden({ a: 1 }) as { a: number }
    result.harden = Object.isFrozen(frozen) ? 'ok' : 'not frozen'
  } catch (e) {
    result.harden = e instanceof Error ? e.message : String(e)
  }

  // A confined controller: endowed `signal`, ambient `fetch` must be denied.
  const source = `({
    setup: () => {
      const s = signal(7)
      return { value: s(), fetch: typeof fetch, win: typeof window }
    }
  })`
  try {
    const ctrl = confineController<{ setup: () => Record<string, unknown> }>(source, {
      signal: (v: number) => () => v,
    })
    const out = ctrl.setup()
    result.confined = out
    result.ambientDenied = out.fetch === 'undefined' && out.win === 'undefined'
  } catch (e) {
    result.confined = e instanceof Error ? e.message : String(e)
  }

  return result
}

;(window as unknown as { __sesSmoke: SmokeResult }).__sesSmoke = run()
