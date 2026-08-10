<!--
  /components/[tag] — detail route. Live stage + prop inspector + usage + props.
  Button pilot: specimen left, select knobs right (Mantine-style configurator).
  L1 from component-details; pack tags from component-pack-meta. Template reads
  only zero-arg calls (`entry()`, `stage.button()`).
-->

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { DETAILS, type DetailEntry } from '../../../src/compositions/component-details'
import {
  DEMO_PHOTO,
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
  packSlotCopy,
} from '../../../src/compositions/component-pack-meta'

export default controller((ctx: ControllerScope<Scope>) => {
    const tag = derived(() => ctx.route.params.tag?.() ?? '')
    const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
    const known = derived(() => entry() !== null)
    // Pack tags lack DETAILS rows; still mount via knownPack (catalog skips
    // message-list/media-csv — importmap gap, not this route).
    const knownPack = derived(() => PACK_TAGS.has(tag()))
    const showViewer = derived(() => known() || knownPack())

    const variant = signal('')
    // Configurator knobs (Mantine-style): per-tag size ladders + shared tone swatches.
    const BTN_SIZES = ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const
    const BDG_SIZES = ['sm', 'md', 'lg'] as const
    const CHIP_SIZES = ['sm', 'md', 'lg'] as const
    const SPN_SIZES = ['xs', 'sm', 'md', 'lg'] as const
    const SKEL_SIZES = ['sm', 'md', 'lg'] as const
    const METER_SIZES = ['sm', 'md', 'lg'] as const
    const sizeIdx = signal(2) // md for button; remapped per tag
    const btnTone = signal('primary')
    // Badge: empty tone = rest component.badge.* tokens (theme-customizable).
    const badgeTone = signal('primary')
    const badgePill = signal(false)
    // Separator: empty tone = rest component.separator.line (never empty bg(tone)).
    const sepOrientation = signal('horizontal')
    const sepTone = signal('')
    // Alert: empty tone = rest component.alert.*; dismiss remount via visible flag.
    const alertTone = signal('info')
    const alertDismissible = signal(false)
    const alertVisible = signal(true)
    // Chip: empty tone = rest component.chip.*; selected/removable bools.
    const chipTone = signal('primary')
    const chipSelected = signal(false)
    const chipRemovable = signal(true)
    // Spinner: empty tone = rest component.spinner.fg; track always spinner.track.
    const spinnerTone = signal('primary')
    const spinnerCentered = signal(false)
    // Skeleton: shape via variant(); size ladder; sheen always component.skeleton.*
    // Meter: value + size; low/high/optimum fixed for band demo (native thresholds).
    const meterVal = signal(55)
    // Chip dismiss demo (re-seed with chip knobs; restore after ×).
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    const restoreChip = (): void => kept.set(true)
    const btnSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BTN_SIZES.length - 1)
      return BTN_SIZES[i] ?? 'md'
    })
    const badgeSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BDG_SIZES.length - 1)
      return BDG_SIZES[i] ?? 'md'
    })
    const chipSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), CHIP_SIZES.length - 1)
      return CHIP_SIZES[i] ?? 'md'
    })
    const spinnerSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SPN_SIZES.length - 1)
      return SPN_SIZES[i] ?? 'md'
    })
    const skelSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SKEL_SIZES.length - 1)
      return SKEL_SIZES[i] ?? 'md'
    })
    const meterSize = derived(() => {
      const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), METER_SIZES.length - 1)
      return METER_SIZES[i] ?? 'md'
    })
    // Scope is reused across detail navigations — re-seed on tag change.
    effect(() => {
      if (tag() === 'button') {
        variant.set('solid')
        sizeIdx.set(2)
        btnTone.set('primary')
      } else if (tag() === 'badge') {
        variant.set('soft')
        sizeIdx.set(1) // md
        badgeTone.set('primary')
        badgePill.set(false)
      } else if (tag() === 'separator') {
        variant.set('solid')
        sepOrientation.set('horizontal')
        sepTone.set('')
      } else if (tag() === 'alert') {
        variant.set('')
        alertTone.set('info')
        alertDismissible.set(false)
        alertVisible.set(true)
      } else if (tag() === 'chip') {
        variant.set('solid')
        sizeIdx.set(1) // md
        chipTone.set('primary')
        chipSelected.set(false)
        chipRemovable.set(true)
        kept.set(true)
      } else if (tag() === 'spinner') {
        variant.set('')
        sizeIdx.set(2) // md
        spinnerTone.set('primary')
        spinnerCentered.set(false)
      } else if (tag() === 'skeleton') {
        variant.set('text')
        sizeIdx.set(1) // md
      } else if (tag() === 'meter') {
        variant.set('')
        sizeIdx.set(1) // md
        meterVal.set(55)
      } else {
        variant.set(entry()?.variants[0] ?? '')
      }
    })
    const pick = (v: string): void => variant.set(v)
    const onVariantChange = (detail: { variant: string }): void =>
      variant.set(detail?.variant ?? '')
    const pickTone = (t: string): void => btnTone.set(t)
    const pickBadgeTone = (t: string): void => badgeTone.set(t)
    const clearBadgeTone = (): void => badgeTone.set('')
    const pickSepTone = (t: string): void => sepTone.set(t)
    const clearSepTone = (): void => sepTone.set('')
    const pickAlertTone = (t: string): void => {
      alertTone.set(t)
      alertVisible.set(true)
    }
    const clearAlertTone = (): void => {
      alertTone.set('')
      alertVisible.set(true)
    }
    const onAlertDismiss = (): void => alertVisible.set(false)
    const restoreAlert = (): void => alertVisible.set(true)
    const pickChipTone = (t: string): void => chipTone.set(t)
    const clearChipTone = (): void => chipTone.set('')
    const pickSpinnerTone = (t: string): void => spinnerTone.set(t)
    const clearSpinnerTone = (): void => spinnerTone.set('')
    const btnMeta = derived(() => `${variant()} · ${btnSize()} · ${btnTone()} · press ×${presses()}`)
    const badgeMeta = derived(() => {
      const t = badgeTone()
      return `${variant()} · ${badgeSize()} · ${t || 'rest (component tokens)'} · pill ${badgePill() ? 'on' : 'off'}`
    })
    const sepMeta = derived(() => {
      const t = sepTone()
      return `${sepOrientation()} · ${variant() || 'solid'} · ${t || 'rest (separator.line)'}`
    })
    const alertMeta = derived(() => {
      const t = alertTone()
      const d = alertDismissible() ? 'dismissible' : 'sticky'
      const live =
        t === 'warning' || t === 'destructive' ? 'assertive alert' : 'polite status'
      return `${t || 'rest (component tokens)'} · ${d} · ${live}${alertVisible() ? '' : ' · dismissed'}`
    })
    const alertTitle = derived(() => {
      switch (alertTone()) {
        case 'success':
          return 'Saved'
        case 'warning':
          return 'Check your input'
        case 'destructive':
          return 'Something went wrong'
        case 'muted':
          return 'Note'
        case 'info':
          return 'Heads up'
        default:
          return 'Notice'
      }
    })
    const alertBody = derived(() => {
      switch (alertTone()) {
        case 'success':
          return 'All changes were written successfully.'
        case 'warning':
          return 'Two fields need attention before you continue.'
        case 'destructive':
          return 'We could not reach the server. Try again.'
        case 'muted':
          return 'Neutral callout painted from component.alert tokens and muted stripe.'
        case 'info':
          return 'A new version is available. Refresh when you get a chance.'
        default:
          return 'Rest paint uses component.alert.* leaves — themeable in Studio.'
      }
    })
    const chipMeta = derived(() => {
      const t = chipTone()
      return `${variant()} · ${chipSize()} · ${t || 'rest (component tokens)'} · selected ${chipSelected() ? 'on' : 'off'} · removable ${chipRemovable() ? 'on' : 'off'}`
    })
    const spinnerMeta = derived(() => {
      const t = spinnerTone()
      return `${spinnerSize()} · ${t || 'rest (spinner.foreground)'} · centered ${spinnerCentered() ? 'on' : 'off'}`
    })
    const skelMeta = derived(() => {
      const sh = variant() || 'text'
      const sizeNote = sh === 'block' ? 'size n/a (block)' : skelSize()
      return `${sh} · ${sizeNote} · sheen component.skeleton.*`
    })
    // Band labels match fixed low=33 high=66 optimum=100 (HTML meter regions).
    const meterBand = derived(() => {
      const v = Number(meterVal()) || 0
      if (v >= 66) return 'optimum'
      if (v >= 33) return 'suboptimum'
      return 'even-less-good'
    })
    const meterMeta = derived(() => {
      return `${meterSize()} · value ${meterVal()} · ${meterBand()} · component.meter.*`
    })
    const isIconSize = derived(() => {
      const s = btnSize()
      return s === 'icon' || s === 'iconSm' || s === 'iconLg'
    })
    // Hide viewer variant strip when stage owns knobs.
    const stageOwnsKnobs = () =>
      tag() === 'button' ||
      tag() === 'badge' ||
      tag() === 'separator' ||
      tag() === 'alert' ||
      tag() === 'chip' ||
      tag() === 'spinner' ||
      tag() === 'skeleton' ||
      tag() === 'meter'
    const viewerVariants = () => (stageOwnsKnobs() ? [] : entry()?.variants ?? [])
    const viewerVariantAttr = () => (stageOwnsKnobs() ? '' : entry()?.variantAttr ?? '')
    // Example walkthrough snippets (code + prose below the stage — not a second matrix).
    const btnExWithIcon = () =>
      `<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>`
    const btnExIconOnly = () =>
      `<button variant="solid" tone="primary" size="icon" label="Settings">\n  <icon name="settings"></icon>\n</button>\n<button variant="ghost" size="iconSm" label="Search">\n  <icon name="search" size="sm"></icon>\n</button>`
    const btnExSizes = () =>
      `<button variant="solid" tone="primary" size="xs">xs</button>\n<button variant="solid" tone="primary" size="lg">lg</button>`
    const bdgExStatus = () =>
      `<badge variant="soft" tone="success" size="sm">active</badge>\n<badge variant="outline" tone="muted" size="sm">away</badge>`
    const bdgExPill = () =>
      `<badge variant="solid" tone="primary" pill>12</badge>\n<badge variant="soft" tone="destructive" pill>3</badge>`
    const bdgExRest = () =>
      `<badge variant="soft">rest tokens</badge>\n<!-- no tone → component.badge.* (themeable) -->`
    const sepExBoth = () =>
      `<column gap="2">\n  <p>Section A</p>\n  <separator></separator>\n  <p>Section B</p>\n  <row gap="2" align="center">\n    <span>Left</span>\n    <separator orientation="vertical"></separator>\n    <span>Right</span>\n  </row>\n</column>`
    const sepExVariant = () =>
      `<separator variant="dashed"></separator>\n<separator variant="solid" tone="primary"></separator>`
    const sepExRest = () =>
      `<separator></separator>\n<!-- no tone → component.separator.line (themeable) -->`
    const alrtExTones = () =>
      `<alert tone="info">\n  <strong slot="title">Heads up</strong>\n  A new version is available.\n</alert>\n<alert tone="success">\n  <strong slot="title">Saved</strong>\n  All changes were written.\n</alert>`
    const alrtExDismiss = () =>
      `<alert tone="warning" dismissible onDismiss={onDismiss}>\n  <strong slot="title">Check your input</strong>\n  Two fields need attention.\n</alert>`
    const alrtExRest = () =>
      `<alert>\n  <strong slot="title">Notice</strong>\n  Rest tokens — component.alert.*\n</alert>\n<!-- no tone → themeable component leaves -->`
    const chipExFilter = () =>
      `<chip variant="solid" tone="primary" selected>Design</chip>\n<chip variant="outline" tone="muted">Research</chip>`
    const chipExRemove = () =>
      `<chip variant="soft" tone="info" removable on-remove={onRemove}>typescript</chip>`
    const chipExRest = () =>
      `<chip variant="solid">rest tokens</chip>\n<!-- no tone → component.chip.* (themeable) -->`
const kbdExChord = () =>
      `<p>Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.</p>`
    const kbdExModifiers = () =>
      `<kbd>⌘</kbd> <kbd>⇧</kbd> <kbd>⌥</kbd> <kbd>⌃</kbd> <kbd>Esc</kbd> <kbd>F12</kbd>`
    const kbdExRest = () =>
      `<kbd>Enter</kbd>\n<!-- paint from component.kbd (bg · foreground · border) — themeable -->`
    const proseExScale = () =>
      `<prose>\n  <h1>Heading one</h1>\n  <h2>Heading two</h2>\n  <p>Body sets the baseline.</p>\n</prose>`
    const proseExBlocks = () =>
      `<prose>\n  <p>Inline <strong>strong</strong>, <em>emphasis</em>, <code>code()</code>, <mark>mark</mark>.</p>\n  <blockquote>A quotation carries its own rule.</blockquote>\n  <ul><li>Unordered</li><li>Items</li></ul>\n</prose>`
    const proseExRest = () =>
      `<prose>\n  <p>Markdown blobs and long copy.</p>\n</prose>\n<!-- host fg → component.prose.fg; descendants via PROSE_MAP -->`
    const spnExInline = () =>
      `<row gap="2" align="center">\n  <spinner size="sm" tone="primary"></spinner>\n  <p>fetching…</p>\n</row>`
    const spnExSizes = () =>
      `<spinner size="xs" tone="primary"></spinner>\n<spinner size="sm" tone="primary"></spinner>\n<spinner tone="primary"></spinner>\n<spinner size="lg" tone="primary"></spinner>`
    const spnExRest = () =>
      `<spinner></spinner>\n<!-- no tone → component.spinner.fg + spinner.track (themeable) -->`
    const spnExCentered = () =>
      `<spinner centered tone="primary"></spinner>\n<!-- block + mx(auto) in the parent -->`
    const skelExLines = () =>
      `<column gap="2">\n  <skeleton shape="text" size="md"></skeleton>\n  <skeleton shape="text" size="sm"></skeleton>\n  <skeleton shape="line" size="sm"></skeleton>\n</column>`
    const skelExAvatar = () =>
      `<row gap="3" align="center">\n  <skeleton shape="circle" size="md"></skeleton>\n  <column gap="2" style({ box: grow(1) })>\n    <skeleton shape="text" size="sm"></skeleton>\n    <skeleton shape="text" size="sm"></skeleton>\n  </column>\n</row>`
    const skelExRest = () =>
      `<skeleton shape="block"></skeleton>\n<!-- sheen → component.skeleton.bg / highlight (themeable) -->`
    const mtrExMeasure = () =>
      `<column gap="2">\n  <p>Disk free — 72%</p>\n  <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>\n</column>`
    const mtrExSizes = () =>
      `<meter value="0.55" size="sm"></meter>\n<meter value="0.55"></meter>\n<meter value="0.55" size="lg"></meter>`
    const mtrExBands = () =>
      `<meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<!-- even-less-good → component.meter.sub-suboptimum -->\n<meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<!-- suboptimum → component.meter.suboptimum -->\n<meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<!-- optimum → component.meter.optimum -->`
    const mtrExRest = () =>
      `<meter value="0.55"></meter>\n<!-- track → component.meter.track; bands → optimum / suboptimum / sub-suboptimum -->`
    // sail-media-image src is a JS identity property, not a reflect attr.
    effect(() => {
      void tag()
      requestAnimationFrame(() => {
        document.querySelectorAll('sail-media-image').forEach((el) => {
          const host = el as HTMLElement & { src?: string }
          if (host.src !== DEMO_PHOTO) host.src = DEMO_PHOTO
        })
      })
    })

    const variants = () => entry()?.variants ?? []
    const hasVariants = derived(() => (entry()?.variants.length ?? 0) > 0)
    const variantAttr = () => entry()?.variantAttr ?? ''
    const blurb = () => entry()?.blurb ?? ''
    const props = () => entry()?.props ?? []
    const usageCode = () => entry()?.usage ?? ''
    const slotCopy = () => (knownPack() ? packSlotCopy(tag()) : '')
    const heading = () => entry()?.heading ?? tag()
    const packBlurb = () => packBlurbFor(tag())
    const packProps = () => PACK_PROPS[tag()] ?? []
    const packLabel = () => packLabelFor(tag())

    const stage = Object.fromEntries([
      ...Object.values(DETAILS).map((e) => [e.key, () => entry()?.key === e.key]),
      ...[...PACK_TAGS].map((k) => [k.replace(/-([a-z])/g, (_, c) => c.toUpperCase()), () => tag() === k]),
    ]) as Record<string, () => boolean>

    const presses = signal(0)
    const press = (): void => presses.set((n) => n + 1)
    const notify = signal(true)
    const agreed = signal(false)
    const load = signal(62)
    const name = signal('')
    const bio = signal('')
    const team = signal('Design')
    const fw = signal('')
    const pickFw = (detail: { value: string }): void => fw.set(detail?.value ?? '')
    const pageNo = signal('2')
    const goPage = (detail: { page?: string; value?: string }): void => pageNo.set(detail?.page ?? detail?.value ?? '1')
    // Toast stage owns instances (push → dismiss → re-push). L1 remove() is
    // correct; the stage must remount via signal, not treat one node as forever.
    let toastSeq = 0
    const toastItems = signal<{ key: string }[]>([{ key: 'toast-0' }])
    const pushToast = (): void => {
      toastSeq += 1
      toastItems.set([{ key: `toast-${toastSeq}` }])
    }
    const onToastDismiss = (): void => {
      toastItems.set([])
    }
    // Re-seed when navigating onto the toast detail (scope is reused).
    effect(() => {
      if (tag() === 'toast') {
        toastSeq = 0
        toastItems.set([{ key: 'toast-0' }])
      }
    })
    const sheetSide = () => (variant() === 'sheet' ? 'end' : null)
    const people = () => [
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
    ]

    return {
      tag, entry, known, knownPack, showViewer, heading,
      variant, pick, onVariantChange, variants, hasVariants, variantAttr,
      viewerVariants, viewerVariantAttr,
      blurb, props, packBlurb, packProps, packLabel, usageCode, stage, slotCopy,
      presses, press, notify, agreed, load, name, bio, team,
      fw, pickFw, pageNo, goPage, kept, dropChip, restoreChip,
      toastItems, pushToast, onToastDismiss,
      sheetSide, people,
      sizeIdx, btnSize, btnTone, btnMeta, isIconSize, pickTone,
      btnExWithIcon, btnExIconOnly, btnExSizes,
      badgeTone, badgeSize, badgePill, badgeMeta, pickBadgeTone, clearBadgeTone,
      bdgExStatus, bdgExPill, bdgExRest,
      sepOrientation, sepTone, sepMeta, pickSepTone, clearSepTone,
      sepExBoth, sepExVariant, sepExRest,
      alertTone, alertDismissible, alertVisible, alertMeta, alertTitle, alertBody,
      pickAlertTone, clearAlertTone, onAlertDismiss, restoreAlert,
      alrtExTones, alrtExDismiss, alrtExRest,
      chipTone, chipSize, chipSelected, chipRemovable, chipMeta, pickChipTone, clearChipTone,
      chipExFilter, chipExRemove, chipExRest,
      kbdExChord, kbdExModifiers, kbdExRest,
      proseExScale, proseExBlocks, proseExRest,
      spinnerTone, spinnerSize, spinnerCentered, spinnerMeta, pickSpinnerTone, clearSpinnerTone,
      spnExInline, spnExSizes, spnExRest, spnExCentered,
      skelSize, skelMeta, skelExLines, skelExAvatar, skelExRest,
      meterVal, meterSize, meterMeta, meterBand,
      mtrExMeasure, mtrExSizes, mtrExBands, mtrExRest,
    }
})
</controller>

<scope>
  // Desktop width — configurator needs room for specimen + 320px prop rail.
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  // Explicit width (not minW) — `.sail-col > * { min-width: 0 }` zeros any
  // min-width on children, so minW(panel-*) never lands. width: token wins.
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style meterCol = { box: w(panel-max-w-sm) }
  style grow1 = { box: grow(1) }
  style cardPad = { box: pad(4) }
  style cardPad5 = { box: pad(5) }
  style mediaStage = { box: w(full).maxW(panel-max-w-sm).h(48).minH(48).overflow(hidden), paint: radius(md) }
  style fullMedia = { box: w(full).h(full) }
  style fullW = { box: w(full) }
  style ticker = { box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3), paint: bg(muted).radius(md) }
  style tickerText = { box: padX(4), paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono).whitespace(nowrap) }
  style scrollDemo = { box: maxH(panel-max-h-sm).w(full).maxW(panel-max-w-sm).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style pressBtn = { animate: press({ scale: 'press' }) }
  style emptyEnter = { animate: enter({ opacity: 'fade', translate-y: 'spring:snappy' }) }
  style skeletonBox = { box: w(7).h(7) }
  // Skeleton stage: bar/block need a definite width (w(full) resolves against this).
  style skelStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelBlock = { box: w(full).maxW(panel-max-w-sm) }
  style ctxCard = { box: pad(6).w(full) }
  style toastStage = { box: w(full).maxW(panel-max-w-sm) }
  // Mantine-style configurator: tall specimen canvas | fixed prop rail.
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  // Color swatches (token paint). selected ring via Style when:{selected}.
  style swatch = {
    box: w(7).h(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  // Rest / clear-tone marker (component.badge / separator.line paint the specimen, not this chip).
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  // Vertical separator needs a definite row height so h(full)/self(stretch) resolve.
  style sepVRow = { box: h(10) }
  style sepStageCol = { box: w(full).maxW(panel-max-w-sm) }
</scope>

<column gap="4" style(page)>
  <if {showViewer()}>
    <sail-component-viewer
      name={heading()}
      blurb={knownPack() ? packBlurb() : blurb()}
      pack={knownPack() ? packLabel() : 'Component primitives'}
      variant-attr={viewerVariantAttr()}
      usage={usageCode()}
      variants={viewerVariants()}
      rows={knownPack() ? packProps() : props()}
      on-variant-change={onVariantChange}
    >
      <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
            <!-- ══ Inputs ══════════════════════════════════════════════ -->
            <if {stage.button()}>
              <!-- Specimen fills the canvas; prop rail is a real sidebar (Mantine configurator). -->
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center">
                    <if {isIconSize()}>
                      <button
                        variant={variant()}
                        size={btnSize()}
                        tone={btnTone()}
                        on:click={press}
                        label="Settings"
                        style(pressBtn)
                      >
                        <icon name="settings"></icon>
                      </button>
                    {else =>
                      <button
                        variant={variant()}
                        size={btnSize()}
                        tone={btnTone()}
                        on:click={press}
                        style(pressBtn)
                      >
                        <icon name="check" size="sm"></icon>
                        Save changes
                      </button>}</if>
                    <p style(monoXs)>{btnMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Variant</label>
                    <select size="sm" bind:value={variant} label="Variant">
                      <option value="solid">solid</option>
                      <option value="soft">soft</option>
                      <option value="outline">outline</option>
                      <option value="ghost">ghost</option>
                    </select>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <!-- Plain divs — L1 <button> skins would override bg(tone) paint. -->
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="primary" placement="top">
                        <div label="primary" on:click={() => pickTone('primary')} selected={btnTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
                      </tooltip>
                      <tooltip content="success" placement="top">
                        <div label="success" on:click={() => pickTone('success')} selected={btnTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
                      </tooltip>
                      <tooltip content="warning" placement="top">
                        <div label="warning" on:click={() => pickTone('warning')} selected={btnTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickTone('destructive')} selected={btnTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                      <tooltip content="accent" placement="top">
                        <div label="accent" on:click={() => pickTone('accent')} selected={btnTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
                      </tooltip>
                      <tooltip content="muted" placement="top">
                        <div label="muted" on:click={() => pickTone('muted')} selected={btnTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
                      </tooltip>
                      <tooltip content="info" placement="top">
                        <div label="info" on:click={() => pickTone('info')} selected={btnTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
                      </tooltip>
                    </row>
                  </column>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{btnSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="6" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>xs</span>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                      <span style(tick)>icon</span>
                      <span style(tick)>iSm</span>
                      <span style(tick)>iLg</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.input()}>
              <column gap="3" style(stageCol)>
                <input placeholder="Ada Lovelace" bind:value={name} />
                <p style(monoXs)>hello, {name() || 'stranger'}</p>
              </column>
            </if>
            <if {stage.textarea()}>
              <column gap="3" style(stageCol)>
                <textarea placeholder="Tell us about yourself" bind:value={bio}></textarea>
                <p style(monoXs)>{bio().length} characters</p>
              </column>
            </if>
            <if {stage.select()}>
              <column gap="3" style(stageCol)>
                <select bind:value={team}>
                  <option>Engineering</option>
                  <option>Design</option>
                  <option>Research</option>
                </select>
                <p style(monoXs)>team: {team()}</p>
              </column>
            </if>
            <if {stage.combobox()}>
              <column gap="3" style(stageCol)>
                <combobox placeholder="Pick a framework — type to filter" on-select={pickFw}>
                  <option value="sail">Sail</option>
                  <option value="react">React</option>
                  <option value="svelte">Svelte</option>
                  <option value="solid">Solid</option>
                  <option value="vue">Vue</option>
                  <option value="qwik">Qwik</option>
                </combobox>
                <p style(monoXs)>picked: {fw() || 'nothing yet'}</p>
              </column>
            </if>
            <if {stage.checkbox()}>
              <column gap="3" align="center">
                <label><input type="checkbox" bind:checked={agreed} /> I agree to the terms</label>
                <p style(monoXs)>{agreed() ? 'thank you' : 'the platform checkbox, skinned'}</p>
              </column>
            </if>
            <if {stage.radioGroup()}>
              <radio-group label="Billing period">
                <label><input type="radio" name="stage-plan" checked /> Monthly</label>
                <label><input type="radio" name="stage-plan" /> Yearly</label>
              </radio-group>
            </if>
            <if {stage.switchToggle()}>
              <row gap="3" align="center">
                <switch bind:checked={notify} size={variant()} tone="primary"></switch>
                <p style(monoXs)>{notify() ? 'on' : 'off'} · size {variant()}</p>
              </row>
            </if>
            <if {stage.slider()}>
              <column gap="3" style(stageCol)>
                <slider bind:value={load} min="0" max="100"></slider>
                <progress value={load()} max="100"></progress>
                <p style(monoXs)>one signal, two components — {load()}%</p>
              </column>
            </if>
            <if {stage.toggleGroup()}>
              <toggle-group variant={variant()}>
                <toggle value="left">Left</toggle>
                <toggle value="center">Center</toggle>
                <toggle value="right">Right</toggle>
              </toggle-group>
            </if>
            <if {stage.field()}>
              <column gap="4" style(stageCol)>
                <field>
                  <label for="stage-name">Name</label>
                  <input id="stage-name" placeholder="Ada Lovelace" />
                </field>
                <field invalid>
                  <label for="stage-bad-email">Email</label>
                  <input id="stage-bad-email" value="not-an-email" />
                </field>
              </column>
            </if>
            <if {stage.label()}>
              <column gap="1" style(stageCol)>
                <label for="stage-email" required size={variant()}>Email</label>
                <input id="stage-email" placeholder="ada@lovelace.dev" />
              </column>
            </if>
            <if {stage.fieldset()}>
              <fieldset style(stageCol)>
                <legend>Account details</legend>
                <column gap="2">
                  <input placeholder="Name" />
                  <input placeholder="Email" />
                </column>
              </fieldset>
            </if>
            <if {stage.dropzone()}>
              <card dropzone style({ box: pad(6).w(full).maxW(panel-max-w-sm) })>Drop a file, or click to browse</card>
            </if>
            <if {stage.themePicker()}>
              <column gap="4" align="center">
                <column gap="1" align="center">
                  <span style(mutedXs)>Full catalog</span>
                  <theme-picker></theme-picker>
                </column>
                <column gap="1" align="center">
                  <span style(mutedXs)>Mode only</span>
                  <theme-picker mode-only></theme-picker>
                </column>
                <p style(monoXs)>themes this whole site, no reload</p>
              </column>
            </if>

            <!-- ══ Feedback ════════════════════════════════════════════ -->
            <if {stage.alert()}>
              <!-- Specimen left + prop rail (Mantine configurator). Tone swatches — not variant chips. -->
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="stretch" style(fullW)>
                    <if {alertVisible()}>
                      <if {alertTone()}>
                        <alert
                          tone={alertTone()}
                          dismissible={alertDismissible()}
                          on-dismiss={onAlertDismiss}
                          style(fullW)
                        >
                          <strong slot="title">{alertTitle()}</strong>
                          {alertBody()}
                        </alert>
                      {else =>
                        <alert
                          dismissible={alertDismissible()}
                          on-dismiss={onAlertDismiss}
                          style(fullW)
                        >
                          <strong slot="title">{alertTitle()}</strong>
                          {alertBody()}
                        </alert>}</if>
                    {else =>
                      <column gap="3" align="center">
                        <p style(mutedSm)>Dismissed — restore to try the knobs again.</p>
                        <button type="button" variant="outline" size="sm" on:click={restoreAlert}>
                          Restore alert
                        </button>
                      </column>}</if>
                    <p style(monoXs)>{alertMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="rest — component.alert tokens" placement="top">
                        <div label="rest" on:click={clearAlertTone} selected={!alertTone() ? 'true' : null} style(swatch, swatchRest)></div>
                      </tooltip>
                      <tooltip content="info" placement="top">
                        <div label="info" on:click={() => pickAlertTone('info')} selected={alertTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
                      </tooltip>
                      <tooltip content="success" placement="top">
                        <div label="success" on:click={() => pickAlertTone('success')} selected={alertTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
                      </tooltip>
                      <tooltip content="warning" placement="top">
                        <div label="warning" on:click={() => pickAlertTone('warning')} selected={alertTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickAlertTone('destructive')} selected={alertTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                      <tooltip content="muted" placement="top">
                        <div label="muted" on:click={() => pickAlertTone('muted')} selected={alertTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
                      </tooltip>
                    </row>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Dismissible</label>
                    <row gap="2" align="center">
                      <switch bind:checked={alertDismissible} size="sm" tone="primary"></switch>
                      <span style(sizeValue)>{alertDismissible() ? 'close control on' : 'sticky'}</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.progressBar()}>
              <column gap="3" style(stageCol)>
                <slider bind:value={load} min="0" max="100"></slider>
                <progress value={load()} max="100"></progress>
                <p style(monoXs)>one signal drives both — {load()}%</p>
              </column>
            </if>
            <if {stage.meter()}>
              <!-- Fixed width: meter has no intrinsic width; rail owns knobs. -->
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center" style(meterCol)>
                    <meter
                      value={meterVal()}
                      min="0"
                      max="100"
                      low="33"
                      high="66"
                      optimum="100"
                      size={meterSize()}
                    ></meter>
                    <p style(monoXs)>{meterMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Value</label>
                      <span style(sizeValue)>{meterVal()} · {meterBand()}</span>
                    </row>
                    <slider bind:value={meterVal} min="0" max="100" size="sm" label="Value"></slider>
                    <p style(mutedXs)>
                      Fixed thresholds <code>low=33</code> · <code>high=66</code> ·
                      <code>optimum=100</code> — drag value to hit optimum / suboptimum /
                      even-less-good bands.
                    </p>
                  </column>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{meterSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.spinner()}>
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center" style(fullW)>
                    <if {spinnerTone()}>
                      <spinner
                        size={spinnerSize()}
                        tone={spinnerTone()}
                        centered={spinnerCentered() ? 'true' : null}
                      ></spinner>
                    {else =>
                      <spinner
                        size={spinnerSize()}
                        centered={spinnerCentered() ? 'true' : null}
                      ></spinner>}</if>
                    <p style(monoXs)>{spinnerMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{spinnerSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>xs</span>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                    </row>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="rest — component.spinner.fg" placement="top">
                        <div label="rest" on:click={clearSpinnerTone} selected={!spinnerTone() ? 'true' : null} style(swatch, swatchRest)></div>
                      </tooltip>
                      <tooltip content="primary" placement="top">
                        <div label="primary" on:click={() => pickSpinnerTone('primary')} selected={spinnerTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
                      </tooltip>
                      <tooltip content="info" placement="top">
                        <div label="info" on:click={() => pickSpinnerTone('info')} selected={spinnerTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
                      </tooltip>
                      <tooltip content="success" placement="top">
                        <div label="success" on:click={() => pickSpinnerTone('success')} selected={spinnerTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
                      </tooltip>
                      <tooltip content="warning" placement="top">
                        <div label="warning" on:click={() => pickSpinnerTone('warning')} selected={spinnerTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickSpinnerTone('destructive')} selected={spinnerTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                      <tooltip content="accent" placement="top">
                        <div label="accent" on:click={() => pickSpinnerTone('accent')} selected={spinnerTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
                      </tooltip>
                    </row>
                    <p style(mutedXs)>Omit tone for themeable <code>spinner.foreground</code>. Track always <code>spinner.track</code>. No muted (host omits).</p>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Centered</label>
                    <row gap="2" align="center">
                      <switch bind:checked={spinnerCentered} size="sm" tone="primary"></switch>
                      <span style(sizeValue)>{spinnerCentered() ? 'block + mx auto' : 'inline'}</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.skeleton()}>
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center" style(skelStageCol)>
                    <if {variant() === 'circle'}>
                      <skeleton shape="circle" size={skelSize()}></skeleton>
                    {else-if {variant() === 'block'} =>
                      <skeleton shape="block" style(skelBlock)></skeleton>
                    {else =>
                      <column gap="2" style(fullW)>
                        <skeleton shape={variant() || 'text'} size={skelSize()}></skeleton>
                      </column>}</if>
                    <p style(monoXs)>{skelMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Shape</label>
                    <select size="sm" bind:value={variant} label="Shape">
                      <option value="text">text</option>
                      <option value="line">line</option>
                      <option value="block">block</option>
                      <option value="circle">circle</option>
                    </select>
                  </column>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{skelSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                    </row>
                    <p style(mutedXs)>
                      Size scales text/line height and circle diameter. Block has a fixed min height
                      (size is ignored). Sheen always uses <code>component.skeleton</code> tokens.
                    </p>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.emptyState()}>
              <empty-state variant={variant()} style(fullW)>
                <strong slot="title">No results</strong>
                Try a different search, or clear a filter.
              </empty-state>
            </if>
            <if {stage.errorState()}>
              <error-state variant={variant()} code="ERR_TIMEOUT" style(fullW)>
                The request timed out. Retry, or check the connection.
              </error-state>
            </if>
            <if {stage.errorBoundary()}>
              <column gap="3" style(stageCol)>
                <error-boundary message="This panel could not load.">
                  <card variant="outline" style(cardPad)>
                    <p style(mutedSm)>
                      A healthy boundary is invisible — this card is the wrapped
                      subtree. A render throw inside would swap in the fallback.
                    </p>
                  </card>
                </error-boundary>
              </column>
            </if>
            <if {stage.toast()}>
              <column gap="3" align="center" style(toastStage)>
                <button type="button" variant="outline" size="sm" on:click={pushToast} toast-show>
                  Show toast
                </button>
                <for {toastItems() as t} key={t.key}>
                  <toast tone={variant()} duration="0" on-dismiss={onToastDismiss} style(stageCol)>
                    <strong>Saved.</strong> Sticky at duration 0 — hover pauses real ones.
                  </toast>
                </for>
              </column>
            </if>

            <!-- ══ Display ═════════════════════════════════════════════ -->
            <if {stage.badge()}>
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center">
                    <if {badgeTone()}>
                      <badge
                        variant={variant()}
                        size={badgeSize()}
                        tone={badgeTone()}
                        pill={badgePill()}
                      >
                        {badgeTone()}
                      </badge>
                    {else =>
                      <badge variant={variant()} size={badgeSize()} pill={badgePill()}>
                        rest
                      </badge>}</if>
                    <p style(monoXs)>{badgeMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Variant</label>
                    <select size="sm" bind:value={variant} label="Variant">
                      <option value="soft">soft</option>
                      <option value="solid">solid</option>
                      <option value="outline">outline</option>
                    </select>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="rest — component.badge tokens" placement="top">
                        <div label="rest" on:click={clearBadgeTone} selected={!badgeTone() ? 'true' : null} style(swatch, swatchRest)></div>
                      </tooltip>
                      <tooltip content="primary" placement="top">
                        <div label="primary" on:click={() => pickBadgeTone('primary')} selected={badgeTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
                      </tooltip>
                      <tooltip content="success" placement="top">
                        <div label="success" on:click={() => pickBadgeTone('success')} selected={badgeTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
                      </tooltip>
                      <tooltip content="warning" placement="top">
                        <div label="warning" on:click={() => pickBadgeTone('warning')} selected={badgeTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickBadgeTone('destructive')} selected={badgeTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                      <tooltip content="accent" placement="top">
                        <div label="accent" on:click={() => pickBadgeTone('accent')} selected={badgeTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
                      </tooltip>
                      <tooltip content="muted" placement="top">
                        <div label="muted" on:click={() => pickBadgeTone('muted')} selected={badgeTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
                      </tooltip>
                      <tooltip content="info" placement="top">
                        <div label="info" on:click={() => pickBadgeTone('info')} selected={badgeTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
                      </tooltip>
                    </row>
                  </column>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{badgeSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                    </row>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Pill</label>
                    <row gap="2" align="center">
                      <switch bind:checked={badgePill} size="sm" tone="primary"></switch>
                      <span style(sizeValue)>{badgePill() ? 'full radius' : 'sm radius'}</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.chip()}>
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center">
                    <if {kept()}>
                      <if {chipTone()}>
                        <chip
                          variant={variant()}
                          size={chipSize()}
                          tone={chipTone()}
                          selected={chipSelected() ? 'true' : null}
                          removable={chipRemovable() ? 'true' : null}
                          on-remove={dropChip}
                        >
                          {chipTone()}
                        </chip>
                      {else =>
                        <chip
                          variant={variant()}
                          size={chipSize()}
                          selected={chipSelected() ? 'true' : null}
                          removable={chipRemovable() ? 'true' : null}
                          on-remove={dropChip}
                        >
                          rest
                        </chip>}</if>
                    {else =>
                      <button variant="ghost" size="sm" on:click={restoreChip}>bring it back</button>}</if>
                    <p style(monoXs)>{chipMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Variant</label>
                    <select size="sm" bind:value={variant} label="Variant">
                      <option value="solid">solid</option>
                      <option value="soft">soft</option>
                      <option value="outline">outline</option>
                      <option value="ghost">ghost</option>
                    </select>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="rest — component.chip tokens" placement="top">
                        <div label="rest" on:click={clearChipTone} selected={!chipTone() ? 'true' : null} style(swatch, swatchRest)></div>
                      </tooltip>
                      <tooltip content="primary" placement="top">
                        <div label="primary" on:click={() => pickChipTone('primary')} selected={chipTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
                      </tooltip>
                      <tooltip content="success" placement="top">
                        <div label="success" on:click={() => pickChipTone('success')} selected={chipTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
                      </tooltip>
                      <tooltip content="warning" placement="top">
                        <div label="warning" on:click={() => pickChipTone('warning')} selected={chipTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickChipTone('destructive')} selected={chipTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                      <tooltip content="accent" placement="top">
                        <div label="accent" on:click={() => pickChipTone('accent')} selected={chipTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
                      </tooltip>
                      <tooltip content="muted" placement="top">
                        <div label="muted" on:click={() => pickChipTone('muted')} selected={chipTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
                      </tooltip>
                      <tooltip content="info" placement="top">
                        <div label="info" on:click={() => pickChipTone('info')} selected={chipTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
                      </tooltip>
                    </row>
                  </column>
                  <column gap="2">
                    <row gap="2" align="center" justify="between">
                      <label style(knobLabel)>Size</label>
                      <span style(sizeValue)>{chipSize()}</span>
                    </row>
                    <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
                    <row gap="0" align="center" style(tickRow)>
                      <span style(tick)>sm</span>
                      <span style(tick)>md</span>
                      <span style(tick)>lg</span>
                    </row>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Selected</label>
                    <row gap="2" align="center">
                      <switch bind:checked={chipSelected} size="sm" tone="primary"></switch>
                      <span style(sizeValue)>{chipSelected() ? 'aria-pressed' : 'idle'}</span>
                    </row>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Removable</label>
                    <row gap="2" align="center">
                      <switch bind:checked={chipRemovable} size="sm" tone="primary"></switch>
                      <span style(sizeValue)>{chipRemovable() ? '× + Backspace' : 'no dismiss'}</span>
                    </row>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.card()}>
              <card variant={variant()} interactive style(cardPad5)>
                <strong>A surface</strong>
                <p style(mutedSm)>the hover lift is one attribute</p>
              </card>
            </if>
            <if {stage.avatar()}>
              <row gap="3" align="center">
                <avatar name="Ada Lovelace" size={variant()} src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
                <avatar initials="GH" tone="primary" size={variant()}></avatar>
                <avatar name="Alan Turing" size={variant()} shape="square" src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
              </row>
            </if>
            <if {stage.icon()}>
              <row gap="4" align="center">
                <icon size={variant()}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5" /></svg>
                </icon>
                <icon size={variant()}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
                </icon>
                <icon size={variant()} spin>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12a9 9 0 1 1-6.2-8.56" /></svg>
                </icon>
              </row>
            </if>
            <if {stage.separator()}>
              <row nowrap gap="0" align="stretch" style(stageSplit)>
                <center style(stageCanvas)>
                  <column gap="4" align="center" style(sepStageCol)>
                    <if {sepOrientation() === 'vertical'}>
                      <row gap="3" align="center" style(sepVRow)>
                        <span style(mutedSm)>A</span>
                        <if {sepTone()}>
                          <separator orientation="vertical" variant={variant()} tone={sepTone()}></separator>
                        {else =>
                          <separator orientation="vertical" variant={variant()}></separator>}</if>
                        <span style(mutedSm)>B</span>
                      </row>
                    {else =>
                      <column gap="2" style(fullW)>
                        <p style(mutedSm)>above</p>
                        <if {sepTone()}>
                          <separator orientation="horizontal" variant={variant()} tone={sepTone()}></separator>
                        {else =>
                          <separator orientation="horizontal" variant={variant()}></separator>}</if>
                        <p style(mutedSm)>below</p>
                      </column>}</if>
                    <p style(monoXs)>{sepMeta()}</p>
                  </column>
                </center>
                <column gap="5" style(knobPanel)>
                  <column gap="2">
                    <label style(knobLabel)>Orientation</label>
                    <select size="sm" bind:value={sepOrientation} label="Orientation">
                      <option value="horizontal">horizontal</option>
                      <option value="vertical">vertical</option>
                    </select>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Variant</label>
                    <select size="sm" bind:value={variant} label="Variant">
                      <option value="solid">solid</option>
                      <option value="dashed">dashed</option>
                    </select>
                  </column>
                  <column gap="2">
                    <label style(knobLabel)>Tone</label>
                    <row gap="2" align="center" style(swatchRow)>
                      <tooltip content="rest — component.separator.line" placement="top">
                        <div label="rest" on:click={clearSepTone} selected={!sepTone() ? 'true' : null} style(swatch, swatchRest)></div>
                      </tooltip>
                      <tooltip content="muted" placement="top">
                        <div label="muted" on:click={() => pickSepTone('muted')} selected={sepTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
                      </tooltip>
                      <tooltip content="primary" placement="top">
                        <div label="primary" on:click={() => pickSepTone('primary')} selected={sepTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
                      </tooltip>
                      <tooltip content="destructive" placement="top">
                        <div label="destructive" on:click={() => pickSepTone('destructive')} selected={sepTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
                      </tooltip>
                    </row>
                    <p style(mutedXs)>Omit tone for themeable <code>separator.line</code>. Host tones only: muted · primary · destructive.</p>
                  </column>
                </column>
              </row>
            </if>
            <if {stage.accordion()}>
              <accordion style(stageCol)>
                <strong slot="trigger">Show the details</strong>
                <p>Animated open and close — and it remembers its state.</p>
              </accordion>
            </if>
            <if {stage.kbd()}>
              <!-- Zero PE props — specimen-only stage (no knobs); paint is component.kbd.* -->
              <column gap="4" align="center" style(stageColMd)>
                <row gap="2" align="center">
                  <kbd>Ctrl</kbd>
                  <span style(mutedSm)>+</span>
                  <kbd>K</kbd>
                </row>
                <row gap="2" align="center">
                  <kbd>⌘</kbd>
                  <kbd>⇧</kbd>
                  <kbd>⌥</kbd>
                  <kbd>⌃</kbd>
                  <kbd>Esc</kbd>
                  <kbd>F12</kbd>
                </row>
                <p style(monoXs)>component.kbd · bg · foreground · border</p>
              </column>
            </if>
            <if {stage.prose()}>
              <!-- Zero PE props — specimen-only stage; host is <prose> + PROSE_MAP descendants -->
              <column gap="3" style(stageColMd)>
                <prose>
                  <h2>Plain tags, already styled</h2>
                  <p>Body copy sets the baseline — <strong>strong</strong>, <em>emphasis</em>,
                     <code>inline_code()</code> and <mark>a highlight</mark> need no classes.</p>
                  <blockquote>A quotation carries its own rule.</blockquote>
                  <ul>
                    <li>Unordered lists keep markers</li>
                    <li>Indented under the measure</li>
                  </ul>
                </prose>
                <p style(monoXs)>component.prose · foreground · descendant type scale</p>
              </column>
            </if>

            <!-- ══ Overlay ═════════════════════════════════════════════ -->
            <if {stage.tooltip()}>
              <!-- Stage is tall enough for top/bottom; long copy is fine here -->
              <tooltip content="Deletes the branch — cannot be undone" placement={variant()}>
                <button tone="destructive" variant="outline">Delete branch</button>
              </tooltip>
            </if>
            <if {stage.hoverCard()}>
              <column gap="3" align="center">
                <button variant="ghost" popovertarget="stage-hc">@ada</button>
                <hover-card id="stage-hc">
                  <card style(cardPad)>
                    <strong>Ada Lovelace</strong>
                    <p style(mutedSm)>wrote the first program.</p>
                  </card>
                </hover-card>
                <p style(monoXs)>focus-triggerable, not just mouse-only</p>
              </column>
            </if>
            <if {stage.popover()}>
              <column gap="3" align="center">
                <button popovertarget="stage-pop">Open popover</button>
                <popover id="stage-pop">
                  <card style(cardPad)>Rides the platform top layer — Escape or click outside closes it.</card>
                </popover>
                <p style(monoXs)>top layer · light dismiss · zero z-index</p>
              </column>
            </if>
            <if {stage.dropdown()}>
              <column gap="3" align="center">
                <button popovertarget="stage-menu">Actions ▾</button>
                <dropdown id="stage-menu">
                  <menu-item>Edit</menu-item>
                  <menu-item>Duplicate</menu-item>
                  <menu-item tone="destructive">Delete</menu-item>
                </dropdown>
                <p style(monoXs)>arrows rove · type-ahead · Escape closes</p>
              </column>
            </if>
            <if {stage.contextMenu()}>
              <column gap="3" align="center" style(stageCol)>
                <card id="stage-ctx" variant="outline" style(ctxCard)>
                  <center><p style(mutedSm)>right-click target</p></center>
                </card>
                <context-menu for="stage-ctx">
                  <menu-item value="cut">Cut</menu-item>
                  <menu-item value="copy">Copy</menu-item>
                  <menu-item value="paste">Paste</menu-item>
                </context-menu>
              </column>
            </if>
            <if {stage.dialog()}>
              <column gap="3" align="center">
                <button command="show-modal" commandfor="stage-dialog" style(pressBtn)>
                  Open the {variant()}
                </button>
                <p style(monoXs)>Escape and the backdrop close it — that’s the platform</p>
                <dialog id="stage-dialog" side={sheetSide()} size="md">
                  <header>Delete project</header>
                  <section>
                    This cannot be undone. The platform owns the top layer, focus trap, Escape, and backdrop — zero custom JS.
                  </section>
                  <footer>
                    <button command="close" commandfor="stage-dialog" variant="ghost" size="sm">Cancel</button>
                    <button command="close" commandfor="stage-dialog" tone="destructive" variant="solid" size="sm">Delete</button>
                  </footer>
                </dialog>
              </column>
            </if>

            <!-- ══ Navigation ══════════════════════════════════════════ -->
            <if {stage.tabs()}>
              <tabs style(fullW)>
                <tab-list variant={variant()}>
                  <tab>Overview</tab>
                  <tab>Activity</tab>
                  <tab>Settings</tab>
                </tab-list>
                <tab-panel><p style(mutedSm)>Arrows rove, Home and End jump — wired for you.</p></tab-panel>
                <tab-panel><p style(mutedSm)>The activity feed panel.</p></tab-panel>
                <tab-panel><p style(mutedSm)>The settings panel.</p></tab-panel>
              </tabs>
            </if>
            <if {stage.breadcrumb()}>
              <breadcrumb>
                <a href="/">Home</a>
                <a href="/components">Components</a>
                <small>breadcrumb</small>
              </breadcrumb>
            </if>
            <if {stage.pagination()}>
              <column gap="3" align="center">
                <pagination value={pageNo()} on-page-change={goPage}>
                  <page value="1">1</page>
                  <page value="2">2</page>
                  <page value="3">3</page>
                  <page value="4">4</page>
                </pagination>
                <p style(monoXs)>on page {pageNo()}</p>
              </column>
            </if>
            <if {stage.navbar()}>
              <navbar bordered style(fullW)>
                <strong slot="brand">SHIP</strong>
                <a href="/components">Docs</a>
                <a href="/components">Components</a>
                <button slot="actions" size="sm" variant="outline">Sign in</button>
              </navbar>
            </if>

            <!-- ══ Data ════════════════════════════════════════════════ -->
            <if {stage.table()}>
              <table style(fullW)>
                <thead>
                  <tr><th>Name</th><th>Team</th><th>Status</th></tr>
                </thead>
                <tbody>
                  <for {people() as p} key={p.id}>
                    <tr>
                      <td>{p.name}</td>
                      <td>{p.team}</td>
                      <td><badge tone={p.status === 'active' ? 'success' : 'muted'} size="sm">{p.status}</badge></td>
                    </tr>
                  </for>
                </tbody>
              </table>
            </if>

            <!-- ══ Layout ══════════════════════════════════════════════ -->
            <if {stage.layout()}>
              <column gap="3" style(stageColMd)>
                <row gap="2" align="center" justify="between">
                  <strong>row — the x axis</strong>
                  <badge tone="primary">end</badge>
                </row>
                <grid cols="3" gap="2">
                  <card style({ box: pad(3) })><center>1</center></card>
                  <card style({ box: pad(3) })><center>2</center></card>
                  <card style({ box: pad(3) })><center>3</center></card>
                </grid>
                <center style(centerBox)>
                  <p style(mutedSm)>center — both axes at once</p>
                </center>
              </column>
            </if>
            <if {stage.scrollArea()}>
              <scroll-area style(scrollDemo)>
                <p>Constrain it, and the overflow scrolls behind thin, themed scrollbars.</p>
                <p>Line two.</p>
                <p>Line three.</p>
                <p>Line four.</p>
                <p>Line five.</p>
                <p>Line six.</p>
                <p>Line seven.</p>
                <p>Line eight.</p>
                <p>Line nine.</p>
                <p>Line ten — still scrolling.</p>
              </scroll-area>
            </if>
            <if {stage.ticker()}>
              <ticker duration="normal" style(ticker)>
                <p style(tickerText)>
                  tokens · signals · a11y · hot reload · forms ·&nbsp;
                </p>
              </ticker>
            </if>

            <!-- ══ Pack components ═══════════════════════════════════════ -->
            <if {stage.sailSparkline()}>
              <column gap="3" align="center" tile-stage="sail-pack-sparkline">
                <sail-sparkline data="2,5,3,8,4,6,7" width="200" height="60"></sail-sparkline>
              </column>
            </if>
            <if {stage.sailAreaChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-area-chart">
                <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-area-chart>
              </column>
            </if>
            <if {stage.sailBarChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-bar-chart">
                <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="200" height="120"></sail-bar-chart>
              </column>
            </if>
            <if {stage.sailDonut()}>
              <column gap="3" align="center" tile-stage="sail-pack-donut">
                <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="120" thickness="12"></sail-donut>
              </column>
            </if>
            <if {stage.sailGauge()}>
              <column gap="3" align="center" tile-stage="sail-pack-gauge">
                <sail-gauge value="72" diameter="120" thickness="12"></sail-gauge>
              </column>
            </if>
            <if {stage.sailHeatmap()}>
              <column gap="3" align="center" tile-stage="sail-pack-heatmap">
                <sail-heatmap rows="4" cols="6" values="1,2,3,4,3,2,4,5,6,5,4,3,2,1,2,3,4,3,2,1,2,3,4,5" cell-size="22" gap="3" tone="primary"></sail-heatmap>
              </column>
            </if>
            <if {stage.sailLineChart()}>
              <column gap="3" align="center" tile-stage="sail-pack-line-chart">
                <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="200" height="120"></sail-line-chart>
              </column>
            </if>
            <if {stage.sailMediaImage()}>
              <column gap="3" align="center" tile-stage="sail-pack-media-image" style(mediaStage)>
                <sail-media-image alt="Harbor at dusk" fit="cover" zoomable={false} style(fullMedia)></sail-media-image>
              </column>
            </if>
            <if {stage.sailMediaAv()}>
              <column gap="3" align="center" tile-stage="sail-pack-media-av">
                <sail-media-av></sail-media-av>
              </column>
            </if>
            <if {stage.sailCodeEditor()}>
              <column gap="3" align="center" tile-stage="sail-pack-code-editor">
                <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="md"></sail-code-editor>
              </column>
            </if>
            <if {stage.sailMermaid()}>
              <column gap="3" align="center" tile-stage="sail-pack-mermaid">
                <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
              </column>
            </if>
            <if {stage.sailMessage()}>
              <column gap="3" tile-stage="sail-pack-message" style(stageColMd)>
                <sail-message from="user" status="sent">Can you summarize yesterday’s deploy?</sail-message>
                <sail-message from="assistant" status="sent">v0.4.2 went out at 14:02 — 3 services, zero rollbacks.</sail-message>
              </column>
            </if>
            <if {stage.sailCitation()}>
              <column gap="3" align="center" tile-stage="sail-pack-citation">
                <sail-citation source="docs.ship.dev" href="https://docs.ship.dev">Deploy pipeline overview</sail-citation>
              </column>
            </if>
            <if {stage.sailComposer()}>
              <column gap="3" tile-stage="sail-pack-composer" style(stageColMd)>
                <sail-composer placeholder="Message the agent…" style({ box: w(full).minW(0) })></sail-composer>
              </column>
            </if>
            <if {stage.sailThinkingDots()}>
              <column gap="3" align="center" tile-stage="sail-pack-thinking-dots" style(stageCol)>
                <row gap="2" align="center">
                  <sail-thinking-dots></sail-thinking-dots>
                  <span style({ paint: fg(muted-foreground), text: fontSize(sm) })>Agent is thinking…</span>
                </row>
              </column>
            </if>
            <if {stage.sailToolCall()}>
              <column gap="3" align="center" tile-stage="sail-pack-tool-call">
                <sail-tool-call name="list_deploys" status="ok" open>
                  {`{ "env": "prod", "limit": 5 }`}
                  <span slot="result">{`[{ "id": "d-1842", "status": "healthy" }]`}</span>
                </sail-tool-call>
              </column>
            </if>
            <if {stage.sailCanvas()}>
              <column gap="3" align="center" tile-stage="sail-pack-canvas">
                <sail-canvas width="200" height="120"></sail-canvas>
              </column>
            </if>
            <if {stage.sailBoard()}>
              <column gap="3" align="center" tile-stage="sail-pack-board">
                <sail-board width="200" height="120">
                  <sail-canvas width="200" height="120"></sail-canvas>
                </sail-board>
              </column>
            </if>
            <if {stage.sailDraw()}>
              <column gap="3" align="center" tile-stage="sail-pack-draw">
                <sail-draw width="200" height="120"></sail-draw>
              </column>
            </if>
            <if {stage.sailGraphCanvas()}>
              <column gap="3" align="center" tile-stage="sail-pack-graph-canvas">
                <sail-graph-canvas width="200" height="120" nodes='[{"id":"a","kind":"agent","title":"A","x":80,"y":60},{"id":"b","kind":"skill","title":"B","x":120,"y":60}]' edges="[]"></sail-graph-canvas>
              </column>
            </if>
            <if {stage.sailAnnotationLayer()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-layer">
                <sail-annotation-layer></sail-annotation-layer>
              </column>
            </if>
            <if {stage.sailAnnotationList()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-list">
                <sail-annotation-list></sail-annotation-list>
              </column>
            </if>
            <if {stage.sailAnnotationMarker()}>
              <column gap="3" align="center" tile-stage="sail-pack-annotation-marker">
                <sail-annotation-marker x="50" y="30"></sail-annotation-marker>
              </column>
            </if>

      </column>
      <if {slotCopy()}>
        <p slot="stage" style(monoXs)>{slotCopy()}</p>
      </if>

      <!-- Walkthroughs: prose + code (after usage/props). Not a second live matrix. -->
      <if {stage.button()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>With icons</h3>
            <p style(exampleLead)>
              Children compose: put <code>&lt;icon&gt;</code> before or after the label.
              The button is inline-flex with gap — no left/right section props.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary">
                <icon name="check" size="sm"></icon>
                Save
              </button>
              <button variant="outline">
                Export
                <icon name="download" size="sm"></icon>
              </button>
            </row>
            <code-block code={btnExWithIcon()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Icon-only</h3>
            <p style(exampleLead)>
              Use <code>size="icon"</code>, <code>iconSm</code>, or <code>iconLg</code> for a square hit target.
              Always set <code>label</code> for accessible name when there is no text.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary" size="icon" label="Settings">
                <icon name="settings"></icon>
              </button>
              <button variant="ghost" size="iconSm" label="Search">
                <icon name="search" size="sm"></icon>
              </button>
              <button variant="outline" size="iconLg" label="Add">
                <icon name="plus" size="lg"></icon>
              </button>
            </row>
            <code-block code={btnExIconOnly()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Sizes</h3>
            <p style(exampleLead)>
              Text controls: <code>xs</code>–<code>lg</code>. Defaults to <code>md</code>.
              Drive the live stage above for the full matrix; these are the named steps.
            </p>
            <row gap="2" align="center">
              <button variant="solid" tone="primary" size="xs">xs</button>
              <button variant="solid" tone="primary" size="sm">sm</button>
              <button variant="solid" tone="primary" size="md">md</button>
              <button variant="solid" tone="primary" size="lg">lg</button>
            </row>
            <code-block code={btnExSizes()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.badge()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Status in a table cell</h3>
            <p style(exampleLead)>
              Soft + tone for readable density. Omit tone when you want the theme’s
              <code>component.badge</code> tokens (rest swatch on the stage).
            </p>
            <row gap="2" align="center">
              <badge variant="soft" tone="success" size="sm">active</badge>
              <badge variant="outline" tone="muted" size="sm">away</badge>
              <badge variant="soft" tone="warning" size="sm">pending</badge>
            </row>
            <code-block code={bdgExStatus()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Pill counters</h3>
            <p style(exampleLead)>
              <code>pill</code> sets full radius — counts, filters, “new” marks.
            </p>
            <row gap="2" align="center">
              <badge variant="solid" tone="primary" pill>12</badge>
              <badge variant="soft" tone="destructive" pill>3</badge>
              <badge variant="outline" tone="accent" pill>beta</badge>
            </row>
            <code-block code={bdgExPill()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest tokens (themeable)</h3>
            <p style(exampleLead)>
              No <code>tone</code> → soft/solid/outline paint from <code>component.badge.*</code>.
              Override those leaves in Studio themes to rebrand every rest badge.
            </p>
            <row gap="2" align="center">
              <badge variant="soft">soft rest</badge>
              <badge variant="solid">solid rest</badge>
              <badge variant="outline">outline rest</badge>
            </row>
            <code-block code={bdgExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.separator()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Both orientations</h3>
            <p style(exampleLead)>
              Horizontal rules sections; vertical sits between inline items (give the row a height
              so the line can stretch). Defaults: <code>orientation="horizontal"</code>,
              <code>variant="solid"</code>, no tone.
            </p>
            <column gap="2" style(sepStageCol)>
              <p style(mutedSm)>Section A</p>
              <separator></separator>
              <p style(mutedSm)>Section B</p>
              <row gap="3" align="center" style(sepVRow)>
                <span style(mutedSm)>Left</span>
                <separator orientation="vertical"></separator>
                <span style(mutedSm)>Right</span>
              </row>
            </column>
            <code-block code={sepExBoth()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Dashed and tone</h3>
            <p style(exampleLead)>
              <code>variant="dashed"</code> draws a border stroke instead of a filled bar.
              Tone paints only when set — host subset is muted · primary · destructive.
            </p>
            <column gap="3" style(sepStageCol)>
              <separator variant="dashed"></separator>
              <separator variant="solid" tone="primary"></separator>
              <separator variant="dashed" tone="destructive"></separator>
            </column>
            <code-block code={sepExVariant()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest line (themeable)</h3>
            <p style(exampleLead)>
              No <code>tone</code> → paint from <code>component.separator.line</code>
              (<code>--separator-line</code>). Override that leaf in Studio themes to rebrand every rest divider.
            </p>
            <column gap="2" style(sepStageCol)>
              <separator></separator>
              <separator variant="dashed"></separator>
            </column>
            <code-block code={sepExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.kbd()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>In prose chords</h3>
            <p style(exampleLead)>
              Drop native <code>kbd</code> inline. Compose combinations with plain text between keycaps —
              no group component and no size/variant props.
            </p>
            <p style(mutedSm)>
              Press <kbd>Ctrl</kbd> + <kbd>K</kbd> to open the command palette.
            </p>
            <code-block code={kbdExChord()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Modifiers and function keys</h3>
            <p style(exampleLead)>
              Glyphs and short labels share the same mono keycap chrome (doubled bottom edge).
            </p>
            <row gap="2" align="center">
              <kbd>⌘</kbd>
              <kbd>⇧</kbd>
              <kbd>⌥</kbd>
              <kbd>⌃</kbd>
              <kbd>Esc</kbd>
              <kbd>F12</kbd>
            </row>
            <code-block code={kbdExModifiers()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Themeable tokens</h3>
            <p style(exampleLead)>
              Paint is always <code>component.kbd</code> leaves
              (<code>--kbd</code>, <code>--kbd-foreground</code>, <code>--kbd-border</code>).
              Override those in Studio themes — there is no tone prop on the tag.
            </p>
            <row gap="2" align="center">
              <kbd>Enter</kbd>
              <kbd>Tab</kbd>
              <kbd>/</kbd>
            </row>
            <code-block code={kbdExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.prose()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Type scale under the scope</h3>
            <p style(exampleLead)>
              Wrap long copy in <code>&lt;prose&gt;</code>. Nested <code>h1</code>…<code>h4</code> and
              <code>p</code> pick up the framework type scale — no size or variant props on the host.
            </p>
            <prose style(stageColMd)>
              <h1>Heading one</h1>
              <h2>Heading two</h2>
              <h3>Heading three</h3>
              <p>Body sets the baseline measure and leading.</p>
            </prose>
            <code-block code={proseExScale()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Inlines and blocks</h3>
            <p style(exampleLead)>
              Strong, emphasis, code, mark, blockquote, and lists are plain tags. Taste comes from the
              prose descendant map (not hand classes or a second Title/Text API).
            </p>
            <prose style(stageColMd)>
              <p>
                Inline <strong>strong</strong>, <em>emphasis</em>,
                <code>inline_code()</code>, and <mark>a highlight</mark>.
              </p>
              <blockquote>A quotation carries its own rule.</blockquote>
              <ul>
                <li>Unordered item</li>
                <li>Another item</li>
              </ul>
              <ol>
                <li>Ordered item</li>
                <li>Another item</li>
              </ol>
            </prose>
            <code-block code={proseExBlocks()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Themeable host leaf</h3>
            <p style(exampleLead)>
              Host ink is <code>component.prose.fg</code> (<code>--prose-foreground</code>).
              Descendants use shared design tokens (heading font, text steps, primary link).
              No tone/size ladder on the tag — theme the leaf and type roles.
            </p>
            <prose style(stageColMd)>
              <p>Markdown blobs, docs bodies, and rendered content share this scope.</p>
            </prose>
            <code-block code={proseExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.alert()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Status tones</h3>
            <p style(exampleLead)>
              Tone brands the left stripe and icon colour. Host fill stays
              <code>component.alert</code> tokens so themes stay coherent.
              <code>warning</code> / <code>destructive</code> flip liveRegion to
              assertive <code>role="alert"</code>.
            </p>
            <column gap="2" style(fullW)>
              <alert tone="info">
                <strong slot="title">Heads up</strong>
                A new version is available.
              </alert>
              <alert tone="success">
                <strong slot="title">Saved</strong>
                All changes were written.
              </alert>
            </column>
            <code-block code={alrtExTones()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Dismissible</h3>
            <p style(exampleLead)>
              <code>dismissible</code> shows the close control.
              <code>onDismiss</code> fires before the host removes itself — keep the
              instance in a list/signal if you need to restore it.
            </p>
            <alert tone="warning" dismissible>
              <strong slot="title">Check your input</strong>
              Two fields need attention before you continue.
            </alert>
            <code-block code={alrtExDismiss()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest tokens (themeable)</h3>
            <p style(exampleLead)>
              No <code>tone</code> → bg/fg/border/stripe from <code>component.alert.*</code>.
              Override those leaves in Studio themes to rebrand every rest alert.
            </p>
            <alert>
              <strong slot="title">Notice</strong>
              Rest paint — component.alert.* leaves.
            </alert>
            <code-block code={alrtExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.chip()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Filter selection</h3>
            <p style(exampleLead)>
              <code>selected</code> is the pressed state (ARIA-synced). Use tone for brand
              filters; outline + muted for idle peers in a strip.
            </p>
            <row gap="2" align="center">
              <chip variant="solid" tone="primary" selected>Design</chip>
              <chip variant="outline" tone="muted">Research</chip>
              <chip variant="soft" tone="info">Ops</chip>
            </row>
            <code-block code={chipExFilter()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Removable tags</h3>
            <p style(exampleLead)>
              <code>removable</code> shows the ×; click or Backspace/Delete fires
              <code>on-remove</code>. The stage above remounts after dismiss.
            </p>
            <row gap="2" align="center">
              <chip variant="soft" tone="info" removable>typescript</chip>
              <chip variant="solid" tone="success" removable size="sm">shipped</chip>
            </row>
            <code-block code={chipExRemove()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest tokens (themeable)</h3>
            <p style(exampleLead)>
              No <code>tone</code> → solid/soft/outline/ghost paint from
              <code>component.chip.*</code>. Override those leaves in Studio themes.
            </p>
            <row gap="2" align="center">
              <chip variant="solid">solid rest</chip>
              <chip variant="soft">soft rest</chip>
              <chip variant="outline">outline rest</chip>
              <chip variant="ghost">ghost rest</chip>
            </row>
            <code-block code={chipExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.meter()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Labeled measurement</h3>
            <p style(exampleLead)>
              A gauge for a quantity in range — disk free, battery, score — not task
              completion. Pair with copy; the native element exposes the measurement to
              AT via <code>value</code> / <code>min</code> / <code>max</code>.
            </p>
            <column gap="2" style(meterCol)>
              <p style(mutedSm)>Disk free — 72%</p>
              <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>
            </column>
            <code-block code={mtrExMeasure()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Size ladder</h3>
            <p style(exampleLead)>
              <code>sm</code> · <code>md</code> · <code>lg</code> scale bar height
              (same ladder as progress). Default is <code>md</code>. Width comes from
              the parent — give the host a definite width.
            </p>
            <column gap="3" style(meterCol)>
              <meter value="0.55" size="sm"></meter>
              <meter value="0.55"></meter>
              <meter value="0.55" size="lg"></meter>
            </column>
            <code-block code={mtrExSizes()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Optimum bands</h3>
            <p style(exampleLead)>
              <code>low</code> / <code>high</code> / <code>optimum</code> drive the
              platform’s three fill regions. Paint maps through
              <code>component.meter.optimum</code>, <code>suboptimum</code>, and
              <code>sub-suboptimum</code> (METER_MAP skins).
            </p>
            <column gap="3" style(meterCol)>
              <column gap="1">
                <p style(monoXs)>value 20 · even-less-good</p>
                <meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>
              </column>
              <column gap="1">
                <p style(monoXs)>value 50 · suboptimum</p>
                <meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>
              </column>
              <column gap="1">
                <p style(monoXs)>value 85 · optimum</p>
                <meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>
              </column>
            </column>
            <code-block code={mtrExBands()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest tokens (themeable)</h3>
            <p style(exampleLead)>
              Track from <code>component.meter.track</code> (<code>--meter</code>);
              bands from the three optimum leaves. Override those leaves in Studio
              themes — there is no tone prop.
            </p>
            <column gap="2" style(meterCol)>
              <meter value="0.55"></meter>
            </column>
            <code-block code={mtrExRest()}></code-block>
          </column>
        </column>
      </if>

      <if {stage.spinner()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Inline with status copy</h3>
            <p style(exampleLead)>
              Pair a small spinner with a short wait message. The ring is decorative
              (<code>aria-hidden</code>); announce loading in surrounding copy or a live region.
            </p>
            <row gap="2" align="center">
              <spinner size="sm" tone="primary"></spinner>
              <p style(mutedSm)>fetching…</p>
            </row>
            <code-block code={spnExInline()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Size ladder</h3>
            <p style(exampleLead)>
              <code>xs</code> · <code>sm</code> · <code>md</code> · <code>lg</code> map to the
              icon-size scale. Default is <code>md</code>.
            </p>
            <row gap="4" align="center">
              <spinner size="xs" tone="primary"></spinner>
              <spinner size="sm" tone="primary"></spinner>
              <spinner tone="primary"></spinner>
              <spinner size="lg" tone="primary"></spinner>
            </row>
            <code-block code={spnExSizes()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Rest tokens (themeable)</h3>
            <p style(exampleLead)>
              No <code>tone</code> → active arc from <code>component.spinner.fg</code>
              (<code>--spinner-foreground</code>); track from <code>component.spinner.track</code>.
              Override those leaves in Studio themes.
            </p>
            <row gap="4" align="center">
              <spinner></spinner>
              <spinner size="lg"></spinner>
            </row>
            <code-block code={spnExRest()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Centered in a container</h3>
            <p style(exampleLead)>
              <code>centered</code> makes the host a block with auto horizontal margins —
              useful inside a panel or empty stage.
            </p>
            <column gap="2" style(centerBox)>
              <spinner centered tone="primary"></spinner>
            </column>
            <code-block code={spnExCentered()}></code-block>
          </column>
        </column>
      </if>
      <if {stage.skeleton()}>
        <column gap="5" style(exampleBlock)>
          <p style(sectionLabel)>Examples</p>

          <column gap="2">
            <h3 style(exampleH)>Text lines</h3>
            <p style(exampleLead)>
              Stack <code>shape="text"</code> / <code>line</code> bars for a paragraph
              placeholder. Width comes from the parent — PE defaults to full width of the
              containing box.
            </p>
            <column gap="2" style(skelStageCol)>
              <skeleton shape="text" size="md"></skeleton>
              <skeleton shape="text" size="sm"></skeleton>
              <skeleton shape="line" size="sm"></skeleton>
            </column>
            <code-block code={skelExLines()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Avatar row</h3>
            <p style(exampleLead)>
              Compose <code>circle</code> with text bars for a list-item or card header
              loading state. Size on circle sets diameter; text size sets bar height.
            </p>
            <row gap="3" align="center" style(skelStageCol)>
              <skeleton shape="circle" size="md"></skeleton>
              <column gap="2" style(grow1)>
                <skeleton shape="text" size="sm"></skeleton>
                <skeleton shape="text" size="sm"></skeleton>
              </column>
            </row>
            <code-block code={skelExAvatar()}></code-block>
          </column>

          <column gap="2">
            <h3 style(exampleH)>Sheen tokens (themeable)</h3>
            <p style(exampleLead)>
              Band colours always ride <code>component.skeleton.bg</code> /
              <code>highlight</code> (<code>--skeleton</code> / <code>--skeleton-highlight</code>).
              Override those leaves in Studio themes — there is no tone prop.
            </p>
            <column gap="2" style(skelStageCol)>
              <skeleton shape="block"></skeleton>
              <skeleton shape="text" size="md"></skeleton>
            </column>
            <code-block code={skelExRest()}></code-block>
          </column>
        </column>
      </if>
    </sail-component-viewer>

    <row gap="2">
      <a variant="default" href="/components">← All components</a>
      <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
    </row>
  {else => <breadcrumb>
      <a href="/components">components</a>
      <small>{tag()}</small>
    </breadcrumb>
    <empty-state style(emptyEnter)>
      <strong slot="title">Nothing named “{tag()}” here</strong>
      Every element on the L1 surface has a detail page. Pack components
      (the 6 un-gated packs) render here too — this looks like a typo.
    </empty-state>
    <row gap="2">
      <a variant="solid" tone="primary" href="/components">← All components</a>
    </row>}</if>
</column>

