<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).
  Host Style + compiler prose descendant map → element-skins (not hand contracts).

  Paint: fg from component.prose token (cssName prose-foreground).
-->
<scope>
  const host = style.display(block).fontFamily(body).fg(prose-foreground).leading(normal-plus)
</scope>
<div sx={host}><slot></slot></div>
