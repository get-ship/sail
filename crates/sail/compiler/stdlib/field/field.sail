<!--
  <field> — stack+gap layout; hint/error gates via structure Style.
  Hint/error paint from input / foreground / border; control surface shared with
  input/select/textarea via input / foreground / border.
-->

<scope>
  props: {
    invalid = false
  }

  style host = {group().type({ family: body })}
  style hint = {margin({ t: 1 }).fg(muted-foreground).type({ size: xs, leading: snug })}
  style error = {margin({ t: 1 }).fg(destructive).type({ size: xs, leading: snug, weight: medium })}
  style icon = {size(icon-size-sm)}
</scope>

<column gap="1" use:describedControl style(host)>
  <div><slot name="label"></slot></div>
  <div><slot></slot></div>
  <div style(hint, {when: {empty: display(none), group-invalid: display(none)}})>
    <slot name="hint"></slot>
  </div>
  <div style({display(none), when: {group-invalid: display(contents)}})>
    <!-- alert role + polite live stamped by use:describedControl (closed anatomy) -->
    <row align="start" gap="1" data-field-error style(error)>
      <svg viewBox="0 0 24 24" hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style(icon, {shrink(0)})>
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
