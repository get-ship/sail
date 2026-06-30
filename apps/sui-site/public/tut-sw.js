// SUI tutorial mock-daemon service worker.
//
// Intercepts iframe requests under /tutorial/ to:
//   - Stream <data live> sources via SSE (`/_deck/data/<source>`)
//   - Accept POST mutations (`/_deck/api/<path>`) and push the new
//     envelope to every active SSE subscriber of the matching source
//   - Return static REST payloads (`GET /_deck/api/<path>`)
//
// Tutorial scenarios are tiny JS modules the parent posts to the SW
// after registration. Each scenario describes a per-source timeline
// with an optional mutation handler:
//
//   {
//     '/users': {
//       initial: { kind: 'ok', value: { items: [...] } },
//       frames: [
//         { afterMs: 1500, payload: { kind: 'ok', value: {...} } },
//       ],
//       // v2: optional mutation handlers, keyed by POST path. Receives
//       // the current envelope's `value` + the request body, returns
//       // the new value. SW wraps in { kind: 'ok', value } and pushes
//       // to every active SSE controller.
//       mutations: {
//         '/users/add': (value, body) => ({ items: [...value.items, body] }),
//       },
//     },
//   }
//
// The SW is plain JS — no bundler, no esm.sh dependency. If route
// growth ever warrants Hono's middleware, that's a one-off swap;
// the scenario contract above stays stable.

const VERSION = 'sui-tutorial-sw-v2'

// Per-scenario timeline keyed by source URL (e.g. '/users/list').
// Each value: { initial, frames?, mutations? }
const scenarios = new Map()

// Per-source live state and active SSE controllers, so a POST mutation
// can compute next state + push to every open subscriber.
//   liveState:        sourceUrl → current envelope payload
//   liveControllers:  sourceUrl → Set<ReadableStreamDefaultController>
const liveState = new Map()
const liveControllers = new Map()

self.addEventListener('install', () => {
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim())
})

self.addEventListener('message', (event) => {
  const data = event.data
  if (!data || typeof data !== 'object') return

  if (data.kind === 'load-scenario') {
    scenarios.clear()
    liveState.clear()
    // Close any existing controllers so the next page-render
    // re-subscribes against the fresh scenario.
    for (const set of liveControllers.values()) {
      for (const ctrl of set) {
        try { ctrl.close() } catch { /* already closed */ }
      }
    }
    liveControllers.clear()

    for (const [sourceUrl, timeline] of Object.entries(data.scenario)) {
      scenarios.set(sourceUrl, timeline)
      if (timeline && typeof timeline === 'object' && 'initial' in timeline) {
        liveState.set(sourceUrl, timeline.initial)
      }
    }
    event.source?.postMessage({ kind: 'scenario-loaded', count: scenarios.size })
    return
  }

  if (data.kind === 'ping') {
    event.source?.postMessage({ kind: 'pong', version: VERSION })
  }
})

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url)
  if (!url.pathname.startsWith('/_deck/')) return

  if (url.pathname.startsWith('/_deck/data/')) {
    const sourceUrl = url.pathname.replace(/^\/_deck\/data/, '')
    const timeline = scenarios.get(sourceUrl)
    if (!timeline) {
      event.respondWith(new Response(`no scenario for ${sourceUrl}`, { status: 404 }))
      return
    }
    event.respondWith(streamScenario(sourceUrl, timeline))
    return
  }

  if (url.pathname.startsWith('/_deck/api/')) {
    if (event.request.method === 'POST' || event.request.method === 'PUT') {
      event.respondWith(handleMutation(url.pathname, event.request))
    } else {
      // GET — static payload from scenario if declared as an `'api:/<path>'` key.
      const apiKey = 'api:' + url.pathname.replace(/^\/_deck\/api/, '')
      const payload = scenarios.get(apiKey)
      if (payload != null) {
        event.respondWith(new Response(JSON.stringify(payload), {
          status: 200,
          headers: { 'content-type': 'application/json' },
        }))
      } else {
        event.respondWith(new Response('null', { status: 404 }))
      }
    }
    return
  }

  // Legacy morph push endpoint. v2 prefers POST /_deck/api/<path>
  // with a mutation handler, since that flows cleanly through the
  // existing SSE channel. Keep this responding 200 so old callers
  // (mcp__ship__patch_view shape) don't error.
  if (url.pathname.startsWith('/_deck/morph/')) {
    event.respondWith(new Response('{"ok":true}', {
      status: 200,
      headers: { 'content-type': 'application/json' },
    }))
    return
  }
})

/**
 * Open an SSE stream that:
 *  1. Sends the current liveState (so re-subscribers see the latest).
 *  2. Plays back the scenario's `frames` timeline (one-shot, on the
 *     first subscribe — frames are not replayed for re-subscribers).
 *  3. Registers the controller so mutations can push to it.
 */
function streamScenario(sourceUrl, timeline) {
  const stream = new ReadableStream({
    start(controller) {
      registerController(sourceUrl, controller)

      const enc = new TextEncoder()
      const send = (payload) => {
        try { controller.enqueue(enc.encode(`data: ${JSON.stringify(payload)}\n\n`)) }
        catch { /* stream closed */ }
      }

      // Initial state — current liveState (which started as `initial`).
      send(liveState.get(sourceUrl))

      const frames = timeline.frames || []
      const firstSubscribe = !timeline.__playedFrames
      if (firstSubscribe && frames.length) {
        timeline.__playedFrames = true
        for (const frame of frames) {
          setTimeout(() => {
            liveState.set(sourceUrl, frame.payload)
            // Push to every active subscriber, not just this one —
            // covers the case of multiple iframes / tabs.
            for (const c of liveControllers.get(sourceUrl) ?? []) {
              try { c.enqueue(enc.encode(`data: ${JSON.stringify(frame.payload)}\n\n`)) }
              catch { /* dropped */ }
            }
          }, frame.afterMs)
        }
      }
    },
    cancel(controller) {
      unregisterController(sourceUrl, controller)
    },
  })
  return new Response(stream, {
    status: 200,
    headers: {
      'content-type': 'text/event-stream',
      'cache-control': 'no-cache',
      'connection': 'keep-alive',
    },
  })
}

/**
 * POST /_deck/api/<path> handler. Looks up `scenarios[<source>].mutations[<path>]`,
 * runs it against the current liveState's `value` + the request body,
 * pushes the new envelope to every subscriber of the matching source.
 *
 * Lookup: find the scenario whose `mutations` map has a key matching
 * the request path (e.g. POST /_deck/api/users/add → look for a
 * scenario with mutations['/users/add']).
 */
async function handleMutation(apiPath, request) {
  const path = apiPath.replace(/^\/_deck\/api/, '')
  let body = null
  try {
    body = await request.json()
  } catch { /* empty body or non-JSON — pass null */ }

  for (const [sourceUrl, timeline] of scenarios.entries()) {
    const handler = timeline.mutations?.[path]
    if (typeof handler !== 'function') continue
    try {
      const current = liveState.get(sourceUrl)
      const currentValue = current && current.kind === 'ok' ? current.value : undefined
      const nextValue = handler(currentValue, body)
      const nextEnvelope = { kind: 'ok', value: nextValue }
      liveState.set(sourceUrl, nextEnvelope)
      const enc = new TextEncoder()
      for (const c of liveControllers.get(sourceUrl) ?? []) {
        try { c.enqueue(enc.encode(`data: ${JSON.stringify(nextEnvelope)}\n\n`)) }
        catch { /* dropped */ }
      }
      return new Response(JSON.stringify({ ok: true, value: nextValue }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    } catch (err) {
      return new Response(JSON.stringify({ ok: false, error: String(err) }), {
        status: 500,
        headers: { 'content-type': 'application/json' },
      })
    }
  }
  return new Response(JSON.stringify({ ok: false, error: 'no mutation handler' }), {
    status: 404,
    headers: { 'content-type': 'application/json' },
  })
}

function registerController(sourceUrl, controller) {
  let set = liveControllers.get(sourceUrl)
  if (!set) {
    set = new Set()
    liveControllers.set(sourceUrl, set)
  }
  set.add(controller)
}

function unregisterController(sourceUrl, controller) {
  const set = liveControllers.get(sourceUrl)
  if (!set) return
  set.delete(controller)
  if (set.size === 0) liveControllers.delete(sourceUrl)
}
