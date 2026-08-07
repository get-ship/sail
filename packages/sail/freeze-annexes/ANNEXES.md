# SAIL Syntax Freeze — Annexes A–E (generated)

> **Do not hand-edit.** Regenerate: `node scripts/gen-freeze-annexes.mjs`
> Tip: `ee95d172a16fe49b00c877750922ba88e2734c12` · Generated: 2026-08-07 · Content hash: `9945c72d5659c445`

Normative freeze prose: mission scratch `SAIL-SYNTAX-FREEZE.md`. These files are the
machine-readable inventory the freeze annexes point at.

## Summary

| Annex | Title | Count | File |
| --- | --- | --- | --- |
| A | L1 element inventory | 55 | a-l1-elements.json |
| B | Token inventory | 435 | b-tokens.json |
| C | Public action catalog | 11 | c-public-actions.json |
| D | Runtime public exports | 125 | d-runtime-exports.json |
| E | Channel vocabulary | 164 | e-channel-vocabulary.json |


## A. L1 element inventory (55)

Source: compiler element manifest / stdlib (`language-surface.gen.ts` ELEMENTS).

| Tag | Description |
| --- | --- |
| `a` | native <a>. Bare (no variant) = text link (sail-base color). With |
| `accordion` | ONE disclosure element (replaces collapsible + details). |
| `alert` | L1 inline status banner. Style value language only. |
| `avatar` | L1 language element. Style value language only. |
| `badge` | L1 language element. Style value language only: |
| `breadcrumb` | trail host. ol/li/a anatomy + separators via compiler |
| `button` | native <button>. L1 platform control. Style value language only. |
| `card` | L1 language element (element mode). Generic surface: header/body/footer |
| `chip` | filter/selection pill. |
| `code-block` | syntax-highlighted source. |
| `combobox` | L1 filtering picker (element mode). A text input |
| `context-menu` | L1 right-click menu. Native [popover] + use:contextMenu |
| `dialog` | native modal / sheet. Style product only. |
| `dropdown` | L1 anchored menu (element mode). |
| `empty-state` | L1 centered empty placeholder. |
| `error-boundary` | group-caught swap on Style when. Fallback chrome from |
| `error-state` | L1 recovery surface. Style value language only. |
| `field` | stack+gap layout; hint/error gates via structure Style. |
| `fieldset` | form group. Style product only. |
| `hover-card` | L1 hover-preview overlay. Style value language only. |
| `icon` | glyph box + size match. use:icon owns spin. glyphFill packs child svg. |
| `input` | native field / toggle / range surface. Style product only. |
| `kbd` | mono keycap chip. Style product only. |
| `label` | size match; weight token; .req gate via group-required when. |
| `legend` | fieldset caption. Style product only. |
| `menu-item` | item inside dropdown/context-menu. Style product host paint. |
| `meter` | gauge (value-in-range). Style product only. |
| `navbar` | application nav bar. Placements top/rail/bottom; sticky; bordered. |
| `page` | size match owns cell geometry (stamped from pagination). |
| `pagination` | nav state via use:pagination (ARIA current). Size is an ordinary |
| `popover` | L1 anchored overlay. Style value language only. |
| `progress` | determinate / indeterminate bar. Style product only. |
| `prose` | typography scope for rendered content (markdown blobs etc.). |
| `radio` | label + native radio. Root stays <label> for native semantics. |
| `radio-group` | L1 language element (element mode). A vertical group of |
| `scroll-area` | overflow axes + standard scrollbar pack on host Style. |
| `select` | native picker. Field surface + size ladder (Style). |
| `separator` | native <hr>. Style value language only. |
| `skeleton` | loading placeholder sheen. Style product for geometry; |
| `slider` | native range alias. use:rangeFill → --rng-pct (WebKit fill). |
| `spinner` | L1 language element. Style value language only: |
| `switch` | native checkbox restyled as sliding toggle. No controller. |
| `tab` | paint + variant/size Style match for pill/enclosed and pad scale. |
| `tab-list` | strip chrome. Sticky + variant Style. Orientation structural stamp from use:tabs. |
| `tab-panel` | orientation Style match (stamped by use:tabs). |
| `table` | structural data table. Style product only. |
| `tabs` | tab strip + panels. use:tabs owns behavior + stamps orientation |
| `textarea` | native multi-line field. Style product only. |
| `theme-picker` | catalog + optional mode-only. |
| `ticker` | infinite linear marquee. Host overflow Style; track is a nowrap |
| `toast` | auto-dismiss + close. Style value language only. |
| `toast-region` | fixed stacking region. Style value language only. |
| `toggle` | pressed-button cell for <toggle-group> only (standalone died). |
| `toggle-group` | segmented Style match; structural orientation stamp on cells only. |
| `tooltip` | hover/focus text bubble. use:tipDescribe supplies per-instance |


Reskins: **0** (retired — empty). Stdlib files: 55.

## B. Token inventory (435)

Source: `packages/sail/tokens/src/token-registry.json`.

| DTCG type | Count |
| --- | --- |
| color | 226 |
| dimension | 125 |
| number | 33 |
| shadow | 15 |
| fontFamily | 13 |
| string | 10 |
| duration | 8 |
| cubicBezier | 5 |


| Path root | Count |
| --- | --- |
| component | 135 |
| color | 107 |
| typography | 46 |
| space | 34 |
| panel | 29 |
| motion | 13 |
| control | 12 |
| shadow | 12 |
| icon | 9 |
| z | 9 |
| interaction | 9 |
| radius | 6 |
| scrollbar | 6 |
| density | 5 |
| breakpoint | 3 |


Product axis: `brand-hue` (entries: 5).
Aliases: 0.

Full name list: [`b-tokens.json`](./b-tokens.json) (`names` array).

## C. Public action catalog (11)

Source: `action_manifest::PUBLIC_ACTION_NAMES` / `PUBLIC_ACTIONS` surface.

| Name | Param | Summary |
| --- | --- | --- |
| `anchor` | AnchorPlacement \| undefined | Position a popper-like surface against an anchor element. |
| `autosize` | void | Autosize a textarea (height) or single-line input (width) to its content. |
| `clickOutside` | (() => void) \| undefined | Fire a callback when a click lands outside the element. |
| `clipboard` | ClipboardParam | Copy param text (or host text content) to the clipboard on click; optional onCopy success callback. |
| `contextMenu` | void | Right-click menu: open on contextmenu, close on outside-click / ESC. |
| `dropzone` | undefined | Drag-and-drop file receiver; reads files into a controller-side handler. |
| `hotkeys` | HotkeysMap \| undefined | Declarative key→handler map on the document; ignores editable fields unless allowInInputs. |
| `rect` | BoxRect \| null \| undefined | Track the element's bounding rect via ResizeObserver. |
| `tipDescribe` | void | Hover/focus-triggered tooltip; open on pointer-in, close on pointer-out with delay. |
| `virtualRow` | number \| null \| undefined | Register a virtualized row; tells the parent scroller its measured height. |
| `zoomPan` | ZoomPanState \| null \| undefined | Pinch/wheel zoom + drag pan on a surface; emits state changes to the controller. |


Sanctioned pre-RC additions (not yet in catalog): `focus`, `inView`.

Manifest drift surface↔PUBLIC_ACTION_NAMES: surfaceOnly=[], manifestOnly=[].

## D. Runtime public exports (125)

Source: `packages/sail/runtime/author-surface.json` (tier B).

| Category | Count |
| --- | --- |
| actions | 20 |
| components | 8 |
| compositions | 5 |
| controller | 8 |
| data | 15 |
| errors | 6 |
| host | 2 |
| motion | 18 |
| reactivity | 20 |
| store | 7 |
| theme | 16 |


| Export | Category |
| --- | --- |
| `Action` | actions |
| `ActionResult` | actions |
| `anchor` | actions |
| `AnchorPlacement` | actions |
| `attachBoundary` | errors |
| `autosize` | actions |
| `batch` | reactivity |
| `BoundaryHandle` | errors |
| `BoundarySource` | errors |
| `clearThemeOverrides` | theme |
| `clearThemeScope` | theme |
| `clickOutside` | actions |
| `clipboard` | actions |
| `ClipboardParam` | actions |
| `clock` | motion |
| `ClockOptions` | motion |
| `ClockSignal` | motion |
| `component` | components |
| `ComponentDef` | components |
| `ComponentFactory` | components |
| `ComponentSchema` | components |
| `ComponentSetupContext` | components |
| `contextKey` | reactivity |
| `ContextKey` | reactivity |
| `contextMenu` | actions |
| `controller` | controller |
| `ControllerDef` | controller |
| `ControllerExport` | controller |
| `ControllerScope` | controller |
| `ControllerSetup` | controller |
| `debounce` | reactivity |
| `defineComponent` | components |
| `defineStore` | store |
| `derived` | reactivity |
| `Derived` | reactivity |
| `detachBoundary` | errors |
| `Disposer` | actions |
| `dropzone` | actions |
| `effect` | reactivity |
| `EffectScope` | reactivity |
| `filteredList` | compositions |
| `findNearestBoundary` | errors |
| `form` | compositions |
| `FormOptions` | compositions |
| `Host` | host |
| `HotkeyEntry` | actions |
| `HotkeyHandler` | actions |
| `hotkeys` | actions |
| `HotkeysMap` | actions |
| `hydrateTheme` | theme |
| `injectContext` | reactivity |
| `interval` | reactivity |
| `isSignal` | reactivity |
| `ListSource` | compositions |
| `LiveSignal` | data |
| `LiveSource` | data |
| `LiveState` | data |
| `LiveValueOf` | data |
| `LoweredThemeToken` | theme |
| `lowerThemeDoc` | theme |
| `LowerThemeResult` | theme |
| `mountAmbient` | reactivity |
| `MutableState` | controller |
| `mutation` | data |
| `Mutation` | data |
| `MutationOpts` | data |
| `MutationState` | data |
| `onCleanup` | reactivity |
| `OptimisticHandle` | data |
| `peekStore` | store |
| `persisted` | reactivity |
| `PersistedOpts` | reactivity |
| `primeIconCache` | actions |
| `prop` | components |
| `PropDef` | components |
| `provideContext` | reactivity |
| `provideStore` | store |
| `query` | data |
| `QueryKey` | data |
| `queryKeyed` | data |
| `QueryOpts` | data |
| `ReadonlyState` | controller |
| `ReadonlyView` | controller |
| `rect` | actions |
| `RouteSignals` | host |
| `runProtected` | errors |
| `SailMountOutsideSetupError` | reactivity |
| `SailTimelineError` | motion |
| `scrollProgress` | motion |
| `ScrollProgressOptions` | motion |
| `selection` | compositions |
| `setThemePersist` | theme |
| `signal` | reactivity |
| `Signal` | reactivity |
| `snapshotStore` | store |
| `SpringOptions` | motion |
| `springValue` | motion |
| `StoreDef` | store |
| `StoreInstance` | store |
| `theme` | theme |
| `Theme` | theme |
| `ThemeApplyScopeOptions` | theme |
| `ThemeDoc` | theme |
| `ThemePatch` | theme |
| `ThemePersist` | theme |
| `ThemeScopePatch` | theme |
| `ThemeState` | theme |
| `ThemeTokenName` | theme |
| `timeline` | motion |
| `TimelineEasing` | motion |
| `TimelineEasingName` | motion |
| `TimelineOptions` | motion |
| `TimelineSignals` | motion |
| `TimelineStop` | motion |
| `TimelineStopOptions` | motion |
| `TimelineTrack` | motion |
| `TimelineTracks` | motion |
| `TimelineValue` | motion |
| `tipDescribe` | actions |
| `untrack` | reactivity |
| `unwrap` | data |
| `unwrapOr` | data |
| `useStore` | store |
| `virtualRow` | actions |
| `zoomPan` | actions |


## E. Channel vocabulary

Channels: **box · paint · text · animate**. Setters mapped: **164**.

| Channel | Setter count | Setters |
| --- | --- | --- |
| box | 47 | `aspect`, `basis`, `bottom`, `display`, `flexDir`, `gap`, `gapX`, `gapY`, `grow`, `h`, `inset`, `justifySelf`, `left`, `m`, `maxH`, `maxW`, `mb`, `me`, `minH`, `minW`, `ml`, `mr`, `ms`, `mt`, `mx`, `my`, `order`, `overflow`, `overflowX`, `overflowY`, `pad`, `padB`, `padE`, `padL`, `padR`, `padS`, `padT`, `padX`, `padY`, `position`, `resize`, `right`, `self`, `shrink`, `size`, `top`, `w` |
| paint | 99 | `accent`, `align`, `animate`, `appearance`, `avatarMedia`, `backdrop`, `bg`, `bgClip`, `bgGlow`, `bgImage`, `blur`, `border`, `borderB`, `borderBWidth`, `borderE`, `borderEWidth`, `borderImage`, `borderL`, `borderLWidth`, `borderR`, `borderRWidth`, `borderS`, `borderSWidth`, `borderT`, `borderTWidth`, `borderWidth`, `borderX`, `borderY`, `boxSizing`, `brightness`, `caret`, `clipPath`, `color`, `colorDeepen`, `colorMix`, `contain`, `container`, `containerName`, `containerType`, `content`, `contrast`, `controlFrame`, `controlLift`, `cursor`, `duration`, `ease`, `fg`, `fill`, `glyphFill`, `grayscale`, `group`, `hueRotate`, `invert`, `isolate`, `list`, `mask`, `maskImage`, `mixBlend`, `objectFit`, `objectPosition`, `opacity`, `origin`, `outline`, `outlineOffset`, `overscroll`, `perspective`, `pointerEvents`, `press`, `radius`, `radiusB`, `radiusL`, `radiusR`, `radiusT`, `ring`, `rotate`, `rotateX`, `rotateY`, `rotateZ`, `saturate`, `scale`, `scroll`, `scrollbarThin`, `scrollbarThumb`, `scrollbarTrack`, `select`, `sepia`, `shadow`, `srOnly`, `stroke`, `tapHighlight`, `textGradient`, `touch`, `transform`, `transition`, `translateX`, `translateY`, `visibility`, `willChange`, `z` |
| text | 18 | `antialiased`, `break`, `decoration`, `fontFamily`, `fontSize`, `fontWeight`, `indent`, `italic`, `leading`, `nums`, `tab`, `textAlign`, `textCase`, `tracking`, `truncate`, `underline`, `underlineOffset`, `whitespace` |
| animate | 0 |  |


### Animate triggers

`enter`, `exit`, `loop`, `reveal`, `scroll`, `hover`, `press`, `focus`, `focus-visible`

### Motion presets (keyframes catalog surface)

| Preset | Kind |
| --- | --- |
| `bounce` | tween |
| `fade` | tween |
| `fade-out` | tween |
| `marquee` | tween |
| `marquee-reverse` | tween |
| `press` | tween |
| `pulse` | tween |
| `release` | tween |
| `spin` | tween |
| `spring:bouncy` | spring |
| `spring:deliberate` | spring |
| `spring:gentle` | spring |
| `spring:snappy` | spring |


### `.when()` media conditions

| Key | Media |
| --- | --- |
| `mobile` | (min-width: 390px) |
| `tablet` | (min-width: 768px) |
| `desktop` | (min-width: 1280px) |
| `motion-reduce` | (prefers-reduced-motion: reduce) |
| `motion-safe` | (prefers-reduced-motion: no-preference) |
| `forced-colors` | (forced-colors: active) |
| `dark` | (prefers-color-scheme: dark) |
| `light` | (prefers-color-scheme: light) |


### Text-bearing hosts (text channel)

`p`, `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `span`, `a`, `label`, `code`, `summary`, `legend`, `button`, `input`, `textarea`, `option`, `pre`, `small`, `strong`, `em`, `b`, `i`, `li`, `th`, `td`, `dt`, `dd`, `figcaption`, `caption`, `cite`, `time`, `mark`, `abbr`, `sub`, `sup`, `blockquote`, `text`

---

Regenerate after any prop-registry / token-registry / action_manifest / author-surface / channel map change.
