<!--
  <form> — native form. Native-kind: the tag is the host, children stay
  the author's. Column integrity is the author's <column>. Fields stay
  L1 <field>. Not a pane.

  layout=measure (default) — reading column. Auth / GetInTouch.
  layout=fill — leftover width, no center. Settings Profile / Billing.
-->

<scope>
  props: {
    name = ''
    layout: 'measure' | 'fill' = 'measure'
  }

  style base = {display(block).size({ w: 38ch, max-w: 100% }).margin({ x: auto })}
  style fill = {size({ w: 100% })}
</scope>

<form style(base, {match: {layout: {fill: fill}}})>
  <slot></slot>
</form>
