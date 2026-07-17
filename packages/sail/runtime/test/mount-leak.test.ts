import { describe, expect, it } from 'vitest'

import { mount, type ViewPayload } from '../src/mount'
import { onCleanup, mount as ambientMount } from '../src/effect-scope'

// Regression for the navigation OOM: a subscription opened inside a
// controller's ambient mount() callback (the common `live()`/`data()`
// shape) must be torn down when the view is disposed. Before the fix,
// the (then section-based) mount callback ran AFTER setup's withScope
// closed (CURRENT_SCOPE = null), so onCleanup() was a no-op and the
// subscription leaked one open handle per navigation — the
// client-process OOM. `runMounts` now runs mount() callbacks INSIDE
// the same scope setup ran in (see `mount.ts` step 8), so onCleanup
// calls made from a mount() callback still enroll correctly.
describe('mount — live-subscription teardown (navigation-OOM regression)', () => {
  // A minimal payload: empty markup/scope so mount() runs setup → ambient
  // mount() callbacks → handle, with no SSE channel. opts.module bypasses
  // the moduleUrl import.
  const minimalPayload = () =>
    ({ name: 't', html: '', css: '', scope: {}, moduleUrl: '', params: {} } as unknown as ViewPayload)

  it('disposes a cleanup registered inside the controller mount() callback', async () => {
    let cleaned = 0
    const mod = {
      default: {
        setup: () => {
          // This is exactly what live()/data() do inside a controller mount().
          ambientMount(() => {
            onCleanup(() => {
              cleaned++
            })
          })
          return {}
        },
      },
    }

    const target = document.createElement('div')
    const handle = await mount(target, minimalPayload(), { module: mod })

    expect(cleaned).toBe(0) // not disposed yet
    handle.dispose()
    expect(cleaned).toBe(1) // the mount()-callback's onCleanup ran on dispose
  })
})
