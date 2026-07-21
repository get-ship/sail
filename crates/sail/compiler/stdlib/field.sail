<!--
  <field> — stack+gap layout; hint/error gates via structure sx.
-->
<scope>
  props: {
    invalid = false
  }
</scope>
<column gap="1" sx="font-body group" use:describedControl>
  <div class="label"><slot name="label"></slot></div>
  <div class="control"><slot></slot></div>
  <div class="hint msg" sx="mt-1 text-xs leading-snug text-muted-foreground empty:hidden group-invalid:hidden">
    <slot name="hint"></slot>
  </div>
  <div class="error-gate" sx="hidden group-invalid:contents">
    <row align="start" gap="1" class="error msg" role="alert" aria-live="polite" sx="mt-1 text-xs leading-snug text-destructive font-medium">
      <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" sx="size-icon-size-sm shrink-0">
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path>
        <path d="M12 9v4M12 17h.01"></path>
      </svg>
      <span><slot name="error"></slot></span>
    </row>
  </div>
</column>
