<!--
  <error-boundary> — group-caught: swap on structure sx. forced-colors in skins anatomy.
-->
<scope>
  props: {
    message = ''
  }
</scope>
<div use:catchRender sx="contents group">
  <div class="content" sx="contents group-caught:hidden"><slot></slot></div>
  <div class="fallback" sx="hidden group-caught:contents">
    <row align="start" gap="3" role="alert" sx="py: 4 px: 5 bg: tone-destructive-bg border border: tone-destructive-border rounded: md shadow: xs color: foreground font-family: body font-size: sm">
      <slot name="fallback">
        <svg class="fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" sx="shrink: 0 size: icon-size-lg color: destructive mt: 1">
          <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
          <path d="M12 9v4"></path>
          <path d="M12 17h.01"></path>
        </svg>
        <column gap="1" class="fallback-body" sx="min-w: 0">
          <div class="message" sx="color: muted-foreground leading: normal">Something went wrong</div>
          <button class="retry" type="button" anatomy sx="self: start bg: surface-1 border border: border rounded: sm py: 1 px: 3 font-family: body font-size: xs font-weight: semibold color: foreground shadow: xs cursor: pointer hover:bg: muted focus:outline: solid focus:outline: 2 focus:outline: ring focus:outline-offset: 2">Retry</button>
        </column>
      </slot>
    </row>
  </div>
</div>
