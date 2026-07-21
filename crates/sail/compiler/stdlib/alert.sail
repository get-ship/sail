<!--
  <alert> — L1 language element (element mode). An inline status banner: a
  tone-colored surface with an optional leading icon, a title, body copy,
  action buttons, and an optional dismiss control. Bare tag lowers to
  <div class="sail-alert">; the skin routes to the framework @layer.

  This is the first L1 element to use COMPILE-TIME NAMED-SLOT PROJECTION: the
  icon / title / actions regions are named slots (`slot="icon"` etc.), and the
  default slot is the body. A use site distributes its slotted children into
  the matching region at compile time — zero runtime, still compiles away.

  Behavior rides use:alert (packages/sail/runtime/src/actions/alert.ts):
  - role/aria-live upgrade to assertive alert for warning/destructive tones
    (sensible static defaults — status/polite — mean it announces correctly
    even before the action hydrates);
  - the dismiss button (shown only under [dismissible]) fires `sail-dismiss` and
    removes the banner.

  Faithful to the feat/next sail-alert baseline. Two shadow-isms dissolve at L1:
  the reactive role/aria-live (a controller effect) becomes the use:alert
  action, and the bare `<button ghost xs>` (which rendered unskinned across the
  shadow boundary — the baseline's LANG-GAP) becomes a self-contained close
  skin authored here, where L1 is allowed to own CSS.
-->

<scope>
  props: {
    tone: 'default' | 'info' | 'success' | 'warning' | 'destructive' | 'muted' = 'default'
    dismissible = false
  }
</scope>

<style>
  .sail-alert {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: var(--space-3);
    position: relative;
    overflow: hidden;
    padding: var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--surface-1);
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    box-shadow: var(--shadow-xs);
    /* Local channel for shared icon/close control size (optional anatomy
       residue lock; collapses duplicate var(--control-h-xs)). */
    --alert-ico: var(--control-h-xs);
  }
  .sail-alert[hidden] { display: none; }

  /* Tone: muted default (specificity 0,2,0) so the GLOBAL [tone='X'] channel
     fills every branded tone. --tone drives the leading stripe; --tone-bg/-fg
     the icon chip. */
  .sail-alert:not([tone]),
  .sail-alert[tone='default'] {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-bg: var(--tone-muted-bg);
  }

  /* Leading tone stripe (anchored to the relative root). */
  .sail-alert::before {
    content: '';
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: var(--space-1);
    background: var(--tone);
  }

  .sail-alert .ico {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--alert-ico);
    height: var(--alert-ico);
    border-radius: var(--radius-md);
    background: var(--tone-bg);
    color: var(--tone-fg);
  }
  .sail-alert .ico:empty { display: none; }
  .sail-alert .ico svg,
  .sail-alert .ico ::slotted(*) { width: var(--icon-size-sm); height: var(--icon-size-sm); }

  .sail-alert .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .sail-alert .title { font-weight: var(--font-weight-semibold); }
  .sail-alert .title:empty { display: none; }
  .sail-alert .body { color: var(--muted-foreground); }
  .sail-alert .actions {
    display: flex;
    flex-direction: row;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }
  .sail-alert .actions:empty { display: none; }

  /* Self-contained dismiss control — hidden until [dismissible]. */
  .sail-alert .close {
    display: none;
    align-items: center;
    justify-content: center;
    align-self: start;
    width: var(--alert-ico);
    height: var(--alert-ico);
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--muted-foreground);
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .sail-alert[dismissible] .close { display: inline-flex; }
  .sail-alert .close:hover { background: var(--hover-bg); color: var(--foreground); }
  .sail-alert .close:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }
  .sail-alert .close svg { width: var(--icon-size-sm); height: var(--icon-size-sm); }

  @media (forced-colors: active) {
    .sail-alert { border-color: CanvasText; }
    .sail-alert::before { background: CanvasText; }
  }
</style>

<div use:alert role="status" aria-live="polite" aria-atomic="true">
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
