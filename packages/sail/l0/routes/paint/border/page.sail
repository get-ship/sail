<!--
  Look: a painted box whose stroke spends every border slot.
  Spelling: border({ color, width, style, edge, kind, offset, gradient })
  Owner: language (border slot table)
  Default: kind stroke · edge all · composites subtle / default / strong
  Trap: border({ radius: md }) — radius is not a slot
-->
<column gap="6">
  <l0-case look="color — primary · accent · muted via ref" spell="border color: ref primary | accent | muted">
    <row gap="3">
      <l0-border-face kind="color-primary"></l0-border-face>
      <l0-border-face kind="color-accent"></l0-border-face>
      <l0-border-face kind="color-muted"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="width — hairline · 1 · 2 · 4 · 8 · control-stroke · ring-width" spell="border width: hairline | 1 | 2 | 4 | 8 | control-stroke | ring-width">
    <row gap="3">
      <l0-border-face kind="width-hairline"></l0-border-face>
      <l0-border-face kind="width-1"></l0-border-face>
      <l0-border-face kind="width-2"></l0-border-face>
      <l0-border-face kind="width-4"></l0-border-face>
      <l0-border-face kind="width-8"></l0-border-face>
      <l0-border-face kind="width-control"></l0-border-face>
      <l0-border-face kind="width-ring"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="style — solid · dashed · dotted · double · hidden · none" spell="border style: solid | dashed | dotted | double | hidden | none">
    <row gap="3">
      <l0-border-face kind="style-solid"></l0-border-face>
      <l0-border-face kind="style-dashed"></l0-border-face>
      <l0-border-face kind="style-dotted"></l0-border-face>
      <l0-border-face kind="style-double"></l0-border-face>
      <l0-border-face kind="style-hidden"></l0-border-face>
      <l0-border-face kind="style-none"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="edge — all · x · y · t · r · b · l · s · e" spell="border edge: all | x | y | t | r | b | l | s | e">
    <row gap="3">
      <l0-border-face kind="edge-all"></l0-border-face>
      <l0-border-face kind="edge-x"></l0-border-face>
      <l0-border-face kind="edge-y"></l0-border-face>
      <l0-border-face kind="edge-t"></l0-border-face>
      <l0-border-face kind="edge-r"></l0-border-face>
      <l0-border-face kind="edge-b"></l0-border-face>
      <l0-border-face kind="edge-l"></l0-border-face>
      <l0-border-face kind="edge-s"></l0-border-face>
      <l0-border-face kind="edge-e"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="kind — stroke · outline · ring · image" spell="border kind: stroke | outline | ring | image">
    <row gap="3">
      <l0-border-face kind="kind-stroke"></l0-border-face>
      <l0-border-face kind="kind-outline"></l0-border-face>
      <l0-border-face kind="kind-ring"></l0-border-face>
      <l0-border-face kind="kind-image"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="offset — 0 · 1 · 2 · ring-offset · ring-inset on outline" spell="border offset: 0 | 1 | 2 | ring-offset | ring-inset · kind outline">
    <row gap="3">
      <l0-border-face kind="offset-0"></l0-border-face>
      <l0-border-face kind="offset-1"></l0-border-face>
      <l0-border-face kind="offset-2"></l0-border-face>
      <l0-border-face kind="offset-ring"></l0-border-face>
      <l0-border-face kind="offset-inset"></l0-border-face>
    </row>
  </l0-case>
  <l0-case look="gradient — primary · accent · surface · none" spell="border gradient: primary | accent | surface | none">
    <row gap="3">
      <l0-border-face kind="grad-primary"></l0-border-face>
      <l0-border-face kind="grad-accent"></l0-border-face>
      <l0-border-face kind="grad-surface"></l0-border-face>
      <l0-border-face kind="grad-none"></l0-border-face>
    </row>
  </l0-case>
</column>
