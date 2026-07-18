<!--
  Root layout — wraps every route. deck reads the first custom-element tag
  (<sail-site-header>) as the chrome, then mounts each route's content into
  that chrome's <slot name="route">. A unique tag (not <sail-shell>) avoids
  colliding with the deck docs site's shell in the shared workspace build.

  This layout owns only the always-on top bar (brand, product switcher,
  theme picker) — see src/sail-site-header/. Each product segment
  (sail/deck/components/ai) nests its own `_layout.sail` + sidebar rail
  inside this one; the two compose via deck's layout-chain mounting.
-->
<sail-site-header></sail-site-header>
