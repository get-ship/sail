<!--
  HeaderSimple — brand, a few links, one action. Not a page.
  Head-to-head: https://ui.mantine.dev/component/header-simple/
  Recorded: Features /about, Pricing /pricing, Learn /learn, Community /community,
  button "Toggle navigation" (Mantine Burger). Brand is the Mantine logo.
-->
<navbar bordered>
  <strong slot="brand">Mantine</strong>
  <row slot="center" gap="2">
    <a href="/about">Features</a>
    <a href="/pricing">Pricing</a>
    <a href="/learn">Learn</a>
    <a href="/community">Community</a>
  </row>
  <button slot="actions" variant="ghost" size="icon" label="Toggle navigation">
    <icon name="menu"></icon>
  </button>
</navbar>
