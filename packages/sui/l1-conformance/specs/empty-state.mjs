// L1 conformance spec: <empty-state> (mechanism-1, use:emptyState). Probe runs
// the essentials (role/aria-live by variant) and asserts the empty-region
// gates: title/desc filled, icon shown iff it has an svg, actions shown iff
// non-empty. default / inline / connecting variants.

const inbox = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h5l2 3h4l2-3h5"/><path d="M5 5h14l2 7v6a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-6z"/></svg>'
const spinner = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 3a9 9 0 1 0 9 9" opacity=".9"/></svg>'
const btnP = 'font:inherit;font-size:13px;padding:6px 12px;border-radius:6px;border:1px solid var(--primary);background:var(--primary);color:var(--primary-foreground);cursor:pointer'
const btnG = 'font:inherit;font-size:13px;padding:6px 12px;border-radius:6px;border:1px solid var(--border);background:var(--surface-1);color:var(--foreground);cursor:pointer'
const es = (variant, tone, icon, title, desc, actions) =>
  `<div class="sui-empty-state"${variant ? ` variant="${variant}"` : ''}${tone ? ` tone="${tone}"` : ''} role="note" aria-live="off" style="max-width:560px"><div class="icon" aria-hidden="true">${icon}</div><div class="copy"><div class="title">${title}</div><div class="desc">${desc}</div></div><div class="actions">${actions}</div></div>`

export default {
  tag: 'empty-state',
  width: 720,
  rows() {
    return [
      es('', 'primary', inbox, 'No messages yet', 'When teammates send you something, it lands here.', `<button style="${btnP}">Compose</button><button style="${btnG}">Learn more</button>`),
      es('inline', 'info', inbox, 'Nothing in this view', 'Adjust filters to see more results.', `<button style="${btnG}">Reset filters</button>`),
      es('connecting', '', spinner, 'Connecting…', 'Establishing a secure session.', ''),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-empty-state').forEach((el) => {
      const variant = el.getAttribute('variant') || 'default'
      const connecting = variant === 'connecting'
      el.setAttribute('role', connecting ? 'status' : 'note')
      el.setAttribute('aria-live', connecting ? 'polite' : 'off')
      const titleFilled = (el.querySelector('.title')?.textContent || '').trim().length > 0
      const descFilled = (el.querySelector('.desc')?.textContent || '').trim().length > 0
      const iconEl = el.querySelector('.icon')
      const iconHasSvg = !!iconEl.querySelector('svg')
      const iconShown = getComputedStyle(iconEl).display !== 'none'
      const actionsEl = el.querySelector('.actions')
      const actionsShown = getComputedStyle(actionsEl).display !== 'none'
      const hasActions = actionsEl.children.length > 0
      const roleOk = el.getAttribute('role') === (connecting ? 'status' : 'note')
      if (!titleFilled || !descFilled || !roleOk) bad.push({ variant, why: 'slot/role', titleFilled, descFilled, roleOk })
      if (iconHasSvg !== iconShown) bad.push({ variant, why: 'icon-empty-gate', iconHasSvg, iconShown })
      if (hasActions !== actionsShown) bad.push({ variant, why: 'actions-empty-gate', hasActions, actionsShown })
    })
    return { count: document.querySelectorAll('.sui-empty-state').length, bad }
  },
}
