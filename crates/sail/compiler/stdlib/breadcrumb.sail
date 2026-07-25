<!--
  <breadcrumb> — list trail host Style; li/a anatomy via compiler descendant map.
-->
<scope>
  const list = style.m(0).pad(0).list(none).fontFamily(body).fontSize(sm)
</scope>
<nav aria-label="Breadcrumb">
  <ol sx={list}>
    <slot></slot>
  </ol>
</nav>
