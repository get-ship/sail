<!--
  <hover-card id="…" placement="top"> — L1 hover-preview overlay (element mode).
  A native [popover] opened on hover/focus of its trigger (popovertarget) with
  open/close delays so it doesn't flicker; use:anchor positions it. Optional
  `open-delay` / `close-delay` (ms) attributes tune the dwell.
  Bare tag lowers to <div class="sail-hover-card" popover use:anchor use:hoverCard>.

    <a href="#" popovertarget="hc1">@ada</a>
    <hover-card id="hc1" placement="top">…profile card…</hover-card>
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<style>
  .sail-hover-card {
    position: fixed;
    inset: auto;
    margin: 0;
    left: var(--anchor-left);
    top: var(--anchor-top);
    min-width: 14rem;
    max-width: 20rem;
    padding: var(--space-3);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover, 1000);
  }
  .sail-hover-card::backdrop { background: transparent; }
</style>

<div popover use:anchor use:hoverCard><slot></slot></div>
