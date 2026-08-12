// L1 conformance spec: <theme-picker> (use:themeSelect + use:anchor).
// visual.cross: value · inline · modeOnly.
// Product grid is runtime-populated by themeSelect from THEMES ∩ effective set.
// RENDER: lowered markup + forced-open panel. BEHAVIOR: open + select via callbacks.

import { lowered } from '../lowered.mjs'

// One instance per row, with unique ids so the popovers don't collide.
const picker = (n, attrs = '') => {
  const html = lowered('theme-picker', { attrs })
  if (!html) {
    return (
      `<div id="tp${n}" class="sail-theme-picker" data-sail-host="theme-picker" data-theme-picker="1" ${attrs}>` +
      `<button type="button" data-picker-trigger="full" popovertarget="tp${n}-panel" aria-label="Choose theme">◐</button>` +
      `<div id="tp${n}-panel" popover data-theme-listbox placement="bottom-end"></div>` +
      `</div>`
    )
  }
  return html
    .replace(/sail-root /g, '')
    .replace(/id="sail-theme-panel"/g, `id="tp${n}-panel"`)
    .replace(/popovertarget="sail-theme-panel"/g, `popovertarget="tp${n}-panel"`)
    .replace(
      /class="([^"]*sail-theme-picker[^"]*)"/,
      `id="tp${n}" class="$1"`,
    )
    // if no sail-theme-picker class, stamp id on first host div
    .replace(
      /^<div (?![^>]*\bid=)/,
      `<div id="tp${n}" `,
    )
}

export default {
  tag: 'theme-picker',
  width: 460,
  rows() {
    // Panel is a popover (top layer). Force one open in-flow for snapshot.
    // visual.cross stamps value / inline / modeOnly (mode-only kebab + modeOnly token).
    return [
      picker(1, 'data-l1-mark="1"').replace(
        /class="([^"]*)"/,
        'class="$1 l1-mark"',
      ),
      picker(2, 'value="console"').replace(
        /(\s)popover(?:="")?(?=\s|>)/,
        '$1popover style="position:static;display:grid;gap:6px;min-width:200px"',
      ),
      picker(3, 'inline'),
      picker(4, 'mode-only modeOnly'),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => (el ? getComputedStyle(el)[p] : null)
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    // Trigger: popovertarget + accessible name
    const triggers = document.querySelectorAll('[data-picker-trigger="full"]')
    if (triggers.length === 0) bad.push({ why: 'no-full-trigger' })
    triggers.forEach((t) => {
      if (!t.getAttribute('popovertarget')) bad.push({ why: 'trigger-has-no-popovertarget' })
      if (!(t.getAttribute('aria-label') ?? '').trim())
        bad.push({ why: 'trigger-has-no-accessible-name' })
    })

    // Panel present
    const panels = document.querySelectorAll('[data-theme-listbox], [popover][id*="panel"]')
    if (panels.length === 0) bad.push({ why: 'no-theme-panel' })

    // Live specimen: at least one [theme] preview paints a real surface
    const previews = document.querySelectorAll('[theme]')
    let painted = 0
    previews.forEach((el) => {
      const bg = cs(el, 'backgroundColor')
      if (bg && !clear(bg)) painted++
    })
    if (previews.length === 0) bad.push({ why: 'no-theme-specimens' })
    else if (painted === 0) bad.push({ why: 'specimens-transparent' })

    // Hosts present for count
    const hosts = document.querySelectorAll('[data-theme-picker], [data-sail-host="theme-picker"]')
    return { count: hosts.length || triggers.length, bad: bad.slice(0, 8) }
  },

  // Behavior deferred: themeSelect reads localStorage (opaque origin under
  // setContent) and runtime-populates the product catalog. Scaffold asserts
  // lowered anatomy + visual.cross attrs only; operator pairs live apply later.
}
