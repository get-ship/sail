<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler descendant map.
  Host list Style: flex + muted-foreground before skins attach.
-->
<scope>
  style list = {display(flex).pad(0).margin(0).fg(muted-foreground).type({ family: body, size: sm, list: none })}
</scope>
<nav label="Breadcrumb">
  <ol style(list)>
    <slot></slot>
  </ol>
</nav>
