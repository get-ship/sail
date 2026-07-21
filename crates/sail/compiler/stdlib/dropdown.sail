<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  A native [popover] carrying menu semantics: use:anchor positions it, use:menu
  gives role=menu + roving keyboard (Arrow/Home/End), Escape-to-close, first-item
  focus on open, and close-on-select (`sail-menu-select`). Compose <menu-item>s.
  Bare tag lowers to <div class="sail-dropdown" popover use:anchor use:menu>.

    <button popovertarget="m1">Actions</button>
    <dropdown id="m1">
      <menu-item value="edit">Edit</menu-item>
      <menu-item value="del" disabled>Delete</menu-item>
    </dropdown>
-->

<scope>
  props: {
    placement = ''
  }
</scope>

<style>
  .sail-dropdown {
    position: fixed;
    inset: auto;
    margin: 0;
    left: var(--anchor-left);
    top: var(--anchor-top);
    min-width: 12rem; /* residue — 12rem/22rem NOT equal to panel-min/max tokens (180/320px); leave rem */
    padding: var(--space-1);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover, 1000);
  }
  .sail-dropdown::backdrop { background: transparent; }
</style>

<div popover use:anchor use:menu><slot></slot></div>
