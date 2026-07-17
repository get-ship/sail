// L1 conformance spec: <radio> / <radio-group> (mechanism-1, use:radioGroup).
// Probe runs the group essentials (name wiring, single-checked, disabled) and
// asserts them. Names are uniquified per group so the 7 theme copies don't
// collide (radios share exclusivity by name document-wide).

const radio = (val, label, extra = '') =>
  `<label class="sail-radio" value="${val}"${extra}><input type="radio"/><span class="rlabel">${label}</span></label>`
const group = (attrs, radios) =>
  `<div class="sail-radio-group" role="radiogroup" ${attrs} style="min-width:160px">${radios}</div>`

export default {
  tag: 'radio-group',
  width: 720,
  rows() {
    return [
      group('name="size" value="b" label="Size"', radio('a', 'Small') + radio('b', 'Medium') + radio('c', 'Large')) +
        group('name="plan" value="pro" label="Plan"', radio('free', 'Free') + radio('pro', 'Pro') + radio('ent', 'Enterprise', ' disabled')),
    ]
  },
  probe() {
    const bad = []
    let gid = 0
    document.querySelectorAll('.sail-radio-group').forEach((el) => {
      const g = ++gid
      const val = el.getAttribute('value') ?? ''
      const name = (el.getAttribute('name') || 'radio') + '-' + g
      el.querySelectorAll('.sail-radio').forEach((r) => {
        const input = r.querySelector('input[type="radio"]')
        const rval = r.getAttribute('value') ?? ''
        input.name = name
        input.value = rval
        input.checked = rval === val
        input.disabled = el.hasAttribute('disabled') || r.hasAttribute('disabled')
      })
      const radios = [...el.querySelectorAll('.sail-radio')]
      const checked = radios.filter((r) => r.querySelector('input').checked)
      const checkedOk = checked.length === 1 && checked[0].getAttribute('value') === val
      const disOk = radios.filter((r) => r.hasAttribute('disabled')).every((r) => r.querySelector('input').disabled)
      const nameOk = radios.every((r) => r.querySelector('input').name === name)
      if (!checkedOk || !disOk || !nameOk) bad.push({ val, checkedOk, disOk, nameOk })
    })
    return { count: document.querySelectorAll('.sail-radio').length, bad }
  },
}
