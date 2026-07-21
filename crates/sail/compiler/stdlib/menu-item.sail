<!--
  <menu-item value="…"> — an item inside <dropdown> / <context-menu>. role=menuitem,
  roving focus, disabled handling and select-then-close are wired by use:menu on
  the parent. Bare tag lowers to <button class="sail-menu-item" type="button">.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<style>
  .sail-menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2_5);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    line-height: var(--leading-tight);
    color: var(--foreground);
    background: transparent;
    border: 0;
    border-radius: var(--radius-sm);
    text-align: start;
    cursor: pointer;
    white-space: nowrap;
  }
  .sail-menu-item:hover:not([disabled]) { background: var(--muted); }
  .sail-menu-item:focus-visible { background: var(--muted); outline: var(--ring-width) solid var(--ring); outline-offset: -1px; }
  .sail-menu-item[disabled] { opacity: 0.5; cursor: not-allowed; }
</style>

<button type="button"><slot></slot></button>
