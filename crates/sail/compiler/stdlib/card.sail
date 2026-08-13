<!--
  <card> — L1 surface with header/body/footer slots, tone border, status rail,
  interactive lift, and optional accent bar.
  Rest border = border token when tone is omitted.
  Semantic border = bound tone.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'default' | 'outline' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }

  style base = {box: place({ position: relative }).overflow({ all: clip }).space({ pad-y: 4, pad-x: 5 }), paint: isolate().radius(lg).border({ color: border, width: 1 }).ink(card).elevation({ shadow: card-rest }).motion({ property: colors, duration: fast, ease: out }).fill(card).filter({ backdrop: card-backdrop-filter })}
  style outline = {paint: fill({ color: transparent })}
  style padMd = {box: space({ pad-y: 4, pad-x: 5 })}
  style padNone = {box: space({ pad: 0 })}
  style padSm = {box: space({ pad-y: 2, pad-x: 3 })}
  style padLg = {box: space({ pad-y: 5, pad-x: 6 })}
  // Header/footer chrome: negative margins cancel host pad so chrome bleeds edge-to-edge.
  // Each step mirrors the padding match (sm/md/lg/none) — never hard-code padMd only.
  style headerChromeBase = {paint: border({ color: border-subtle, width: 1, edge: b }), text: type({ family: heading })}
  style headerChromeMd = {box: space({ mt: -4, mx: -5, mb: 3, pad-x: 5, pad-y: 4 })}
  style headerChromeSm = {box: space({ mt: -2, mx: -3, mb: 2, pad-x: 3, pad-y: 2 })}
  style headerChromeLg = {box: space({ mt: -5, mx: -6, mb: 4, pad-x: 6, pad-y: 5 })}
  style headerChromeNone = {box: space({ mb: 3, pad-y: 3, pad-x: 0 })}
  style footerChromeBase = {paint: border({ color: border-subtle, width: 1, edge: t })}
  style footerChromeMd = {box: space({ mt: 3, mx: -5, mb: -4, pad-x: 5, pad-y: 4 })}
  style footerChromeSm = {box: space({ mt: 2, mx: -3, mb: -2, pad-x: 3, pad-y: 2 })}
  style footerChromeLg = {box: space({ mt: 4, mx: -6, mb: -5, pad-x: 6, pad-y: 5 })}
  style footerChromeNone = {box: space({ mt: 3, pad-y: 3, pad-x: 0 })}
  // Shared ::before stripe for accent-bar static/hover; opacity differs per arm.
  style accentBar = {box: place({ position: absolute, left: 0, right: 0, top: 0 }).size({ h: half }), paint: content(empty).fill({ image: primary-accent }).radius({ t: lg }).motion({ property: opacity, duration: medium, ease: out }).pointerEvents(none)}
</scope>

<div
  style(base, {match: {variant: {default: {paint: fill(card)}, outline: outline}, padding: {
        md: padMd,
        none: padNone,
        sm: padSm,
        lg: padLg,
      }}, compound: [
      {tone: *, paint: border({ color: tone })},
      {interactive: true, paint: cursor(pointer).tapHighlight(none), when: {hover: {paint: border({ color: border-strong }).elevation({ shadow: hover }).motion({ y: -px })}, active: {paint: elevation({ shadow: card-rest }).motion({ y: 0, duration: instant })}, motion-reduce: {paint: motion({ y: 0 })}, focus: {paint: border({ width: 2, kind: outline })}, forced-colors: {when: {focus: {paint: border({ color: CanvasText, kind: outline })}}}}},
      {fill: true, paint: fill(muted).border({ color: border })},
      {tone: muted, paint: border({ color: border })},
      {status-rail: true, box: space({ pad-s: 4 }), paint: border({ color: border-strong, width: 3, edge: s })},
      // padS tracks named padding tokens (not continuous) — keep as compounds.
      {padding: sm, status-rail: true, box: space({ pad-s: 2 })},
      {padding: md, status-rail: true, box: space({ pad-s: 4 })},
      {padding: lg, status-rail: true, box: space({ pad-s: 5 })},
      {padding: none, status-rail: true, box: space({ pad-s: 0 })},
      {accent-bar: static, when: {before: style(accentBar, {paint: filter({ opacity: 100 })})}},
      {accent-bar: hover, when: {before: style(accentBar, {paint: filter({ opacity: 0 })}), hover: {when: {before: {paint: filter({ opacity: 100 })}}}}},
    ]})
>
  <div
    style(headerChromeBase, {match: {
        padding: {
          md: headerChromeMd,
          sm: headerChromeSm,
          lg: headerChromeLg,
          none: headerChromeNone,
        },
      }, when: {empty: {box: flow({ display: none })}}})
  >
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div
    style(footerChromeBase, {match: {
        padding: {
          md: footerChromeMd,
          sm: footerChromeSm,
          lg: footerChromeLg,
          none: footerChromeNone,
        },
      }, when: {empty: {box: flow({ display: none })}}})
  >
    <slot name="footer"></slot>
  </div>
</div>
