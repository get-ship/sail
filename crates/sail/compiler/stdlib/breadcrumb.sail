<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler descendant map.
  Host list Style: flex + component-token fg before skins attach.
-->
<scope>
  style list = {box: flow({ display: flex }).space({ m: 0, pad: 0 }), paint: ink({ color: breadcrumb.foreground }), text: type({ family: body, size: sm, list: none })}
</scope>
<nav label="Breadcrumb">
  <ol style(list)>
    <slot></slot>
  </ol>
</nav>
