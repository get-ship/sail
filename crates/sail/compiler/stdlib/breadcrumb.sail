<!--
  <breadcrumb> — list trail host Style; li/a anatomy via compiler descendant map.
-->
<scope>
  const list = style({ box: m(0).pad(0), paint: list(none), text: fontFamily(body).fontSize(sm) })
</scope>
<nav>
  <ol>
    style(list)

    <slot></slot>
  </ol>
</nav>
