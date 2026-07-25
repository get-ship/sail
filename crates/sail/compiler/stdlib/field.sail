<!--
  <field> — stack+gap layout; hint/error gates via structure Style.
  Paint: hint/error colors from component.field tokens (cssNames field-hint, field-error).
-->

<scope>
  props: {
    invalid = false
  }

  const host = style.fontFamily(body).group()
  const hint = style.mt(1).fontSize(xs).leading(snug).fg(field-hint)
  const error = style.mt(1).fontSize(xs).leading(snug).fg(field-error).fontWeight(medium)
  const icon = style.size(icon-size-sm).shrink(0)
</scope>

<column gap="1" sx={host} use:describedControl>
  <div class="label"><slot name="label"></slot></div>
  <div class="control"><slot></slot></div>
  <div class="hint msg" sx={[
    hint,
    when({
      empty: style.display(none),
      group-invalid: style.display(none),
    }),
  ]}>
    <slot name="hint"></slot>
  </div>
  <div class="error-gate" sx={[
    style.display(none),
    when({
      group-invalid: style.display(contents),
    }),
  ]}>
    <row align="start" gap="1" class="error msg" role="alert" aria-live="polite" sx={error}>
      <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx={icon}>
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
