<!--
  <error-boundary> — group-caught swap on Style when.
  Fallback chrome from muted / destructive / foreground.
-->
<scope>
  props: {
    message = ''
    onErrorReset: () => void
    onErrorCaught: (detail: { error: Error; source: string }) => void
  }

  style root = {box: flow({ display: contents }), paint: group()}
  style content = {box: flow({ display: contents })}
  style fallback = {box: flow({ display: none })}
  style alert = {box: space({ pad-y: 4, pad-x: 5 }), paint: fill({ color: muted }).border({ color: destructive, width: 1 }).radius(md).elevation(xs).ink({ color: foreground }), text: type({ family: body, size: sm })}
  style icon = {box: size(icon-size-lg).space({ mt: 1 }), paint: ink({ color: destructive })}
  style body = {box: size({ min-w: 0 })}
  style message = {paint: ink({ color: muted-foreground }), text: type({ leading: normal })}
  style retry = {box: flow({ self: start }).space({ pad-y: 1, pad-x: 3 }), paint: fill({ color: card }).border({ color: border, width: 1 }).radius(sm).ink({ color: foreground }).elevation(xs).cursor(pointer), text: type({ family: body, size: xs, weight: semibold })}
</scope>
<div use:catchRender style(root)>
  <div style(content, {when: {group-caught: {box: flow({ display: none })}}})><slot></slot></div>
  <div style(fallback, {when: {group-caught: {box: flow({ display: contents })}}})>
    <!-- alert role stamped by use:catchRender (closed anatomy) -->
    <row align="start" gap="3" data-error-alert style(alert)>
      <slot name="fallback">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(icon, {box: flow({ shrink: 0 })})>
          <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
          <path d="M12 9v4"></path>
          <path d="M12 17h.01"></path>
        </svg>
        <column gap="1" style(body)>
          <div style(message)>Something went wrong</div>
          <button type="button" anatomy style(retry, {when: {hover: {paint: fill(muted)}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}}})>Retry</button>
        </column>
      </slot>
    </row>
  </div>
</div>
