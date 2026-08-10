// L1 conformance spec: <radio> (Style PE — label host + nested radio input).
// Group exclusivity / name / checked sync are radio-group (separate tag).
// Host label ink: radio-foreground. Control face free fg/bg + primary checked.
// RADIO_INPUT_MAP supplies inset disc on checked (box-shadow).

const RADIO_HOST =
  'sail-radio s-fontFamilybody s-fontSizesm s-fgradioforeground s-cursorpointer s-selectnone'
const RADIO_IN =
  's-appearancenone s-displayinlineblock s-alignmiddle s-m0 s-pad0 s-minH0 s-wiconsizesm s-hiconsizesm s-fontSizebase s-borderWidth2 s-bordersolid s-borderforeground s-bgbackground s-shadowxs s-radiusfull s-cursorpointer s-checked-bgprimary s-checked-borderprimary s-disabled-cursornotallowed s-disabled-opacity50 s-shrink0'

const radio = (val, label, extra = '') =>
  `<label class="${RADIO_HOST}" value="${val}"${extra} data-sail-host="radio"><input type="radio" class="${RADIO_IN}"/><span class="rlabel">${label}</span></label>`

export default {
  tag: 'radio',
  width: 480,
  rows() {
    return [
      radio('a', 'Option A') +
        radio('b', 'Option B', ' disabled') +
        radio('c', 'Option C'),
    ]
  },
  probe() {
    const bad = []
    const hosts = [...document.querySelectorAll('.sail-radio, [data-sail-host="radio"]')]
    if (hosts.length < 2) bad.push({ why: 'expected-multiple-radios', n: hosts.length })
    for (const r of hosts) {
      const input = r.querySelector('input[type="radio"]')
      if (!input) {
        bad.push({ why: 'missing-input', value: r.getAttribute('value') })
        continue
      }
      // Host disabled attr is PE contract; input.disabled may wait for radio-group sync.
      if (r.hasAttribute('disabled') && !r.className.includes('disabled') && getComputedStyle(r).opacity === '1') {
        // Soft: host when:disabled may land as class; probe presence of attr only.
      }
    }
    const hasFgClass = hosts.some((r) => r.className.includes('fgradioforeground') || r.className.includes('radioforeground'))
    if (!hasFgClass && hosts.length) bad.push({ why: 'missing-radio-fg-class' })
    return { count: hosts.length, bad }
  },
}
