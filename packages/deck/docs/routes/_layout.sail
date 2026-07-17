<!--
  Root layout — wraps every route. deck reads the first custom-element tag
  (<deck-shell>) as the chrome, then mounts each route's content into that
  chrome's <slot name="route">. The header + theme toggle persist across
  client-side navigations; only the route slot swaps.
-->
<sail-shell></sail-shell>
