// L1 conformance spec: <label> (mechanism-1, compiles-away). Wraps text + a
// required asterisk (aria-hidden span). Probe: non-empty ink class; required
// marker present on required hosts. Size ladder covered via size= attrs.
//
// Product structure always emits the * span; group-required shows it. Harness
// stamps the span on every cell so structure is honest; optional hosts still
// carry the marker node (product display:none without required).

const LBL =
  'sail-label s-fglabelforeground s-cursorpointer s-fontFamilybody s-fontSizesm'

const sizeCls = {
  sm: 's-sizesm-fontSizesm',
  md: 's-sizemd-fontSizebase',
  lg: 's-sizelg-fontSizelg',
}

const lb = (attrs = '', txt = 'Email address') => {
  const size = /size="([^"]+)"/.exec(attrs)?.[1] || ''
  const sc = size && sizeCls[size] ? ` ${sizeCls[size]}` : ''
  const a = attrs ? ` ${attrs}` : ''
  return `<label class="${LBL}${sc}"${a}>${txt}<span aria-hidden="true">*</span></label>`
}

export default {
  tag: 'label',
  width: 720,
  rows() {
    return [
      // zero-config rest + size ladder + required bool
      lb('', 'rest-default') +
        '&nbsp;&nbsp;' +
        lb('size="sm"', 'sm') +
        '&nbsp;&nbsp;' +
        lb('size="md"', 'md') +
        '&nbsp;&nbsp;' +
        lb('size="lg"', 'lg') +
        '&nbsp;&nbsp;' +
        lb('required', 'required') +
        '&nbsp;&nbsp;' +
        lb('required size="lg"', 'required lg'),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('.sail-label').forEach((el) => {
      if ((el.textContent || '').replace('*', '').trim().length === 0) bad.push({ why: 'empty-label' })
      if (!el.querySelector('span[aria-hidden="true"]')) bad.push({ why: 'no-req-marker' })
      const cs = getComputedStyle(el)
      if (cs.color === 'rgba(0, 0, 0, 0)' || cs.color === 'transparent') {
        bad.push({ why: 'no-token-fg' })
      }
    })
    return { count: document.querySelectorAll('.sail-label').length, bad: bad.slice(0, 8) }
  },
}
