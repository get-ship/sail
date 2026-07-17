<!--
  <context-menu for="…"> — L1 right-click menu (element mode). Same native
  [popover] + menu semantics as <dropdown>, but opened at the pointer on
  contextmenu of the target (`for` names it; defaults to the parent element).
  Bare tag lowers to <div class="sail-context-menu" popover use:contextMenu use:menu>.

    <div id="canvas">right-click me</div>
    <context-menu for="canvas">
      <menu-item value="cut">Cut</menu-item>
    </context-menu>
-->

<scope>
  props: {
    for = ''
  }
</scope>

<style>
  .sail-context-menu {
    position: fixed;
    inset: auto;
    margin: 0;
    min-width: 12rem;
    padding: var(--space-1);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover, 1000);
  }
  .sail-context-menu::backdrop { background: transparent; }
</style>

<div popover use:contextMenu use:menu><slot></slot></div>
