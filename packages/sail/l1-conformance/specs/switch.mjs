// L1 conformance spec: <switch> (mechanism-1, native checkbox role=switch +
// skin). state / size / tone matrices. Probe asserts it's custom-styled
// (appearance:none), the checked track color differs from unchecked, and the
// frozen-token guard on the off-track.

const TONES = ['default', 'primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
const sw = (a) => `<input type="checkbox" role="switch" class="sail-switch" ${a}>`

export default {
  tag: 'switch',
  width: 720,
  rows() {
    return [
      `${sw('')} ${sw('checked')} ${sw('disabled')} ${sw('disabled checked')}`,
      `${sw('size="sm" checked')} ${sw('size="md" checked')} ${sw('size="lg" checked')}`,
      TONES.map((t) => sw(`tone="${t}" checked`)).join(' '),
    ]
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    document.querySelectorAll('.sail-switch').forEach((el) => {
      if (cs(el, 'appearance') !== 'none') bad.push({ why: 'not-appearance-none', appearance: cs(el, 'appearance') })
    })
    const scope = "[theme='ship'] "
    const off = cs(document.querySelector(scope + '.sail-switch:not([checked]):not([disabled])'), 'backgroundColor')
    const on = cs(document.querySelector(scope + ".sail-switch[checked]:not([tone]):not([disabled])"), 'backgroundColor')
    if (off === on) bad.push({ why: 'checked-no-color-change', off, on })
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .sail-switch:not([checked]):not([disabled])`), 'backgroundColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship'), dark: mark('console') })
    return { count: document.querySelectorAll('.sail-switch').length, bad: bad.slice(0, 10) }
  },
}
