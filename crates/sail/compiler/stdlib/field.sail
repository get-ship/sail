<!--
  <field> — L1 language element (element mode). A vertical form-field stack:
  label → control → hint / error. Named slots (label / hint / error + the
  default control slot) project at compile time; hint↔error swap on [invalid]
  is pure CSS. Bare tag lowers to <div class="sail-field">.

  Behavior rides use:field: assign ids to the hint/error regions and wire the
  first slotted control's aria-describedby to the active one (hint when valid,
  error when [invalid]) + set aria-invalid. Faithful to the sail-field twin;
  label↔control association (for/id) stays the author's responsibility, exactly
  as the twin.
-->

<scope>
  props: {
    invalid = false
  }
</scope>

<style>
  /* residue lock: layout flex + named slots + [invalid] CSS swap + svg stroke (tokenized; no bare leading/space/ring literals) */
  .sail-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-family: var(--font-body);
  }
  .sail-field[hidden] { display: none; }

  .sail-field .msg { margin-top: var(--space-1); }
  .sail-field .hint,
  .sail-field .error {
    font-size: var(--text-xs);
    line-height: var(--leading-snug);
  }
  .sail-field .hint { color: var(--muted-foreground); }
  .sail-field .hint:empty { display: none; }
  .sail-field .error {
    display: flex;
    align-items: flex-start;
    gap: var(--space-1);
    color: var(--destructive);
    font-weight: var(--font-weight-medium);
  }
  .sail-field .error svg { width: var(--icon-size-sm); height: var(--icon-size-sm); flex-shrink: 0; }

  /* hint shows when valid, error shows when [invalid] — pure CSS swap. */
  .sail-field:not([invalid]) .error { display: none; }
  .sail-field[invalid] .hint { display: none; }
</style>

<div use:field>
  <div class="label"><slot name="label"></slot></div>
  <div class="control"><slot></slot></div>
  <div class="hint msg"><slot name="hint"></slot></div>
  <div class="error msg" role="alert" aria-live="polite">
    <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path><path d="M12 9v4M12 17h.01"></path></svg>
    <span><slot name="error"></slot></span>
  </div>
</div>
