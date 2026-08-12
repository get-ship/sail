// L1 conformance spec: <fieldset> (mechanism-2, native + FIELDSET_SKIN_CSS).
// Probe: bordered group with a legend.
export default {
  tag: 'fieldset',
  width: 640,
  rows() {
    return [
      // default rest
      `<fieldset style="width:280px"><legend>Account details</legend><input placeholder="Name" style="width:100%"><output style="display:block;margin-top:8px">Preview output</output></fieldset>`,
      // disabled axis (visual.cross)
      `<fieldset disabled style="width:280px"><legend>Billing (locked)</legend><input placeholder="Card" style="width:100%"></fieldset>`,
    ]
  },
  probe() {
    const bad = []
    document.querySelectorAll('fieldset').forEach((el) => {
      const cs = getComputedStyle(el)
      if (cs.borderTopStyle === 'none' || parseFloat(cs.borderTopWidth) === 0) bad.push({ why: 'no-border' })
      if (!el.querySelector('legend')) bad.push({ why: 'no-legend' })
    })
    if (!document.querySelector('fieldset[disabled]')) bad.push({ why: 'missing-disabled-row' })
    return { count: document.querySelectorAll('fieldset').length, bad: bad.slice(0, 8) }
  },
}
