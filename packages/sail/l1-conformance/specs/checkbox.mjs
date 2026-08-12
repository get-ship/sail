// L1 conformance: <checkbox> (spliced type=checkbox, zero JS).

const CB =
  'sail-checkbox s-appearancenone s-displayinlineblock s-alignmiddle s-m0 s-pad0 s-minH0 s-wiconsizesm s-hiconsizesm s-fontSizebase s-borderWidth2 s-bordersolid s-borderforeground s-bgbackground s-shadowxs s-radiussm s-cursorpointer s-checked-bgprimary s-checked-borderprimary s-disabled-cursornotallowed s-disabled-opacity50 s-sizexs-wiconsizexs s-sizexs-hiconsizexs s-sizesm-wiconsizexs s-sizesm-hiconsizexs s-sizemd-wiconsizesm s-sizemd-hiconsizesm s-sizelg-wiconsizemd s-sizelg-hiconsizemd s-invalidtrue-borderdestructive s-shrink0'

const box = (attrs = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${CB} ${m[1]}` : CB
  return `<label><input type="checkbox" class="${cls}" data-sail-host="checkbox" ${rest}> Label</label>`
}

export default {
  tag: 'checkbox',
  width: 640,
  rows() {
    return [
      [box('class="l1-mark"'), box('checked'), box('disabled'), box('checked disabled')].join(''),
      [box('id="ind"'), box('invalid')].join(''),
      [box("size='sm'"), box("size='md'"), box("size='lg'")].join(''),
    ]
  },
  setup() {
    document.querySelectorAll('#ind').forEach((el) => (el.indeterminate = true))
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll("input[type='checkbox']").forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      if (Math.abs(r.width - r.height) > 1) bad.push({ i, why: 'not-square', w: r.width, h: r.height })
      if (r.width < 12) bad.push({ i, why: 'too-small', w: r.width })
      if (clear(cs(el, 'borderTopColor')) && clear(cs(el, 'backgroundColor')))
        bad.push({ i, why: 'invisible' })
    })

    const scope = "[theme='ship'] "
    const q = (s) => document.querySelector(scope + s)
    const base = q("input[type='checkbox'].l1-mark")
    const on = q("input[type='checkbox'][checked]")
    const ind = q('#ind')
    const off = q("input[type='checkbox'][disabled]")

    if (base && on && cs(base, 'backgroundColor') === cs(on, 'backgroundColor'))
      bad.push({ why: 'checked-not-filled', base: cs(base, 'backgroundColor') })
    if (base && ind && cs(ind, 'backgroundColor') === cs(base, 'backgroundColor'))
      bad.push({ why: 'indeterminate-looks-unchecked' })
    if (base && off && cs(off, 'opacity') === cs(base, 'opacity') && cs(off, 'backgroundColor') === cs(base, 'backgroundColor'))
      bad.push({ why: 'disabled-not-distinct' })
    const w = (s) => {
      const el = q(`input[type='checkbox'][size='${s}']`)
      return el ? el.getBoundingClientRect().width : 0
    }
    const [sm, md, lg] = [w('sm'), w('md'), w('lg')]
    if (sm && md && lg && !(sm < md && md < lg)) bad.push({ why: 'size-channel-dead', sm, md, lg })

    return { count: document.querySelectorAll("input[type='checkbox']").length, bad: bad.slice(0, 10) }
  },
}
