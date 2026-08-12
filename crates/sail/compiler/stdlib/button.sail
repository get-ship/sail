<!--
  <button> — native <button>. L1 platform control. Style value language only.
  `default` is an alias of `solid` (prop default solid). tone always paints when set.
  solid without tone → inverted fg/bg; solid + tone → filled tone.
  soft / outline / ghost take tone the same way.
  Sizes: xs–lg text; icon / iconSm / iconLg square hit targets.
-->

<scope>
  props: {
    // default ≡ solid (kept as enum value so existing markup doesn't break).
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    // icon / iconSm / iconLg = square hit targets (no hyphen — Style match keys are idents).
    size: 'xs' | 'sm' | 'md' | 'lg' | 'icon' | 'iconSm' | 'iconLg' = 'md'
    disabled = false
  }

  // border(solid) kills UA button outset so freeform demos use design-system chrome.
  // No base bg(transparent) or border(transparent): unlayered product atomics
  // (.s-bgtransparent / .s-bordertransparent) beat layered variant fills and
  // outline strokes. Variants own fill + border color; ghost is the transparent pair.
  // Cursor is NOT on base: base cursor(pointer) + when-disabled cursor(not-allowed) both
  // stamp classes and pointer was winning in the cascade. Match on the disabled prop so
  // only one cursor utility is active.
  style base = {
    box: display(inline-flex).position(relative).gap(2),
    paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).borderImage(corner-image).isolate().controlFrame().select(none),
    text: fontWeight(medium).leading(none).whitespace(nowrap),
    when: { hover: { paint: controlLift() }, active: { paint: press() } },
  }
  style cursorOn = { paint: cursor(pointer) }
  style cursorOff = { paint: cursor(not-allowed).opacity(50) }

  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
  // Square icon hit targets — explicit w/h (not minW: parent .sail-col zeros min-width).
  style sizeIcon = { box: w(control-h-md).h(control-h-md).pad(0).padX(0), text: fontSize(base) }
  style sizeIconSm = { box: w(control-h-sm).h(control-h-sm).pad(0).padX(0), text: fontSize(sm) }
  style sizeIconLg = { box: w(control-h-lg).h(control-h-lg).pad(0).padX(0), text: fontSize(md) }

  // solid / default without tone: inverted · + tone: filled tone (IACVT).
  style vSolid = { paint: bg(foreground).border(foreground).fg(background).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSolidTone = { paint: bg(tone).border(tone).fg(tone.foreground).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSoft = { paint: bg(hover-bg).border(button.border).fg(button.foreground), when: { hover: { paint: opacity(90) } } }
  style vSoftTone = { paint: colorMix(tone, 15).border(tone).fg(tone), when: { hover: { paint: colorMix(tone, 25) } } }
  style vOutline = { paint: bg(transparent).border(button.foreground).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vOutlineTone = { paint: bg(transparent).border(tone).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhost = { paint: bg(transparent).border(transparent).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhostTone = { paint: bg(transparent).border(transparent).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
</scope>

<button
  style(base, {
    match: {
      disabled: {
        true: cursorOff,
        false: cursorOn,
      },
      variant: {
        // default ≡ solid — one fill model, tone always applies when set.
        default: style(vSolid, vSolidTone),
        solid: style(vSolid, vSolidTone),
        soft: style(vSoft, vSoftTone),
        outline: style(vOutline, vOutlineTone),
        ghost: style(vGhost, vGhostTone),
      },
      size: {
        xs: sizeXs,
        sm: sizeSm,
        md: sizeMd,
        lg: sizeLg,
        icon: sizeIcon,
        iconSm: sizeIconSm,
        iconLg: sizeIconLg,
      },
    },
    compound: [
      { tone: muted, variant: default, paint: bg(muted-foreground).border(muted-foreground).fg(background) },
      { tone: muted, variant: solid, paint: bg(muted-foreground).border(muted-foreground).fg(background) },
      { tone: muted, variant: soft, paint: bg(muted).border(border-strong).fg(foreground) },
      { tone: muted, variant: outline, paint: border(muted-foreground).fg(muted-foreground) },
      { tone: muted, variant: ghost, paint: fg(muted-foreground) },
    ],
  })
>
  <slot></slot>
</button>
