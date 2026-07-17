<!--
  <tab name="…"> — a tab button inside <tab-list>. role=tab, aria-selected,
  roving tabindex, and the active state are wired by use:tabs on the parent
  <tabs>. The line variant paints a tone-colored underline when active. Bare tag
  lowers to <button class="sail-tab" type="button">.
-->

<scope>
  props: {
    name = ''
    disabled = false
  }
</scope>

<style>
  .sail-tab {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--tab-py, var(--space-2)) var(--tab-px, var(--space-3));
    font-family: var(--font-body);
    font-size: var(--tab-fs, var(--text-sm));
    font-weight: var(--font-weight-medium);
    line-height: 1;
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    -webkit-tap-highlight-color: transparent;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .sail-tab:hover:not([disabled]) { color: var(--foreground); }
  .sail-tab[aria-selected='true'] {
    color: var(--foreground);
    font-weight: var(--font-weight-semibold);
    border-bottom-color: var(--tone, var(--primary));
  }
  .sail-tab[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sail-tab:focus-visible {
    outline: var(--ring-width) solid var(--tone-ring, var(--ring));
    outline-offset: calc(-1 * var(--ring-width));
  }
</style>

<button type="button"><slot></slot></button>
