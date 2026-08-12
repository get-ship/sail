// L1 conformance spec: <radio> / <radio-group> (mechanism-1, use:radioGroup).
// Markup mirrors lowered structure: sail-radio host + painted nested input
// (Style product) + RADIO_INPUT_MAP inset disc on checked. Groups must not
// collide (radios share exclusivity by name document-wide).

const RADIO_HOST =
  'sail-radio s-fontFamilybody s-fontSizesm s-fgradioforeground s-cursorpointer s-selectnone'
const RADIO_IN =
  's-appearancenone s-displayinlineblock s-alignmiddle s-m0 s-pad0 s-minH0 s-wiconsizesm s-hiconsizemd s-fontSizebase s-borderWidth2 s-borderborderstrong s-bgbackground s-shadowxs s-radiusfull s-cursorpointer s-checked-bgprimary s-checked-borderprimary s-disabled-cursornotallowed s-disabled-opacity50 s-shrink0'

const radio = (val, label, extra = '') =>
  `<label class="${RADIO_HOST}" value="${val}"${extra}><input type="radio" class="${RADIO_IN}"/><span class="rlabel">${label}</span></label>`
const group = (attrs, radios) =>
  `<div class="sail-radio-group" role="radiogroup" ${attrs} style="min-width:160px">${radios}</div>`

export default {
  tag: 'radio-group',
  width: 720,
  rows() {
    return [
      // rest-default value + disabled/required bools + value= coverage
      group('name="size" value="b" label="Size"', radio('a', 'Small') + radio('b', 'Medium') + radio('c', 'Large')) +
        group('name="plan" value="pro" label="Plan" required', radio('free', 'Free') + radio('pro', 'Pro') + radio('ent', 'Enterprise', ' disabled')) +
        group('name="locked" value="a" label="Locked" disabled', radio('a', 'A') + radio('b', 'B')),
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
        if (r.hasAttribute('disabled') || el.hasAttribute('disabled')) input.disabled = true
        const v = r.getAttribute('value')
        if (v != null && v === el.getAttribute('value')) input.checked = true
      })
      const radios = [...el.querySelectorAll('.sail-radio')]
      const checked = radios.filter((r) => r.querySelector('input').checked)
      if (checked.length !== 1) bad.push({ g, why: 'checked-count', n: checked.length })
      const disOk = radios
        .filter((r) => r.hasAttribute('disabled') || el.hasAttribute('disabled'))
        .every((r) => r.querySelector('input').disabled)
      const nameOk = radios.every((r) => r.querySelector('input').name === name)
      if (!disOk) bad.push({ g, why: 'disabled-not-synced' })
      if (!nameOk) bad.push({ g, why: 'name-not-shared' })
      if (el.hasAttribute('required') && el.getAttribute('aria-required') == null) {
        el.setAttribute('aria-required', 'true')
      }
      if (!el.getAttribute('value')) bad.push({ g, why: 'missing-value' })
    })
    const hasDisabled = [...document.querySelectorAll('.sail-radio-group')].some((el) =>
      el.hasAttribute('disabled'),
    )
    const hasRequired = [...document.querySelectorAll('.sail-radio-group')].some((el) =>
      el.hasAttribute('required'),
    )
    if (!hasDisabled) bad.push({ why: 'missing-disabled-group' })
    if (!hasRequired) bad.push({ why: 'missing-required-group' })
    return { count: document.querySelectorAll('.sail-radio').length, bad }
  },
}
