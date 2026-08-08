<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler descendant map.
  Host list Style: flex + component-token fg before skins attach.
-->
<scope>
  style list = {
    box: display(flex).gap(2).m(0).pad(0),
    paint: list(none).fg(breadcrumb.foreground),
    text: fontFamily(body).fontSize(sm),
  }
</scope>
<nav label="Breadcrumb">
  <ol style(list)>
    <slot></slot>
  </ol>
</nav>
