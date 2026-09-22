<!--
  <prose> — typography scope for rendered content (markdown blobs etc.).
  Look: this host owns Style. Slots, not :part. Descendant type map is
  platform/prose.css (native-anatomy) via skins, not hand contracts.
  State: none — no controller, no use:prose.
  Listen: none — slotted native HTML (h1/p/a).
-->

<scope>
  style host = {display(block).fg(foreground).type({ family: body, leading: normal })}
</scope>

<div style(host)>
  <slot></slot>
</div>
