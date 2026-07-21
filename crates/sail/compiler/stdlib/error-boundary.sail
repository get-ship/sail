<!--
  <error-boundary> — L1 language element (element mode). RENDER-FAULT ISOLATION.

  Wraps a subtree. If a render expression, a controller `mount()`, or a reactive
  effect inside it throws, the throw is routed here (the runtime walks up the DOM
  from the fault site via `findNearestBoundary`), the protected content is pulled
  out of view, and the fallback takes its place. Innermost boundary wins, exactly
  like React: the walk stops at the first ancestor carrying a handle.

    <error-boundary message="This panel could not load.">
      <risky-widget></risky-widget>
    </error-boundary>

  Supply your own fallback with the named slot; otherwise the built-in one
  renders (destructive tone, message, Retry):

    <error-boundary>
      <risky-widget></risky-widget>
      <div slot="fallback">…your markup…</div>
    </error-boundary>

  Recovery: the built-in Retry button resets the boundary, as does `el.reset()`.
  A reset clears the caught state and the content returns.

  Paint (one path):
    host sx=         — display:contents (healthy boundary is layout-invisible)
    child sx=        — fallback chrome + retry control paint
    contracts        — [data-caught] swap (parent attr → child display);
                       content/fallback display gates; forced-colors
  No residual error-boundary matrix.
-->

<scope>
  props: {
    message = ''
  }
</scope>

<div use:catchRender sx="contents">
  <div class="content" sx="contents"><slot></slot></div>
  <div
    class="fallback"
    role="alert"
    sx="py-4 px-5 bg-tone-destructive-bg border border-tone-destructive-border rounded-md shadow-xs text-foreground font-body text-sm"
  >
    <slot name="fallback">
      <svg class="fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" sx="shrink-0 size-icon-size-lg text-destructive mt-1">
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
        <path d="M12 9v4"></path>
        <path d="M12 17h.01"></path>
      </svg>
      <column gap="1" class="fallback-body" sx="min-w-0">
        <div class="message" sx="text-muted-foreground leading-normal">Something went wrong</div>
        <button
          class="retry"
          type="button"
          anatomy
          sx="self-start bg-surface-1 border border-border rounded-sm py-1 px-3 font-body text-xs font-semibold text-foreground shadow-xs cursor-pointer hover:bg-muted focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2"
        >Retry</button>
      </column>
    </slot>
  </div>
</div>
