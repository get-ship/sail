import { test, expect } from '@playwright/test'

// ── deck dev — security headers reach the wire ───────────────────────────────
//
// deck-core's error responses carry the security header set (esp.
// `X-Content-Type-Options: nosniff`) because a 404 body can reflect
// request-derived detail in dev — a `text/plain` body MIME-sniffed as HTML is a
// reflected-XSS vector. That's unit-tested at the `DeckResponse` layer; THIS
// proves it survives the axum adapter and lands on the actual HTTP response.

test('a 404 carries the security header set (nosniff etc.) at the wire', async ({ request }) => {
  // A dotted, single-segment path that doesn't exist → public-asset lookup →
  // 404 (the path that reflects `{file}` into the body in dev).
  const res = await request.get('/this-does-not-exist.js')
  expect(res.status()).toBe(404)
  const h = res.headers()
  expect(h['x-content-type-options'], 'nosniff must reach the wire on a 404').toBe('nosniff')
  expect(h['x-frame-options']).toBe('SAMEORIGIN')
  expect(h['content-security-policy'], 'CSP present on the error response').toBeTruthy()
})

test('the SPA shell (HTML) carries the security headers too', async ({ request }) => {
  const res = await request.get('/')
  expect(res.status()).toBe(200)
  const h = res.headers()
  expect(h['x-content-type-options']).toBe('nosniff')
  expect(h['x-frame-options']).toBe('SAMEORIGIN')
  expect(h['content-security-policy']).toBeTruthy()
})
