// L1 conformance spec: <radio> / <radio-group> (mechanism-1, use:radioGroup).
// Markup mirrors lowered structure: sail-radio host + painted nested input
// (Style product) + RADIO_INPUT_MAP inset disc on checked. Groups must not
// collide (radios share exclusivity by name document-wide).

const RADIO_HOST =
  'sail-radio sx-fontFamilybody sx-fontSizesm sx-fgradioforeground sx-cursorpointer sx-selectnone'
const RADIO_IN =
  'sx-appearancenone sx-displayinlineblock sx-alignmiddle sx-m0 sx-pad0 sx-minH0 sx-wiconsizesm sx-hiconsizesm sx-fontSizebase sx-borderWidth2 sx-borderborderstrong sx-bgbackground sx-shadowxs sx-radiusfull sx-cursorpointer sx-checked-bgprimary sx-checked-borderprimary sx-disabled-cursornotallowed sx-disabled-opacity50 sx-shrink0'

const radio = (val, label, extra = '') =>
  `<label class="${RADIO_HOST}" value="${val}"${extra}><input type="radio" class="${RADIO_IN}"/><span class="rlabel">${label}</span></label>`
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
    document.querySelectorAll('.sail-radio-group').forEach((el) => {
      // emulate use:radioGroup essentials: shared name + value→checked
      const g = el.getAttribute('name') || 'g'
      const name = (el.getAttribute('name') || 'radio') + '-' + g
      el.querySelectorAll('.sail-radio').forEach((r) => {
        const input = r.querySelector('input[type="radio"]')
        if (!input) return
        input.name = name
        if (r.hasAttribute('disabled')) input.disabled = true
        const v = r.getAttribute('value')
        if (v != null && v === el.getAttribute('value')) input.checked = true
      })
      const radios = [...el.querySelectorAll('.sail-radio')]
      const checked = radios.filter((r) => r.querySelector('input').checked)
      if (checked.length !== 1) bad.push({ g, why: 'checked-count', n: checked.length })
      const disOk = radios.filter((r) => r.hasAttribute('disabled')).every((r) => r.querySelector('input').disabled)
      const nameOk = radios.every((r) => r.querySelector('input').name === name)
      if (!disOk) bad.push({ g, why: 'disabled-not-synced' })
      if (!nameOk) bad.push({ g, why: 'name-not-shared' })
    })
    return { count: document.querySelectorAll('.sail-radio').length, bad }
  },
}
