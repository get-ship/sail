// L1 conformance spec: <meter> (mechanism-2, native + METER_SKIN_CSS). value
// range + sizes. Probe asserts each renders with a non-zero box (the skin
// dropped native chrome via appearance:none, so a bare meter must still show).

const mt = (a) => `<meter ${a} style="width:180px"></meter>`

export default {
  tag: 'meter',
  width: 700,
  rows() {
    return [
      [mt('value="0.2" min="0" max="1"'), mt('value="0.6" min="0" max="1"'), mt('value="0.9" min="0" max="1"')].join(' '),
      [mt('value="0.5" min="0" max="1" size="sm"'), mt('value="0.5" min="0" max="1" size="lg"')].join(' '),
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('meter').forEach((el) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) bad.push({ why: 'not-rendered', value: el.getAttribute('value') })
      if (getComputedStyle(el).appearance !== 'none') bad.push({ why: 'not-appearance-none' })
    })
    return { count: document.querySelectorAll('meter').length, bad: bad.slice(0, 8) }
  },
}
