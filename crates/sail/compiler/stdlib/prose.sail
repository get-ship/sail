<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).
  Host Style + compiler prose descendant map → element-skins (not hand contracts).
-->

<scope>
  style host = {box: flow({ display: block }), paint: ink({ color: foreground }), text: type({ family: body, leading: loose })}
</scope>

<div style(host)>
  <slot></slot>
</div>
