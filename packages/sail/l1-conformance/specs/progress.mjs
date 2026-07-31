// L1 conformance spec: <progress> (mechanism-2, native + PROGRESS map).
// determinate values + sizes + indeterminate. Probe asserts each renders and
// the determinate ones reflect value/max (native .position), indeterminate
// reads -1. Markup mirrors freeform host stamp.

const PR =
  'sail-progress sx-appearancenone sx-displayinlineblock sx-alignmiddle sx-wfull sx-h2 sx-borderWidth0 sx-radiusfull sx-overflowhidden sx-bgmuted sx-fgprimary sx-sizesm-h15 sx-sizemd-h2 sx-sizelg-h3'

const pr = (a) => {
  const m = a.match(/class="([^"]*)"/)
  const rest = a.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${PR} ${m[1]}` : PR
  return `<progress class="${cls}" ${rest} style="width:180px"></progress>`
}

export default {
  tag: 'progress',
  width: 700,
  rows() {
    return [
      [pr('value="0.3" max="1"'), pr('value="0.6" max="1"'), pr('value="0.9" max="1"')].join(' '),
      [pr('value="0.5" max="1" size="sm"'), pr('value="0.5" max="1" size="lg"'), pr('')].join(' '),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('progress').forEach((el) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ why: 'not-rendered' })
      if (el.hasAttribute('value')) {
        const expected = parseFloat(el.getAttribute('value')) / parseFloat(el.getAttribute('max') || '1')
        if (Math.abs(el.position - expected) > 0.01) bad.push({ why: 'value-mismatch', pos: el.position, expected })
      } else if (el.position !== -1) {
        bad.push({ why: 'not-indeterminate', pos: el.position })
      }
    })
    return { count: document.querySelectorAll('progress').length, bad: bad.slice(0, 8) }
  },
}
