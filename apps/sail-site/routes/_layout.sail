<!--
  Root layout — wraps every route. deck reads the first custom-element tag
  (<sail-site-shell>) as the chrome, then mounts each route's content into that
  chrome's <slot name="route">. A unique tag (not <sail-shell>) avoids colliding
  with the deck docs site's shell in the shared workspace build.
-->
<sail-site-shell></sail-site-shell>
