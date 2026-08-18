<!--
  Registered CE is src/layout-participate/.
  Look: grow / hug / max per host.
        row: hug child + grow leftover; hug child + max stretch.
        column: hug child + grow leftover (host is 16 tall).
        grid: hug cell + grow cell.
        center: hug child.
        stack: hug layer over a muted box.
  Spelling: size(hug) / size(grow) / size(max) on children of
            <row> <column> <grid> <center> <stack>. Paint with named bg only.
  Owner: language (participation)
  Default: hug
  Trap: fixtures/grow-outside-host.bad.sail — `size(grow)` shares leftover inside a host (`<row>` `<column>` `<grid>` `<center>` `<stack>`). This element is not a child of a host.
        Quoted below. Not compiled on this page.

        <column style({ size(grow) })>x</column>
-->
<scope>
  style hugged = { size(hug), bg(primary) }
  style leftover = { size(grow), bg(accent) }
  style stretched = { size(max), bg(muted) }
  style tall = { size({ h: 16 }) }
  style layer = { size({ w: 8, h: 8 }), bg(muted) }
</scope>
<column gap="4">
  <row>
    <column style(hugged)>hug</column>
    <column style(leftover)>grow</column>
  </row>
  <row>
    <column style(hugged)>hug</column>
    <column style(stretched)>max</column>
  </row>
  <column style(tall)>
    <row style(hugged)>hug</row>
    <row style(leftover)>grow</row>
  </column>
  <grid cols="2" gap="2">
    <column style(hugged)>hug</column>
    <column style(leftover)>grow</column>
  </grid>
  <center>
    <column style(hugged)>hug</column>
  </center>
  <stack>
    <column style(layer)></column>
    <column style(hugged)>hug</column>
  </stack>
</column>
