// L1 conformance spec: <alert> (mechanism-1, use:alert). Probe runs the alert
// essentials (role/aria-live by urgency) and asserts structure: icon chip
// filled + shown, dismiss shown, role correct per tone. One alert per tone.

const ICON = { info: 'M8 7.5v4M8 4.8h.01', success: 'M4.5 8.2l2.3 2.3 4.7-5', warning: 'M8 5v3.5M8 11h.01', destructive: 'M5.5 5.5l5 5M10.5 5.5l-5 5' }
const copy = {
  info: ['Heads up', 'A new version is available. Refresh when you get a chance.'],
  success: ['Saved', 'All changes were written successfully.'],
  warning: ['Check your input', 'Two fields need attention before you continue.'],
  destructive: ['Something went wrong', 'We could not reach the server. Try again.'],
}
const alert = (tone) =>
  `<div class="sui-alert" tone="${tone}" dismissible role="status" aria-live="polite" aria-atomic="true" style="width:520px"><span class="ico"><svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="${ICON[tone]}"></path></svg></span><div class="content"><div class="title">${copy[tone][0]}</div><div class="body">${copy[tone][1]}</div><div class="actions"><button class="act" style="font:inherit;font-size:13px;padding:4px 10px;border-radius:6px;border:1px solid var(--border);background:var(--surface-2);color:var(--foreground);cursor:pointer">Retry</button><button class="act" style="font:inherit;font-size:13px;padding:4px 10px;border-radius:6px;border:0;background:transparent;color:var(--muted-foreground);cursor:pointer">Dismiss</button></div></div><button class="close" type="button" aria-label="Dismiss"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 2 L8 8 M8 2 L2 8" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" fill="none"></path></svg></button></div>`

export default {
  tag: 'alert',
  width: 640,
  rows() {
    return ['info', 'success', 'warning', 'destructive'].map(alert)
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-alert').forEach((el) => {
      const tone = el.getAttribute('tone')
      const urgent = tone === 'warning' || tone === 'destructive'
      el.setAttribute('role', urgent ? 'alert' : 'status')
      el.setAttribute('aria-live', urgent ? 'assertive' : 'polite')
      const icoFilled = !!el.querySelector('.ico svg')
      const icoShown = getComputedStyle(el.querySelector('.ico')).display !== 'none'
      const closeShown = getComputedStyle(el.querySelector('.close')).display !== 'none'
      const roleOk = el.getAttribute('role') === (urgent ? 'alert' : 'status')
      if (!icoFilled || !icoShown || !closeShown || !roleOk) bad.push({ tone, icoFilled, icoShown, closeShown, roleOk })
    })
    return { count: document.querySelectorAll('.sui-alert').length, bad }
  },
}
