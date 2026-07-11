// L1 conformance spec: <label> (mechanism-1, compiles-away). Wraps text + a
// required asterisk (.req). Probe: renders non-empty; the required marker gates
// on [required] if the skin gates it (else just present).
const lb = (a = '', txt = 'Email address') => `<label class="sui-label" ${a}>${txt}<span class="req" aria-hidden="true">*</span></label>`
export default {
  tag: 'label',
  width: 640,
  rows() {
    return [`${lb('required')}&nbsp;&nbsp;&nbsp;${lb('', 'Optional field')}`]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sui-label').forEach((el) => {
      if ((el.textContent || '').replace('*', '').trim().length === 0) bad.push({ why: 'empty-label' })
      if (!el.querySelector('.req')) bad.push({ why: 'no-req-marker' })
    })
    return { count: document.querySelectorAll('.sui-label').length, bad: bad.slice(0, 8) }
  },
}
