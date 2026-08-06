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
  const icon = style.size(icon-size-sm)
</scope>

<column gap="1" use:describedControl>
  style(host)

  <div><slot name="label"></slot></div>
  <div><slot></slot></div>
  <div>
    style([
    hint,
    when({
      empty: style.display(none),
      group-invalid: style.display(none),
    }),
  ])

    <slot name="hint"></slot>
  </div>
  <div>
    style([
    style.display(none),
    when({
      group-invalid: style.display(contents),
    }),
  ])

    <row align="start" gap="1" role="alert" live="polite">
      style(error)

      <svg viewBox="0 0 24 24" hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" shrink="0">
        style(icon)

        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
