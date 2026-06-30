import { test, expect } from '@playwright/test'

// ── deck dev — public/ static assets + the asset-vs-route split ──────────────
//
// deck dev serves files from the app's `public/` dir at the ROOT, and decides
// public-asset-vs-SPA-route by a single rule: a single-segment DOTTED path
// (`favicon.svg`, `robots.txt`) is a public asset; a bare segment (`/settings`)
// is a client route → the SPA shell. The SPA/HMR/SSR specs never exercise this,
// so a regression in `serve_public_asset` / the dotted-name split would go
// unnoticed.

test('serves a public/ asset at the root with the right content-type', async ({ request }) => {
  const res = await request.get('/favicon.svg')
  expect(res.status()).toBe(200)
  expect(res.headers()['content-type']).toContain('image/svg+xml')
  expect(await res.text()).toContain('<svg')
})

test('a bare segment is an SPA route (shell), NOT a public-asset 404', async ({ request }) => {
  // No dot → client route → the dev server serves the SPA shell and lets the
  // client router resolve it (this is the split that keeps `/settings` working
  // while `/favicon.svg` hits the asset path).
  const res = await request.get('/some-client-route')
  expect(res.status()).toBe(200)
  expect(await res.text()).toContain('id="root"')
})
