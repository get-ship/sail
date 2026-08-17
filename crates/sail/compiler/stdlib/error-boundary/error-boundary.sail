<!--
  <error-boundary> — group-caught swap on Style when.
  Fallback chrome from muted / danger / foreground.
-->
<scope>
  props: {
    message = ''
    onErrorReset: () => void
    onErrorCaught: (detail: { error: Error; source: string }) => void
  }

  style root = {display(contents).group()}
  style content = {display(contents)}
  style fallback = {display(none)}
  style alert = {pad({ y: 4, x: 5 }).bg(muted).border({ color: danger, width: 1 }).radius(md).elevation(xs).fg(foreground).type({ family: body, size: sm })}
  style icon = {size(icon-size-lg).margin({ t: 1 }).fg(danger)}
  style body = {size({ min-w: 0 })}
  style message = {fg(muted-foreground).type({ leading: normal })}
  style retry = {self(start).pad({ y: 1, x: 3 }).bg(surface).border({ color: border, width: 1 }).radius(sm).fg(foreground).elevation(xs).cursor(pointer).type({ family: body, size: xs, weight: semibold })}
</scope>
<div use:catchRender style(root)>
  <div style(content, {when: {group-caught: display(none)}})><slot></slot></div>
  <div style(fallback, {when: {group-caught: display(contents)}})>
    <!-- alert role stamped by use:catchRender (closed anatomy) -->
    <row align="start" gap="3" data-error-alert style(alert)>
      <slot name="fallback">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(icon, {shrink(0)})>
          <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
          <path d="M12 9v4"></path>
          <path d="M12 17h.01"></path>
        </svg>
        <column gap="1" style(body)>
          <div style(message)>Something went wrong</div>
          <button type="button" anatomy style(retry, {when: {hover: bg(muted), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})>Retry</button>
        </column>
      </slot>
    </row>
  </div>
</div>
