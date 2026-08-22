<!--
  <breadcrumb> — trail host. ol/li/a anatomy + separators via compiler descendant map.
  Host list Style: flex + muted-foreground before skins attach.
  size: type scale on the host so attr selectors match.

  Look: this host owns size Style. Slotted crumbs are author markup. Slots, not :part.
  State: none — no controller, no breadcrumb action.
  Listen: native <a> crumbs + native <nav>. No hotkeys/focus/use:.

  LANGUAGE GAP — separator: 'slash' | 'chevron' | 'dot'
  content() only accepts `empty`. Platform ol > :not(:first-child)::before
  is a fixed '/' glyph; Style cannot switch that string by prop.

  LANGUAGE GAP — collapse
  Hiding middle crumbs needs descendant nth-child or dynamic structure.
  Style when keys stop at first/last; slotted crumbs are author markup.
-->
<scope>
  props: {
    size: 'sm' | 'md' | 'lg' = 'md'
  }

  style list = {display(flex).pad(0).margin(0).fg(muted-foreground).type({ family: body, list: none })}
  // size match must live on the host — attrs stamp on <nav>, not the inner ol.
  style sizeMd = {type({ size: sm })}
  style sizeSm = {type({ size: xs })}
  style sizeLg = {type({ size: base })}
</scope>
<nav label="Breadcrumb" style(sizeMd, {match: {size: { sm: sizeSm, md: sizeMd, lg: sizeLg }}})>
  <ol style(list)>
    <slot></slot>
  </ol>
</nav>
