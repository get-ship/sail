<!--
  <caption> — table caption. Style product only.
-->

<scope>
  props: {}

  style base = {pad({ y: 2, x: 3 }).fg(muted-foreground).type({ align: start })}
</scope>

<caption style(base)>
  <slot></slot>
</caption>
