<!--
  <toggle value="…"> — a toggle cell inside <toggle-group>. role=button,
  aria-pressed, and roving tabindex are wired by use:toggleGroup on the parent.
  Pressed = inverted fill (theme-safe, high-contrast). Bare tag lowers to
  <button class="sail-toggle" type="button">.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<style>
  .sail-toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    height: var(--tg-h, var(--control-h-md));
    padding: 0 var(--tg-px, var(--space-3));
    font-family: var(--font-body);
    font-size: var(--tg-fs, var(--text-sm));
    font-weight: var(--font-weight-medium);
    line-height: var(--leading-none);
    color: var(--foreground);
    background: var(--background);
    border: 1px solid var(--border);
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }
  .sail-toggle:hover:not([disabled]):not([aria-pressed='true']) { background: var(--muted); }
  .sail-toggle[aria-pressed='true'] {
    color: var(--background);
    background: var(--foreground);
    border-color: var(--foreground);
    position: relative;
    z-index: 1;
  }
  .sail-toggle[disabled] { opacity: 0.5; cursor: not-allowed; }
  .sail-toggle:focus-visible {
    outline: var(--ring-width) solid var(--ring);
    outline-offset: 1px;
    position: relative;
    z-index: 2;
  }
</style>

<button type="button"><slot></slot></button>
