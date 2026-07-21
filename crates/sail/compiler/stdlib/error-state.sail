<!--
  <error-state> — L1 language element (element mode). The "something broke"
  sibling of <empty-state>: an always-destructive placeholder with a fixed
  alert-circle icon tile, a title, description, an optional error code chip, an
  optional collapsible technical-details section, and a retry action.

  Named slots project at compile time (title / details / action + the default
  description slot). Behavior rides use:errorState: fill the code chip from the
  `code` attribute, gate + drive the details toggle (Show/Hide, aria-expanded),
  and auto-focus the retry action. Bare tag lowers to <div class=sail-error-state>.

  Faithful to the feat/next sail-error-state baseline. L1 divergences: the icon is
  a fixed alert-circle (error states are always destructive) rather than a
  name-lookup prop; title/description come from slots (static fallback "Something
  went wrong"); the code chip is a `code` string attribute filled by the action.
-->

<scope>
  props: {
    variant: 'default' | 'inline' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    code = ''
  }
</scope>

<div use:errorState role="alert">
  <div class="icon" aria-hidden="true">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </div>
  <div class="copy">
    <div class="title"><slot name="title">Something went wrong</slot></div>
    <div class="desc"><slot></slot></div>
    <div class="code"></div>
    <button class="details-toggle" type="button" anatomy aria-expanded="false">Show details</button>
    <div class="details"><slot name="details"></slot></div>
  </div>
  <div class="actions"><slot name="action"></slot></div>
</div>
