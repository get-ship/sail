<!--
  app-main — leftover pane next to a rail. L2 pad/size. Two consumers
  (settings + dashboard). Not a pane. Identities: presets only.
-->
<scope>
  style insetMd = { pad(page-md) }
  style insetLg = { pad(page-lg) }
  style base = { size(grow), pad(page), when: { tablet: insetMd, desktop: insetLg } }
</scope>

<column gap="6" style(base)>
  <slot />
</column>
