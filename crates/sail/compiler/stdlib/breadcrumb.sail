<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler descendant map.
  Host list Style: flex + muted-foreground before skins attach.
-->
<scope>
  style list = {box: flow({ display: flex }).space({ m: 0, pad: 0 }), paint: ink({ color: muted-foreground }), text: type({ family: body, size: sm, list: none })}
</scope>
<nav label="Breadcrumb">
  <ol style(list)>
    <slot></slot>
  </ol>
</nav>
