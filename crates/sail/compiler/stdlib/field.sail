<!--
  <field> — L1 language element (element mode). A vertical form-field stack:
  label → control → hint / error. Named slots (label / hint / error + the
  default control slot) project at compile time; hint↔error swap on [invalid]
  is pure CSS. Bare tag lowers to a layout column (`.sail-field`).

  Host paint: layout owns stack+gap (`<column gap="1">`); typeface is sx
  (`font-body`).

  Behavior rides use:describedControl: assign ids to the hint/error regions and
  wire the first slotted control's aria-describedby to the active one (hint when
  valid, error when [invalid]) + set aria-invalid. Faithful to the sail-field
  twin; label↔control association (for/id) stays the author's responsibility,
  exactly as the twin.
-->

<scope>
  props: {
    invalid = false
  }
</scope>

<column gap="1" sx="font-body" use:describedControl>
  <div class="label"><slot name="label"></slot></div>
  <div class="control"><slot></slot></div>
  <div class="hint msg"><slot name="hint"></slot></div>
  <div class="error msg" role="alert" aria-live="polite">
    <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"></path><path d="M12 9v4M12 17h.01"></path></svg>
    <span><slot name="error"></slot></span>
  </div>
</column>
