<!--
  <navbar> — the application nav bar. One element, three placements.

    <navbar sticky bordered>                     top bar (default)
      <brand-mark slot="brand" />
      <a href="/">Home</a> <a href="/docs">Docs</a>
      <input slot="center" type="search" />
      <button slot="actions">Sign in</button>
    </navbar>

    <navbar placement="rail" bordered>…</navbar>   vertical rail (sidebar)
    <navbar placement="bottom">…</navbar>          bottom bar (mobile)

  A rail is not a different component — it is a navbar on its side. `bordered`
  puts the hairline on the edge that faces the content, and `sticky` pins it to
  the edge it sits on, so both follow from the placement rather than from three
  separate elements with three sets of props.

  Four regions, no looks: brand (start), the default slot (links, beside the
  brand), center (whatever belongs on the bar's center line — a search box, a
  product switcher), actions (end). The element owns placement and link chrome;
  what a centered control looks like is the author's sentence to write, in sx.
-->

<scope>
  props: {
    placement: 'top' | 'rail' | 'bottom' = 'top'
    sticky = false
    bordered = false
  }
</scope>

<nav>
  <div class="sail-nav-brand"><slot name="brand"></slot></div>
  <div class="sail-nav-links"><slot></slot></div>
  <div class="sail-nav-center"><slot name="center"></slot></div>
  <div class="sail-nav-actions"><slot name="actions"></slot></div>
</nav>
