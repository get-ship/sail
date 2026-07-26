// L1 conformance spec: <label> (mechanism-1, compiles-away). Wraps text + a
// required asterisk (aria-hidden span). Probe: renders non-empty; the required
// marker is present as the last child span.
const lb = (a = '', txt = 'Email address') => `<label class="sail-label" ${a}>${txt}<span aria-hidden="true">*</span></label>`
export default {
  tag: 'label',
  width: 640,
  rows() {
    return [`${lb('required')}&nbsp;&nbsp;&nbsp;${lb('', 'Optional field')}`]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-label').forEach((el) => {
      if ((el.textContent || '').replace('*', '').trim().length === 0) bad.push({ why: 'empty-label' })
      if (!el.querySelector('span[aria-hidden="true"]')) bad.push({ why: 'no-req-marker' })
    })
    return { count: document.querySelectorAll('.sail-label').length, bad: bad.slice(0, 8) }
  },
}
