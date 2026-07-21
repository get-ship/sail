// L1 conformance spec: <error-state> (mechanism-1, use:recoverySurface). Probe runs
// the essentials (fill code region, expand details when present) and asserts the
// gates: title filled, role=alert, code region shown iff a code, details toggle
// shown iff there are details. default (code+details) / inline variants.

const ICON = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"></circle><path d="M12 8v5"></path><path d="M12 16h.01"></path></svg>'
const btnP = 'font:inherit;font-size:13px;padding:6px 12px;border-radius:6px;border:1px solid var(--destructive);background:var(--destructive);color:var(--destructive-foreground);cursor:pointer'
const btnG = 'font:inherit;font-size:13px;padding:6px 12px;border-radius:6px;border:1px solid var(--border);background:var(--surface-1);color:var(--foreground);cursor:pointer'
const es = (variant, opts = {}) =>
  `<div class="sail-error-state"${variant ? ` variant="${variant}"` : ''}${opts.code ? ` code="${opts.code}"` : ''} role="alert" style="max-width:560px"><div class="icon" aria-hidden="true">${ICON}</div><div class="copy"><div class="title">Couldn't load your data</div><div class="desc">The request failed after 3 retries. Check your connection and try again.</div><div class="code"></div><button class="details-toggle" type="button" aria-expanded="false">Show details</button><div class="details">${opts.details || ''}</div></div><div class="actions">${opts.actions || ''}</div></div>`

export default {
  tag: 'error-state',
  width: 720,
  rows() {
    return [
      es('', { code: 'ERR_NETWORK_503', details: 'at fetchData (api.ts:42)\n  at load (dashboard.ts:19)\n  Error: gateway timeout', actions: `<button style="${btnP}">Retry</button><button style="${btnG}">Go back</button>` }),
      es('inline', { actions: `<button style="${btnG}">Retry</button>` }),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-error-state').forEach((el) => {
      const code = el.querySelector('.code')
      const details = el.querySelector('.details')
      const toggle = el.querySelector('.details-toggle')
      if (code) code.textContent = el.getAttribute('code') ?? ''
      const has = details && (details.textContent ?? '').trim().length > 0
      if (has) { el.setAttribute('data-state', 'has-details open'); toggle.textContent = 'Hide details'; toggle.setAttribute('aria-expanded', 'true') }
      const variant = el.getAttribute('variant') || 'default'
      const titleOk = (el.querySelector('.title').textContent || '').trim().length > 0
      const roleOk = el.getAttribute('role') === 'alert'
      const codeShown = getComputedStyle(code).display !== 'none'
      const codeExpected = (el.getAttribute('code') || '').length > 0
      const toggleShown = getComputedStyle(toggle).display !== 'none'
      if (!titleOk || !roleOk) bad.push({ variant, why: 'title/role', titleOk, roleOk })
      if (codeShown !== codeExpected) bad.push({ variant, why: 'code-gate', codeShown, codeExpected })
      if (toggleShown !== has) bad.push({ variant, why: 'details-toggle-gate', toggleShown, has })
    })
    return { count: document.querySelectorAll('.sail-error-state').length, bad }
  },
}
