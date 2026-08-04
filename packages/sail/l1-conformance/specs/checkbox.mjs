// L1 conformance spec: <input type=checkbox> (mechanism-2, native + skin, zero JS).
// The native control IS the L1 checkbox — it brings the platform's label
// association, form participation, indeterminate state and keyboard for free. The
// skin's whole job is to make :checked / :indeterminate / :disabled legible in
// every theme, which is exactly what the probe asserts.
// Markup mirrors freeform host stamp: sail-input + type=checkbox Style classes.

const CB =
  'sail-input sx-typecheckbox-appearancenone sx-typecheckbox-displayinlineblock sx-typecheckbox-alignmiddle sx-typecheckbox-m0 sx-typecheckbox-pad0 sx-typecheckbox-minH0 sx-typecheckbox-wiconsizesm sx-typecheckbox-hiconsizesm sx-typecheckbox-fontSizebase sx-typecheckbox-borderWidth2 sx-typecheckbox-borderborderstrong sx-typecheckbox-bgbackground sx-typecheckbox-shadowxs sx-typecheckbox-radiussm sx-typecheckbox-cursorpointer sx-typecheckboxchecked-bgprimary sx-typecheckboxchecked-borderprimary sx-typecheckboxdisabled-cursornotallowed sx-typecheckboxdisabled-opacity50 sx-typecheckboxsizexs-wiconsizexs sx-typecheckboxsizexs-hiconsizexs sx-typecheckboxsizesm-wiconsizexs sx-typecheckboxsizesm-hiconsizexs sx-typecheckboxsizemd-wiconsizesm sx-typecheckboxsizemd-hiconsizesm sx-typecheckboxsizelg-wiconsizemd sx-typecheckboxsizelg-hiconsizemd'

const box = (attrs = '') => {
  const m = attrs.match(/class="([^"]*)"/)
  const rest = attrs.replace(/class="[^"]*"\s*/, '').trim()
  const cls = m ? `${CB} ${m[1]}` : CB
  return `<label><input type="checkbox" class="${cls}" ${rest}> Label</label>`
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
    // :indeterminate is a PROPERTY, not an attribute — it has no markup form, so
    // it can only be set here. It's the state most skins forget.
    document.querySelectorAll('#ind').forEach((el) => (el.indeterminate = true))
  },
  probe() {
    const bad = []
    const cs = (el, p) => getComputedStyle(el)[p]
    const clear = (v) => v === 'rgba(0, 0, 0, 0)' || v === 'transparent' || v === ''

    document.querySelectorAll("input[type='checkbox']").forEach((el, i) => {
      const r = el.getBoundingClientRect()
      if (r.width <= 0 || r.height <= 0) return bad.push({ i, why: 'not-rendered' })
      // it must be square — a stretched checkbox is the classic appearance:none bug
      if (Math.abs(r.width - r.height) > 1) bad.push({ i, why: 'not-square', w: r.width, h: r.height })
      // hit target: the box is small by design (it's sized in em off the shared
      // --size-fs channel), and the <label> carries the real target area — so the
      // floor here is legibility, not WCAG's 24px pointer minimum.
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

    // checked must not merely be "the same box" — it carries the accent fill
    if (cs(base, 'backgroundColor') === cs(on, 'backgroundColor'))
      bad.push({ why: 'checked-not-filled', base: cs(base, 'backgroundColor') })
    // indeterminate is its OWN state — distinct from both off and on
    if (cs(ind, 'backgroundColor') === cs(base, 'backgroundColor'))
      bad.push({ why: 'indeterminate-looks-unchecked' })
    if (cs(off, 'opacity') === cs(base, 'opacity') && cs(off, 'backgroundColor') === cs(base, 'backgroundColor'))
      bad.push({ why: 'disabled-not-distinct' })
    // The box rides the [size] channel — the skin's own claim (it's sized in em
    // off --size-fs, shared with button/input). If that's true it's monotonic.
    const w = (s) => q(`input[type='checkbox'][size='${s}']`).getBoundingClientRect().width
    const [sm, md, lg] = [w('sm'), w('md'), w('lg')]
    if (!(sm < md && md < lg)) bad.push({ why: 'size-channel-dead', sm, md, lg })
    // frozen-token guard
    const mark = (t) => cs(document.querySelector(`[theme='${t}'] .l1-mark`), 'borderTopColor')
    if (mark('ship') === mark('console')) bad.push({ why: 'frozen-token', light: mark('ship') })

    return { count: document.querySelectorAll("input[type='checkbox']").length, bad: bad.slice(0, 10) }
  },
}
