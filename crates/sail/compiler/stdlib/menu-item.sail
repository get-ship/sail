<!--
  <menu-item> — item inside dropdown/context-menu. Style product host paint.
  Button L1 contract supplies inline-flex/align/gap chrome.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  // display(flex) not inline-flex: inline items sit on one line under a
  // max-content menu and the panel width becomes the SUM of labels.
  // padY(2)+padX(3)+gap(2_5) ≈ 32px hit; icons align. Button floor already
  // supplies align-items:center.
  style base = {box: flow({ display: flex }).space({ pad-y: 2, pad-x: 3 }).size({ w: full }), paint: border({ width: 0 }).ink({ color: menu-item.foreground }).fill({ color: transparent }).radius(sm).cursor(pointer), text: type({ align: start, family: body, size: sm, leading: tight, wrap: nowrap })}
</scope>

<button
  type="button"
  style(base, {when: {hover: {paint: fill({ color: hover-bg })}, focus: {paint: fill({ color: hover-bg }).border({ style: solid, kind: outline, width: 2, color: ring })}, disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}}})
>
  <slot></slot>
</button>
