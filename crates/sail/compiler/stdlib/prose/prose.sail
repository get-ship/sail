<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).
  Host Style + compiler prose descendant map → element-skins (not hand contracts).
-->

<scope>
  style host = {display(block).fg(foreground).type({ family: body, leading: normal })}
</scope>

<div style(host)>
  <slot></slot>
</div>
