<scope>
  props: {
    kind: 'family-body' | 'family-heading' | 'family-mono' | 'family-display' | 'size-xs' | 'size-sm' | 'size-base' | 'size-md' | 'size-lg' | 'size-xl' | 'size-2xl' | 'weight-normal' | 'weight-medium' | 'weight-semibold' | 'weight-bold' | 'leading-none' | 'leading-tight' | 'leading-snug' | 'leading-normal' | 'leading-relaxed' | 'tracking-tighter' | 'tracking-tight' | 'tracking-normal' | 'tracking-wide' | 'tracking-wider' | 'align-left' | 'align-center' | 'align-right' | 'align-justify' | 'align-start' | 'align-end' | 'case-uppercase' | 'case-lowercase' | 'case-capitalize' | 'case-normal' | 'case-label' | 'wrap-normal' | 'wrap-nowrap' | 'wrap-truncate' | 'wrap-pretty' | 'wrap-words' | 'wrap-pre-wrap' | 'list-none' | 'list-disc' | 'list-decimal' | 'decoration-none' | 'decoration-underline' | 'decoration-line-through' | 'italic-true' | 'italic-false' | 'nums-tabular' | 'nums-lining' | 'nums-oldstyle' | 'nums-proportional' | 'nums-ordinal' | 'nums-slashed-zero' | 'nums-normal' | 'break-words' | 'break-normal' | 'break-anywhere' = 'family-body'
  }
  style familyBody = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: body }) }
  style familyHeading = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: heading }) }
  style familyMono = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono }) }
  style familyDisplay = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: display }) }
  style sizeXs = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: xs }) }
  style sizeSm = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: sm }) }
  style sizeBase = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: base }) }
  style sizeMd = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: md }) }
  style sizeLg = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: lg }) }
  style sizeXl = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: xl }) }
  style size2xl = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ size: 2xl }) }
  style weightNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ weight: normal }) }
  style weightMedium = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ weight: medium }) }
  style weightSemibold = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ weight: semibold }) }
  style weightBold = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ weight: bold }) }
  style leadingNone = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ leading: none }) }
  style leadingTight = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ leading: tight }) }
  style leadingSnug = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ leading: snug }) }
  style leadingNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ leading: normal }) }
  style leadingRelaxed = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ leading: relaxed }) }
  style trackingTighter = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ tracking: tighter }) }
  style trackingTight = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ tracking: tight }) }
  style trackingNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ tracking: normal }) }
  style trackingWide = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ tracking: wide }) }
  style trackingWider = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ tracking: wider }) }
  style alignLeft = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: left }) }
  style alignCenter = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: center }) }
  style alignRight = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: right }) }
  style alignJustify = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: justify }) }
  style alignStart = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: start }) }
  style alignEnd = { size({ w: 24 }), pad(2), bg(surface), radius(sm), fg(primary), type({ align: end }) }
  style caseUpper = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ case: uppercase }) }
  style caseLower = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ case: lowercase }) }
  style caseCap = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ case: capitalize }) }
  style caseNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ case: normal }) }
  style caseLabel = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ case: label-transform }) }
  style wrapNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: normal }) }
  style wrapNowrap = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: nowrap }) }
  style wrapTruncate = { size({ w: 16, h: 8 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: truncate }) }
  style wrapPretty = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: pretty }) }
  style wrapWords = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: words }) }
  style wrapPre = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ wrap: pre-wrap }) }
  style listNone = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ list: none }) }
  style listDisc = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ list: disc }) }
  style listDecimal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ list: decimal }) }
  style decoNone = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ decoration: none }) }
  style decoUnderline = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ decoration: underline }) }
  style decoStrike = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ decoration: line-through }) }
  style italicTrue = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ italic: true }) }
  style italicFalse = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ italic: false }) }
  style numsTabular = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: tabular }) }
  style numsLining = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: lining }) }
  style numsOldstyle = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: oldstyle }) }
  style numsProportional = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: proportional }) }
  style numsOrdinal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: ordinal }) }
  style numsSlashed = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: slashed-zero }) }
  style numsNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ family: mono, nums: normal }) }
  style breakWords = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ break: words }) }
  style breakNormal = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ break: normal }) }
  style breakAnywhere = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type({ break: anywhere }) }
</scope>
<column style({ match: { kind: { 'family-body': familyBody, 'family-heading': familyHeading, 'family-mono': familyMono, 'family-display': familyDisplay, 'size-xs': sizeXs, 'size-sm': sizeSm, 'size-base': sizeBase, 'size-md': sizeMd, 'size-lg': sizeLg, 'size-xl': sizeXl, 'size-2xl': size2xl, 'weight-normal': weightNormal, 'weight-medium': weightMedium, 'weight-semibold': weightSemibold, 'weight-bold': weightBold, 'leading-none': leadingNone, 'leading-tight': leadingTight, 'leading-snug': leadingSnug, 'leading-normal': leadingNormal, 'leading-relaxed': leadingRelaxed, 'tracking-tighter': trackingTighter, 'tracking-tight': trackingTight, 'tracking-normal': trackingNormal, 'tracking-wide': trackingWide, 'tracking-wider': trackingWider, 'align-left': alignLeft, 'align-center': alignCenter, 'align-right': alignRight, 'align-justify': alignJustify, 'align-start': alignStart, 'align-end': alignEnd, 'case-uppercase': caseUpper, 'case-lowercase': caseLower, 'case-capitalize': caseCap, 'case-normal': caseNormal, 'case-label': caseLabel, 'wrap-normal': wrapNormal, 'wrap-nowrap': wrapNowrap, 'wrap-truncate': wrapTruncate, 'wrap-pretty': wrapPretty, 'wrap-words': wrapWords, 'wrap-pre-wrap': wrapPre, 'list-none': listNone, 'list-disc': listDisc, 'list-decimal': listDecimal, 'decoration-none': decoNone, 'decoration-underline': decoUnderline, 'decoration-line-through': decoStrike, 'italic-true': italicTrue, 'italic-false': italicFalse, 'nums-tabular': numsTabular, 'nums-lining': numsLining, 'nums-oldstyle': numsOldstyle, 'nums-proportional': numsProportional, 'nums-ordinal': numsOrdinal, 'nums-slashed-zero': numsSlashed, 'nums-normal': numsNormal, 'break-words': breakWords, 'break-normal': breakNormal, 'break-anywhere': breakAnywhere } } })>
  <slot></slot>
</column>
