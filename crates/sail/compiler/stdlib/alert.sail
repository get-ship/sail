<!--
  <alert> — L1 language element (element mode). An inline status banner: a
  tone-colored surface with an optional leading icon, a title, body copy,
  action buttons, and an optional dismiss control. Bare tag lowers to
  <div class="sail-alert">; the skin routes to the framework @layer.

  This is the first L1 element to use COMPILE-TIME NAMED-SLOT PROJECTION: the
  icon / title / actions regions are named slots (`slot="icon"` etc.), and the
  default slot is the body. A use site distributes its slotted children into
  the matching region at compile time — zero runtime, still compiles away.

  Behavior rides stacked capability use: mix-ins
  (packages/sail/runtime/src/actions/live-region.ts + dismiss-control.ts):
  - use:liveRegion — role/aria-live upgrade to assertive alert for
    warning/destructive tones (sensible static defaults — status/polite —
    mean it announces correctly even before the action hydrates);
  - use:dismissControl — the dismiss button (shown only under [dismissible])
    fires `sail-dismiss` and removes the banner.

  Faithful to the feat/next sail-alert baseline. Two shadow-isms dissolve at L1:
  the reactive role/aria-live (a controller effect) becomes use:liveRegion,
  and the bare `<button ghost xs>` (which rendered unskinned across the
  shadow boundary — the baseline's LANG-GAP) becomes a self-contained close
  skin authored here, where L1 is allowed to own CSS.
-->

<scope>
  props: {
    tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'muted' = 'default'
    dismissible = false
  }
</scope>

<div use:liveRegion use:dismissControl role="status" aria-live="polite" aria-atomic="true">
  <span class="ico"><slot name="icon"></slot></span>
  <div class="content">
    <div class="title"><slot name="title"></slot></div>
    <div class="body"><slot></slot></div>
    <div class="actions"><slot name="actions"></slot></div>
  </div>
  <button class="close" type="button" anatomy aria-label="Dismiss">
    <svg viewBox="0 0 10 10" aria-hidden="true">
      <path d="M2 2 L8 8 M8 2 L2 8" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" fill="none"></path>
    </svg>
  </button>
</div>
