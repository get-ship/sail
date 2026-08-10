<!--
  <button> — native <button>. L1 platform control. Style value language only.
  Rest paint = component.button tokens when tone is omitted.
  Semantic paint = bound tone.
  Sizes: xs–lg text controls; icon / icon-sm / icon-lg square hit targets
  (Mantine/shadcn-class icon buttons). Children may include <icon> glyphs —
  base is inline-flex + gap so left/right icons compose without props.
-->

<scope>
  props: {
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    // icon / iconSm / iconLg = square hit targets (no hyphen — Style match keys are idents).
    size: 'xs' | 'sm' | 'md' | 'lg' | 'icon' | 'iconSm' | 'iconLg' = 'md'
    disabled = false
  }

  // border(solid) kills UA button outset so freeform demos use design-system chrome.
  // No base bg(transparent): unlayered product .s-bgtransparent beats layered solid
  // skins and empties primary fills. Variants own background; ghost is the transparent fill.
  // gap + align: icon + label (and icon-only) compose via children, not slots.
  // gap: icon + label compose as children (Mantine left/right sections via slot content).
  style base = { box: display(inline-flex).position(relative).gap(2), paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).borderImage(corner-image).isolate().controlFrame().select(none).cursor(pointer), text: fontWeight(medium).leading(none).whitespace(nowrap), when: { hover: { paint: controlLift() }, active: { paint: press() }, disabled: { paint: cursor(not-allowed).opacity(50) } } }

  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
  // Square icon hit targets — explicit w/h (not minW: parent .sail-col zeros min-width).
  style sizeIcon = { box: w(control-h-md).h(control-h-md).pad(0).padX(0), text: fontSize(base) }
  style sizeIconSm = { box: w(control-h-sm).h(control-h-sm).pad(0).padX(0), text: fontSize(sm) }
  style sizeIconLg = { box: w(control-h-lg).h(control-h-lg).pad(0).padX(0), text: fontSize(md) }

  style vDefault = { paint: bg(button).border(button.border).fg(button.foreground).shadow(xs), when: { hover: { paint: bg(hover-bg).border(border-strong) } } }
  // Solid without tone: inverted fg/bg. Solid + tone: bound group paint.
  style vSolid = { paint: bg(foreground).border(foreground).fg(background).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSolidTone = { paint: bg(tone).border(tone).fg(tone.foreground).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSoft = { paint: bg(hover-bg).border(button.border).fg(button.foreground), when: { hover: { paint: opacity(90) } } }
  style vSoftTone = { paint: colorMix(tone, 15).border(tone).fg(tone), when: { hover: { paint: colorMix(tone, 25) } } }
  // Outline rest must set bg(transparent) so UA buttonface does not show on dark themes.
  // Scoped to outline only — not base — so transparent bg cannot empty solid fills.
  style vOutline = { paint: bg(transparent).border(button.foreground).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vOutlineTone = { paint: bg(transparent).border(tone).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhost = { paint: bg(transparent).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhostTone = { paint: bg(transparent).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
</scope>

<button
  style(base, {
    match: {
      variant: {
        default: vDefault,
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
      { tone: muted, variant: solid, paint: bg(muted-foreground).border(muted-foreground).fg(background) },
      { tone: muted, variant: soft, paint: bg(muted).border(border-strong).fg(foreground) },
      { tone: muted, variant: outline, paint: border(muted-foreground).fg(muted-foreground) },
      { tone: muted, variant: ghost, paint: fg(muted-foreground) },
    ],
  })
>
  <slot></slot>
</button>
