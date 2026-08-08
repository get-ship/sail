<!--
  <field> — stack+gap layout; hint/error gates via structure Style.
  Paint: hint/error from component.field tokens; control surface (bg/fg/border)
  is shared with input/select/textarea via field / field-foreground / field-border.
-->

<scope>
  props: {
    invalid = false
  }

  const host = style({ paint: group(), text: fontFamily(body) })
  const hint = style({ box: mt(1), paint: fg(field-hint), text: fontSize(xs).leading(snug) })
  const error = style({ box: mt(1), paint: fg(field-error), text: fontSize(xs).leading(snug).fontWeight(medium) })
  const icon = style({ box: size(icon-size-sm) })
</scope>

<column gap="1" use:describedControl style(host)>
  <div><slot name="label"></slot></div>
  <div><slot></slot></div>
  <div style(hint, { when: { empty: { box: display(none) }, group-invalid: { box: display(none) } } })>
    <slot name="hint"></slot>
  </div>
  <div style({ box: display(none), when: { group-invalid: { box: display(contents) } } })>
    <!-- alert role + polite live stamped by use:describedControl (closed anatomy) -->
    <row align="start" gap="1" data-field-error style(error)>
      <svg viewBox="0 0 24 24" hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style(icon, { box: shrink(0) })>
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
