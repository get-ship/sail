<!--
  <card> — L1 surface with header/body/footer/image slots, tone border, status rail,
  interactive lift, optional accent bar, and spotlight (accent-bar=static alias).
  tone defaults to muted — muted rides the border token; other tones tint the frame.
  Semantic border = bound tone.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'muted'
    variant: 'default' | 'outline' | 'soft' | 'elevated' | 'ghost' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    // Alias of variant=soft — kept so existing markup does not break.
    fill = false
    // Alias of accent-bar=static — same ::before stripe, no second rail.
    spotlight = false
  }

  style base = {position(relative).overflow({ all: clip }).pad({ y: 4, x: 5 }).isolate().radius(lg).border({ color: border, width: 1 }).fg(card-foreground).elevation({ shadow: card-rest }).bg(card).filter({ backdrop: card-backdrop-filter }), animate: { transition: play(surface, fast) }}
  style outline = {bg(transparent)}
  style soft = {bg(muted).border({ color: border })}
  style elevated = {elevation({ shadow: lg }).border({ color: border-subtle })}
  style ghost = {bg(transparent).border({ color: transparent }).elevation({ shadow: none })}
  style padMd = {pad({ y: 4, x: 5 })}
  style padNone = {pad(0)}
  style padSm = {pad({ y: 2, x: 3 })}
  style padLg = {pad({ y: 5, x: 6 })}
  // Header/footer chrome: negative margins cancel host pad so chrome bleeds edge-to-edge.
  // Each step mirrors the padding match (sm/md/lg/none) — never hard-code padMd only.
  style headerChromeBase = {border({ color: border-subtle, width: 1, edge: b }).type({ family: heading })}
  style headerChromeMd = {pad({ x: 5, y: 4 }).margin({ t: -4, x: -5, b: 3 })}
  style headerChromeSm = {pad({ x: 3, y: 2 }).margin({ t: -2, x: -3, b: 2 })}
  style headerChromeLg = {pad({ x: 6, y: 5 }).margin({ t: -5, x: -6, b: 4 })}
  style headerChromeNone = {pad({ y: 3, x: 0 }).margin({ b: 3 })}
  style footerChromeBase = {border({ color: border-subtle, width: 1, edge: t })}
  style footerChromeMd = {pad({ x: 5, y: 4 }).margin({ t: 3, x: -5, b: -4 })}
  style footerChromeSm = {pad({ x: 3, y: 2 }).margin({ t: 2, x: -3, b: -2 })}
  style footerChromeLg = {pad({ x: 6, y: 5 }).margin({ t: 4, x: -6, b: -5 })}
  style footerChromeNone = {pad({ y: 3, x: 0 }).margin({ t: 3 })}
  // Shared ::before stripe for accent-bar static/hover; opacity differs per arm.
  style accentBar = {position({ scheme: absolute, l: 0, r: 0, t: 0 }).size({ h: half }).content(empty).bg({ image: primary-accent }).radius({ t: lg }).transition(opacity).duration(medium).ease(out).pointerEvents(none)}
  // Image slot: full-bleed top crop box. object-cover lives on the author's
  // <img> — Style cannot paint a slotted descendant.
  style imageWrap = {overflow({ all: hidden }).size({ w: full, h: 36, min-h: 36 }).bg(muted)}
  style imageMd = {margin({ t: -4, x: -5, b: 4 })}
  style imageSm = {margin({ t: -2, x: -3, b: 3 })}
  style imageLg = {margin({ t: -5, x: -6, b: 5 })}
  style imageNone = {margin({ b: 0 })}
</scope>

<div
  style(base, {match: {variant: {default: bg(card), outline: outline, soft: soft, elevated: elevated, ghost: ghost}, padding: { md: padMd, none: padNone, sm: padSm, lg: padLg }}, compound: [
      {tone: *, border({ color: tone })},
      {interactive: true, cursor(pointer).tapHighlight(none), when: {hover: border({ color: border-strong }).elevation({ shadow: hover }).translateY(-px), active: elevation({ shadow: card-rest }).translateY(0).duration(instant), motion-reduce: translateY(0), focus: border({ width: 2, kind: outline }), forced-colors: {when: {focus: border({ color: CanvasText, kind: outline })}}}},
      {fill: true, soft},
      {tone: muted, border({ color: border })},
      {status-rail: true, pad({ s: 4 }).border({ color: border-strong, width: 3, edge: s })},
      // padS tracks named padding tokens (not continuous) — keep as compounds.
      {padding: sm, status-rail: true, pad({ s: 2 })},
      {padding: md, status-rail: true, pad({ s: 4 })},
      {padding: lg, status-rail: true, pad({ s: 5 })},
      {padding: none, status-rail: true, pad({ s: 0 })},
      {accent-bar: static, when: {before: style(accentBar, {filter({ opacity: 100 })})}},
      {accent-bar: hover, when: {before: style(accentBar, {filter({ opacity: 0 })}), hover: {when: {before: filter({ opacity: 100 })}}}},
      {spotlight: true, when: {before: style(accentBar, {filter({ opacity: 100 })})}},
    ]})
>
  <div
    style(imageWrap, {match: { padding: { md: imageMd, sm: imageSm, lg: imageLg, none: imageNone } }, when: {empty: display(none)}})
  >
    <slot name="image"></slot>
  </div>
  <div
    style(headerChromeBase, {match: { padding: { md: headerChromeMd, sm: headerChromeSm, lg: headerChromeLg, none: headerChromeNone } }, when: {empty: display(none)}})
  >
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div
    style(footerChromeBase, {match: { padding: { md: footerChromeMd, sm: footerChromeSm, lg: footerChromeLg, none: footerChromeNone } }, when: {empty: display(none)}})
  >
    <slot name="footer"></slot>
  </div>
</div>
