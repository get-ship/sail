// L1 conformance spec: <error-boundary> (mechanism-1, use:errorBoundary).
//
// The render surface is nearly all state, so the SNAPSHOT rows show the two
// states side by side: healthy (the boundary is invisible — you should see only
// the content) and caught (the fallback, destructive tone).
//
// The BEHAVIOR test is the one that matters. A boundary's whole contract is that
// a throw ANYWHERE in the protected subtree finds it: `runProtected` walks up the
// DOM from the fault site looking for a registered handle. Asserting `[data-caught]`
// from hand-written markup would only re-test the CSS gate — so this throws for
// real, through the runtime's own routing, and checks that the right boundary
// catches it.

const ICON =
  '<svg class="fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path><path d="M12 9v4"></path><path d="M12 17h.01"></path></svg>'

const eb = (id, { caught = false, message = '', content = '<p>The panel rendered fine.</p>' } = {}) =>
  `<div class="sui-error-boundary" id="${id}"${caught ? ' data-caught data-error-message="boom"' : ''}${message ? ` message="${message}"` : ''}>` +
  `<div class="content">${content}</div>` +
  `<div class="fallback" role="alert">${ICON}` +
  `<div class="fallback-body">` +
  `<div class="message">${caught ? message || 'Something went wrong' : 'Something went wrong'}</div>` +
  `<button class="retry" type="button">Retry</button>` +
  `</div></div></div>`

export default {
  tag: 'error-boundary',
  width: 640,
  rows() {
    return [
      eb('eb-healthy'),
      eb('eb-caught', { caught: true }),
      eb('eb-custom', { caught: true, message: 'This panel could not load.' }),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => (el ? getComputedStyle(el)[p] : null)

    document.querySelectorAll('.sui-error-boundary').forEach((el) => {
      const caught = el.hasAttribute('data-caught')
      const content = el.querySelector('.content')
      const fallback = el.querySelector('.fallback')

      // A HEALTHY boundary must not participate in layout at all — you wrap it
      // around markup you already laid out, and it must not disturb it.
      if (!caught) {
        if (cs(el, 'display') !== 'contents')
          bad.push({ id: el.id, why: 'healthy-root-boxes', display: cs(el, 'display') })
        if (cs(content, 'display') !== 'contents')
          bad.push({ id: el.id, why: 'healthy-content-boxes', display: cs(content, 'display') })
      }

      // The two halves must never be visible at once, in either state.
      const contentShown = cs(content, 'display') !== 'none'
      const fallbackShown = cs(fallback, 'display') !== 'none'
      if (contentShown === fallbackShown)
        bad.push({ id: el.id, why: 'both-or-neither', contentShown, fallbackShown, caught })
      if (caught && contentShown) bad.push({ id: el.id, why: 'caught-still-shows-content' })
      if (!caught && fallbackShown) bad.push({ id: el.id, why: 'healthy-shows-fallback' })

      if (caught && fallback.getAttribute('role') !== 'alert')
        bad.push({ id: el.id, why: 'fallback-not-announced' })
    })

    return { count: document.querySelectorAll('.sui-error-boundary').length, bad }
  },

  behavior: {
    rows() {
      // NESTED, deliberately: the innermost boundary must win. A boundary that
      // catches errors from a subtree already protected by a nearer one would
      // blow away a whole page for a fault in one widget.
      return [
        eb('outer', {
          content: eb('inner', { content: '<p id="fault">inner content</p>' }),
        }),
      ]
    },
    attach: [
      ['#outer', 'errorBoundary'],
      ['#inner', 'errorBoundary'],
    ],
    async run(page) {
      const bad = []
      const caughtOn = (id) => page.evaluate((i) => document.getElementById(i).hasAttribute('data-caught'), id)
      const shown = (sel) =>
        page.evaluate((s) => {
          const el = document.querySelector(s)
          return el ? getComputedStyle(el).display !== 'none' : null
        }, sel)

      // ---- healthy -------------------------------------------------------
      if (await caughtOn('inner')) bad.push({ why: 'caught-before-any-throw' })
      if (!(await shown('#inner > .content'))) bad.push({ why: 'healthy-content-hidden' })
      if (await shown('#inner > .fallback')) bad.push({ why: 'healthy-fallback-shown' })

      // ---- a REAL throw, routed by the real runtime -----------------------
      const evt = await page.evaluate(() => {
        globalThis.__caught = null
        document
          .getElementById('inner')
          .addEventListener('sui-error-caught', (e) => {
            globalThis.__caught = { message: e.detail.error.message, source: e.detail.source }
          })
        // Throw from a node DEEP inside the protected subtree and let the
        // runtime's own walk find the boundary. This is the contract.
        globalThis.SuiBoundary.runProtected(document.getElementById('fault'), 'render', () => {
          throw new Error('boom')
        })
        return globalThis.__caught
      })

      if (!evt) bad.push({ why: 'throw-was-not-routed-to-any-boundary' })
      else {
        if (evt.message !== 'boom') bad.push({ why: 'wrong-error', got: evt.message })
        if (evt.source !== 'render') bad.push({ why: 'source-not-classified', got: evt.source })
      }

      // INNERMOST WINS.
      if (!(await caughtOn('inner'))) bad.push({ why: 'inner-boundary-did-not-catch' })
      if (await caughtOn('outer')) bad.push({ why: 'outer-boundary-caught-too' })

      // The swap actually happened.
      if (await shown('#inner > .content')) bad.push({ why: 'content-still-visible-after-catch' })
      if (!(await shown('#inner > .fallback'))) bad.push({ why: 'fallback-not-visible-after-catch' })

      // The RAW ERROR IS NOT RENDERED. An exception string is written for a
      // developer, not for the person using the app. It rides the event and the
      // data attribute instead.
      const visible = await page.locator('#inner > .fallback .message').innerText()
      if (visible.includes('boom')) bad.push({ why: 'raw-error-text-rendered-to-the-user', visible })
      const attr = await page.evaluate(() => document.getElementById('inner').getAttribute('data-error-message'))
      if (attr !== 'boom') bad.push({ why: 'error-not-published-out-of-band', attr })

      // A SECOND throw must not overwrite the first. Effects inside the subtree
      // are owned by the enclosing component, survive the catch, and may throw
      // again — the first error is the true one; the rest are echoes.
      await page.evaluate(() => {
        globalThis.SuiBoundary.runProtected(document.getElementById('fault'), 'effect', () => {
          throw new Error('echo')
        })
      })
      const still = await page.evaluate(() => document.getElementById('inner').getAttribute('data-error-message'))
      if (still !== 'boom') bad.push({ why: 're-entrancy-guard-failed', got: still })

      // ---- recovery ------------------------------------------------------
      // Only drive the button if the fallback actually rendered. Clicking a
      // control that isn't there costs a 30s Playwright timeout, and the throw
      // takes every diagnostic gathered above down with it — so a broken
      // boundary would report "TimeoutError" instead of the reason it broke.
      if (await shown('#inner > .fallback')) {
        await page.locator('#inner > .fallback .retry').click()
        if (await caughtOn('inner')) bad.push({ why: 'retry-did-not-reset' })
        if (!(await shown('#inner > .content'))) bad.push({ why: 'content-did-not-return-after-reset' })
        if (await shown('#inner > .fallback')) bad.push({ why: 'fallback-still-shown-after-reset' })
      } else {
        bad.push({ why: 'recovery-unreachable', note: 'no fallback rendered, so Retry could not be exercised' })
      }

      return bad
    },
  },
}
