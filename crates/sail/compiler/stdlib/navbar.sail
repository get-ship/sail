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

<style>
  /* The DEFAULT placement lives in the BASE rule, not behind [placement='top'].
     An element-mode prop default is not stamped as an attribute, so a bare
     <navbar> carries no placement= at all — and a rule gated on it never
     matches. The bar rendered 34px tall with no padding and no border. Defaults
     belong in the base selector; the other placements override it. */
  .sail-navbar {
    display: flex;
    flex-direction: row;
    align-items: center;
    min-height: 3.5rem;
    padding: var(--space-2) var(--space-4);
    background: var(--background);
    color: var(--foreground);
    gap: var(--space-4);
  }
  .sail-navbar[bordered]         { border-block-end: 1px solid var(--border); }
  .sail-navbar[sticky] {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 100);
  }

  /* ── bottom: the same bar, pinned to the other edge ──────────────────── */
  .sail-navbar[placement='bottom'][bordered] {
    border-block-end: 0;
    border-block-start: 1px solid var(--border);
  }
  .sail-navbar[placement='bottom'][sticky] {
    top: auto;
    bottom: 0;
  }

  /* ── rail: the same bar, on its side ────────────────────────────────── */
  .sail-navbar[placement='rail'] {
    flex-direction: column;
    min-height: 0;
    align-items: stretch;
    gap: var(--space-5);
    padding: var(--space-5) var(--space-4);
    min-width: 14rem;
  }
  .sail-navbar[placement='rail'][bordered] {
    border-block-end: 0;
    border-inline-end: 1px solid var(--border);
  }
  /* A sticky rail under a sticky top bar: the rail yields — one z below, and
     `offset` (the height of the bar above it) keeps it from sliding beneath
     that bar while scrolled.

     `offset` is a SPACE TOKEN, not a length: <navbar placement="rail" sticky
     offset="14">. The bar above is 56px, so the rail starts 56px down and is
     56px shorter. Only the tokens listed below resolve — an unlisted value
     simply does not match, so there is no way to smuggle a raw length in. */
  .sail-navbar[placement='rail'][sticky] {
    position: sticky;
    top: var(--rail-offset, 0px);
    align-self: start;
    height: calc(100vh - var(--rail-offset, 0px));
    overflow-y: auto;
    z-index: calc(var(--z-sticky, 100) - 1);
  }
  .sail-navbar[placement='rail'][offset='8']  { --rail-offset: var(--space-8); }
  .sail-navbar[placement='rail'][offset='10'] { --rail-offset: var(--space-10); }
  .sail-navbar[placement='rail'][offset='12'] { --rail-offset: var(--space-12); }
  .sail-navbar[placement='rail'][offset='14'] { --rail-offset: var(--space-14); }
  .sail-navbar[placement='rail'][offset='16'] { --rail-offset: var(--space-16); }
  .sail-navbar[placement='rail'][offset='20'] { --rail-offset: var(--space-20); }
  .sail-navbar[placement='rail'][offset='24'] { --rail-offset: var(--space-24); }

  /* ── the four regions ───────────────────────────────────────────────── */
  /* A link in a nav bar is chrome wherever it sits — undecorated, ink from
     its region. Zero specificity, so one sx atom on the link overrides it. */
  :where(.sail-navbar a) {
    text-decoration: none;
    color: inherit;
  }
  .sail-nav-brand {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    flex: 0 0 auto;
    font-weight: var(--font-weight-semibold);
  }
  .sail-nav-links {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex: 0 0 auto;
    min-width: 0;
  }
  /* The center region absorbs the bar's spare width, so its content rides the
     center line between the fixed ends. */
  .sail-nav-center {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1 1 auto;
    min-width: 0;
  }
  .sail-navbar[placement='rail'] .sail-nav-links {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-1);
    flex: 1 1 auto;
  }
  .sail-navbar[placement='rail'] .sail-nav-center {
    flex-direction: column;
    flex: 0 0 auto;
  }
  .sail-nav-actions {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    flex: 0 0 auto;
    margin-inline-start: auto;
  }
  .sail-navbar[placement='rail'] .sail-nav-actions {
    flex-direction: column;
    align-items: stretch;
    margin-inline-start: 0;
    margin-block-start: auto;
  }

  /* Links in the links region light up on hover/current. */
  .sail-nav-links a {
    display: block;
    padding: var(--space-1_5) var(--space-2_5);
    border-radius: var(--radius-sm);
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }
  .sail-nav-links a:hover { color: var(--foreground); background: var(--hover-bg); }
  .sail-nav-links a[aria-current] {
    color: var(--foreground);
    background: var(--hover-bg);
    font-weight: var(--font-weight-medium);
  }

  /* ── rail groups: a <label> is a group heading, links carry an accent ─── */
  .sail-navbar[placement='rail'] .sail-nav-links label {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-bold);
    text-transform: var(--label-transform);
    letter-spacing: 0.08em;
    color: var(--muted-foreground);
    margin: var(--space-4) 0 var(--space-1) var(--space-2);
  }
  .sail-navbar[placement='rail'] .sail-nav-links label:first-child { margin-block-start: 0; }
  .sail-navbar[placement='rail'] .sail-nav-links a {
    border-inline-start: 2px solid transparent;
    border-radius: var(--radius-md);
  }
  .sail-navbar[placement='rail'] .sail-nav-links a[aria-current] {
    color: var(--foreground);
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    border-inline-start-color: var(--primary);
    font-weight: var(--font-weight-semibold);
  }
</style>

<nav>
  <div class="sail-nav-brand"><slot name="brand"></slot></div>
  <div class="sail-nav-links"><slot></slot></div>
  <div class="sail-nav-center"><slot name="center"></slot></div>
  <div class="sail-nav-actions"><slot name="actions"></slot></div>
</nav>
