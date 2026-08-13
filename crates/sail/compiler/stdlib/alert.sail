<!--
  <alert> — L1 inline status banner. Style value language only.
  Stripe / icon rest = alert tokens when tone is omitted.
  Semantic paint = bound tone group paths.
  Host subset: muted | info | success | warning | destructive (no primary/accent product use).
-->

<scope>
  props: {
    tone: 'muted' | 'info' | 'success' | 'warning' | 'destructive'
    dismissible = false
    onDismiss: () => void
  }

  style base = {box: place({ position: relative }).overflow({ all: hidden }).space({ pad: 3 }), paint: group().border({ color: border, width: 1 }).radius(md).fill({ color: muted }).ink({ color: foreground }).elevation(xs), text: type({ family: body, size: base, leading: normal })}
  // ::before stripe geometry (rest fill primary; host attaches tone via when.before).
  style stripe = {box: place({ position: absolute, left: 0, top: 0, bottom: 0 }).size({ w: 1 }), paint: content(empty).fill({ color: primary })}
  style ico = {box: size(control-h-xs), paint: radius(md).fill({ color: muted }).ink({ color: primary })}
  style title = {text: type({ weight: semibold })}
  style body = {paint: ink({ color: primary })}
  style actions = {box: space({ mt: 2 })}
  style close = {box: flow({ display: none, self: start }).size(control-h-xs).space({ pad: 0 }), paint: border({ width: 0 }).radius(sm).fill({ color: transparent }).ink({ color: primary }).motion({ property: colors, duration: fast, ease: out }).cursor(pointer)}
  style xmark = {box: flow({ display: block }).size(icon-size-sm)}
  style growCol = {box: flow({ grow: 1 }).size({ min-w: 0 })}
</scope>

<row
  nowrap
  align="start"
  gap="3"
  use:liveRegion
  use:dismissControl
  style(base, {when: {before: style(stripe, {paint: fill({ color: tone })}), forced-colors: {paint: border({ color: CanvasText }), when: {before: {paint: fill({ color: CanvasText })}}}}, compound: [
      {tone: muted, when: {before: {paint: fill({ color: primary })}}},
    ]})
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    style(ico, {box: flow({ shrink: 1 }), paint: ink({ color: tone }), when: {empty: {box: flow({ display: none })}}, compound: [{tone: muted, paint: fill({ color: muted }).ink({ color: primary })}]})
  >
    <slot name="icon"></slot>
  </row>
  <column gap="1" style(growCol)>
    <div style(title, {when: {empty: {box: flow({ display: none })}}})>
      <slot name="title"></slot>
    </div>
    <div style(body)>
      <slot></slot>
    </div>
    <row align="center" gap="2" style(actions, {when: {empty: {box: flow({ display: none })}}})>
      <slot name="actions"></slot>
    </row>
  </column>
  <button
    data-dismiss
    type="button"
    anatomy
    label="Dismiss"
    style(close, {box: flow({ shrink: 1 }), when: {group-dismissible: {box: flow({ display: inline-block })}, hover: {paint: fill({ color: muted }).ink({ color: foreground })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}}})
  >
    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round" stroke="currentColor" fill="none" style(xmark)>
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
