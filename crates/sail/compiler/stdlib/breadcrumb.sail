<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler
  descendant map (stdlib BREADCRUMB_MAP); host list Style so the trail is
  flex + muted even before skins attach.
-->
<scope>
  const list = style({
    box: display(flex).gap(2).m(0).pad(0),
    paint: list(none).fg(muted-foreground),
    text: fontFamily(body).fontSize(sm),
  })
</scope>
<nav label="Breadcrumb">
  <ol>
    style(list)

    <slot></slot>
  </ol>
</nav>
