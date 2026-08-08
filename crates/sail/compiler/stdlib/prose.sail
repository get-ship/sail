<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).
  Host Style + compiler prose descendant map → element-skins (not hand contracts).
-->

<scope>
  style host = {
    box: display(block),
    paint: fg(prose.foreground),
    text: fontFamily(body).leading(loose),
  }
</scope>

<div style(host)>
  <slot></slot>
</div>
