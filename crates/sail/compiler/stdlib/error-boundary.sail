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

  SCOPE. The protected content arrives through a `<slot>`, so it belongs to the
  enclosing component's reactive scope, not the boundary's — a boundary hides a
  broken subtree, it does not dispose one. Effects inside it stay alive and may
  throw again; the re-entrancy guard makes that a no-op, so the first error is the
  one reported. Disposal would require locating the boundary by owner chain rather
  than by DOM walk.

  The visible message is the `message` prop (default "Something went wrong"), not
  the raw error text: an exception string is for a developer, not a user. The real
  error rides the `sail-error-caught` event detail and the `data-error-message`
  attribute.
-->

<scope>
  props: {
    message = ''
  }
</scope>

<div use:catchRender>
  <div class="content"><slot></slot></div>
  <div class="fallback" role="alert">
    <slot name="fallback">
      <svg class="fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
        <path d="M12 9v4"></path>
        <path d="M12 17h.01"></path>
      </svg>
      <div class="fallback-body">
        <div class="message">Something went wrong</div>
        <button class="retry" type="button" anatomy>Retry</button>
      </div>
    </slot>
  </div>
</div>
