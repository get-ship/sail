// L1 conformance spec: <accordion> (mechanism-1, native <details>).
// visual.cross: open · disabled · name · multiple.
// Probe asserts open/closed content height contract.

const chev =
  '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>'
const trig =
  'display:flex;align-items:center;justify-content:space-between;gap:8px;padding:12px 14px;font-weight:600;font-size:14px;color:var(--accordion-foreground,var(--foreground));background:var(--accordion,var(--card))'
const body =
  'padding:0 14px 14px;font-size:14px;color:var(--muted-foreground);line-height:1.5'
const box =
  'border:1px solid var(--accordion-border,var(--border));border-radius:10px;background:var(--accordion,var(--card));overflow:hidden;width:340px'

const coll = (attrs = '', open = false, extraClass = '') =>
  `<details class="sail-accordion${extraClass ? ` ${extraClass}` : ''}" data-sail-host="accordion" ${attrs}${open ? ' open' : ''} style="${box}">` +
  `<summary class="trigger-wrap" style="${trig}"><span>What is SAIL?</span>${chev}</summary>` +
  `<div class="content-wrap"><div class="content-inner" style="${body}">A trustworthy compiler and a beautiful component language — L1 primitives that compile away to static DOM with zero runtime.</div></div>` +
  `</details>`

export default {
  tag: 'accordion',
  width: 780,
  rows() {
    return [
      // default/rest (closed) + open
      coll('', false, 'l1-mark') + coll('', true),
      // disabled
      coll('disabled', false),
      // name + multiple group
      coll('name="faq"', false) + coll('name="faq" multiple', true),
    ]
  },
  probe() {
    const bad = []
    // Measure whether the content ADDS to the details' height — true only when
    // open. Robust to the collapse mechanism (display:none OR clip-to-0 via
    // ::details-content both give detailsH ≈ summaryH when closed).
    document.querySelectorAll('.sail-accordion').forEach((el) => {
      const open = el.hasAttribute('open')
      const detailsH = el.getBoundingClientRect().height
      const summaryH = el.querySelector('summary').getBoundingClientRect().height
      const contentContributes = detailsH > summaryH + 2
      if (contentContributes !== open)
        bad.push({ open, detailsH: Math.round(detailsH), summaryH: Math.round(summaryH) })
      // Shell paints via accordion tokens (or fallbacks)
      const cs = getComputedStyle(el)
      if (
        cs.backgroundColor === 'rgba(0, 0, 0, 0)' ||
        cs.backgroundColor === 'transparent'
      ) {
        bad.push({ why: 'shell-transparent' })
      }
    })
    return { count: document.querySelectorAll('.sail-accordion').length, bad }
  },
}
