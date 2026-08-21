<!--
  <field> — stack+gap layout; hint/error gates via structure Style.
  Hint/error paint from input / foreground / border.

  Look: host + hint/error Style owns paint — no DOM.
  State: this file's <controller> (invalid + ids → aria-describedby/invalid).
-->

<scope>
  props: {
    invalid = false
    fill = true
  }

  style host = {group().type({ family: body })}
  style hint = {margin({ t: 1 }).fg(muted-foreground).type({ size: xs, leading: snug })}
  style error = {margin({ t: 1 }).fg(danger).type({ size: xs, leading: snug, weight: medium })}
  style icon = {size(icon-size-sm)}
</scope>

<controller>
export default controller((init) => {
  function read(v) { return typeof v === 'function' ? v() : v }
  const gen = `sail-field-${Math.random().toString(36).slice(2, 10)}`
  const hintId = derived(() => String(read(init.id) ?? '').trim() || `${gen}-hint`)
  const errorId = derived(() => String(read(init.id) ?? '').trim() ? `${String(read(init.id)).trim()}-error` : `${gen}-error`)
  const invalid = derived(() => {
    const v = read(init.invalid)
    return v === true || v === '' || v === 'true'
  })
  const describedBy = derived(() => invalid() ? errorId() : hintId())
  return { hintId, errorId, invalid, describedBy }
})
</controller>

<column gap="1" aria-describedby={describedBy()} aria-invalid={invalid()} style(host)>
  <div><slot name="label"></slot></div>
  <div><slot></slot></div>
  <div id={hintId()} style(hint, {when: {empty: display(none), group-invalid: display(none)}})>
    <slot name="hint"></slot>
  </div>
  <div style({display(none), when: {group-invalid: display(contents)}})>
    <row align="start" gap="1" id={errorId()} role="alert" aria-live="polite" data-field-error style(error)>
      <svg viewBox="0 0 24 24" hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style(icon, {shrink(0)})>
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
