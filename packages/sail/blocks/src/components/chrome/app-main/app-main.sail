<!--
  app-main — leftover pane next to a rail. L2 pad/size. Two consumers
  (settings + dashboard). Not a pane. Identities: presets only.
-->
<scope>
  style base = { size(grow), pad(page) }
</scope>

<column gap="6" style(base)>
  <slot />
</column>
