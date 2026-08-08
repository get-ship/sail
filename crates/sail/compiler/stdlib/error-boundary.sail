<!--
  <error-boundary> — group-caught swap on Style when. Fallback chrome from
  component.error-boundary tokens.
-->
<scope>
  props: {
    message = ''
  
    onErrorReset: () => void
    onErrorCaught: (detail: { error: Error; source: string }) => void
}

  style root = { box: display(contents), paint: group() }
  style content = { box: display(contents) }
  style fallback = { box: display(none) }
  style alert = { box: padY(4).padX(5), paint: bg(error-boundary).borderWidth(1).border(error-boundary.border).radius(md).shadow(xs).fg(error-boundary.foreground), text: fontFamily(body).fontSize(sm) }
  style icon = { box: size(icon-size-lg).mt(1), paint: fg(error-boundary.icon-fg) }
  style body = { box: minW(0) }
  style message = { paint: fg(error-boundary.message-fg), text: leading(normal) }
  style retry = { box: self(start).padY(1).padX(3), paint: bg(error-boundary.retry-bg).borderWidth(1).border(border).radius(sm).fg(error-boundary.retry-fg).shadow(xs).cursor(pointer), text: fontFamily(body).fontSize(xs).fontWeight(semibold) }
</scope>
<div use:catchRender style(root)>
  <div style(content, { when: { group-caught: { box: display(none) } } })><slot></slot></div>
  <div style(fallback, { when: { group-caught: { box: display(contents) } } })>
    <!-- alert role stamped by use:catchRender (closed anatomy, not free author API) -->
    <row align="start" gap="3" data-error-alert style(alert)>
      <slot name="fallback">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" hidden="true" style(icon, { box: shrink(0) })>
          <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
          <path d="M12 9v4"></path>
          <path d="M12 17h.01"></path>
        </svg>
        <column gap="1" style(body)>
          <div style(message)>Something went wrong</div>
          <button type="button" anatomy style(retry, { when: { hover: { paint: bg(muted) }, focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) } } })>Retry</button>
        </column>
      </slot>
    </row>
  </div>
</div>
