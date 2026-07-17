<!--
  <popover id="…" placement="bottom"> — L1 anchored overlay (element mode) on the
  NATIVE popover API. Any element with popovertarget="<id>" is the trigger; the
  platform gives open/close, light-dismiss and top-layer for free, and use:anchor
  does the one thing it doesn't: positioning (with a flip when there's no room).
  Bare tag lowers to <div class="sail-popover" popover use:anchor>.

    <button popovertarget="p1">Open</button>
    <popover id="p1" placement="bottom-end">…</popover>
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<style>
  .sail-popover {
    position: fixed;
    inset: auto;
    margin: 0;
    min-width: 12rem;
    max-width: 22rem;
    padding: var(--space-3);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover, 1000);
  }
  .sail-popover::backdrop { background: transparent; }
</style>

<div popover use:anchor><slot></slot></div>
