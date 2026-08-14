/**
 * Authored sidecar — blurbs and category only.
 * Axes, props, concerns come from generated/catalog.ts.
 * Docs hub derives tiles from FAMILIES / hosts(); copy is optional.
 */

export type L1Category =
  | 'inputs'
  | 'display'
  | 'feedback'
  | 'overlay'
  | 'navigation'
  | 'data'
  | 'layout'
  | 'effects'

export type L1Copy = {
  category: L1Category
  blurb: string
}

/** Family-host keyed. Parts inherit the host's copy unless listed. */
export const L1_COPY: Record<string, L1Copy> = {
  a: {
    category: 'navigation',
    blurb:
      'Native <a>. Bare (no variant attr) stays a text link — freeform never stamps variant/tone. With variant set it becomes a button-link: default rest uses component.button.*; solid / soft / outline / ghost match the button family (default ≠ solid). tone always paints when set. No size axis — use <button> for sized CTAs. disabled paint needs aria-disabled on anchors.',
  },
  accordion: {
    category: 'display',
    blurb:
      'ONE disclosure element (replaces collapsible + details). open · disabled · name · multiple · onToggle. Shared name = exclusive multi-item (native details name). Paint component.accordion.* (bg · fg · border · hover-bg · chevron).',
  },
  alert: {
    category: 'feedback',
    blurb:
      'Inline status banner. Omit tone for component.alert tokens (themeable rest); set tone and the left stripe + icon brand paint. warning/destructive flip liveRegion to assertive role=alert; dismissible shows the close control (onDismiss then remove).',
  },
  avatar: {
    category: 'display',
    blurb:
      'Identity frame: image, icon-slug, or initials — broken-image falls back to initials. size xs–xl, shape round|square, optional tone. Faces component.avatar.bg / fg.',
  },
  badge: {
    category: 'display',
    blurb:
      'A small statement of state. Soft by default; solid or outline when it must shout or whisper. Omit tone to use component.badge tokens (themeable rest); set tone and the semantic colour always paints.',
  },
  breadcrumb: {
    category: 'navigation',
    blurb:
      'Trail host only — no host props. Compose nested <a> path crumbs and inert current (small). Faces component.breadcrumb.fg / current.',
  },
  button: {
    category: 'inputs',
    blurb: 'Buttons for what people do next — save, cancel, confirm, open.',
  },
  card: {
    category: 'display',
    blurb:
      'The surface everything else sits on. variant default|outline, padding, tone, interactive, accent-bar, status-rail, fill.',
  },
  checkbox: {
    category: 'inputs',
    blurb: 'A boolean check. Pair with a label. Use switch for on/off settings.',
  },
  chip: {
    category: 'display',
    blurb:
      'An interactive filter pill — the host is the control (role, keyboard, aria-pressed). Omit tone for component.chip rest tokens; set tone and the semantic colour always paints. removable adds the ×; Backspace/Delete fires on-remove.',
  },
  'code-block': {
    category: 'display',
    blurb:
      'Syntax-highlighted source block. Host is a div with use:syntaxHighlight; paint on the inner pre from component.code-block (bg · foreground) plus global border/radius. Pass code as a string prop (not children). lang defaults to sail and is an authoring attr — the highlighter classifies by content shape (markup vs code), not by inventing line numbers or a copy chrome.',
  },
  color: {
    category: 'inputs',
    blurb: 'A system colour swatch for a single hex value.',
  },
  combobox: {
    category: 'inputs',
    blurb: 'Type to filter a list. Arrows move, Enter commits, Escape closes.',
  },
  'context-menu': {
    category: 'overlay',
    blurb:
      'Dropdown\'s right-click sibling: menu semantics, opened at the pointer on contextmenu of the target that for= names. onSelect event. Paint component.context-menu.*.',
  },
  date: {
    category: 'inputs',
    blurb: 'Date and time fields. type picks precision.',
  },
  dialog: {
    category: 'overlay',
    blurb:
      'Platform dialog — top layer, focus trap, Escape, backdrop. size · backdrop · side · dismissible; onClose / onDismiss. Scaffold shows open anatomy; invoker open/close is native.',
  },
  dropdown: {
    category: 'overlay',
    blurb:
      'Popover with menu semantics: role=menu, roving arrows, type-ahead, <menu-item> children. placement string; onSelect event. Paint component.dropdown.*.',
  },
  'empty-state': {
    category: 'feedback',
    blurb:
      'Nothing-here surface: icon + title + body + action slots. Chrome via variant (bordered default, dashed inline, bare placeholder/connecting). Omit tone for component.empty-state.* (themeable rest); set tone to brand the icon only. connecting upgrades to role=status / aria-live=polite via use:variantLive.',
  },
  'error-boundary': {
    category: 'feedback',
    blurb:
      'Render-fault isolation. A throw inside the subtree swaps in the fallback (built-in or slot="fallback"); Retry — or el.reset() — brings the content back.',
  },
  'error-state': {
    category: 'feedback',
    blurb:
      'Recovery surface for failed loads and partial failures. Rest chrome paints component.error-state (bg · fg · border); icon + inline border keep brand destructive. code fills a mono chip; details slot gates Show details; action slot holds Retry. use:recoverySurface stamps role=alert, syncs code, and focuses the first action.',
  },
  field: {
    category: 'inputs',
    blurb: 'Label plus control. Wires spacing and invalid state so the message and border agree.',
  },
  fieldset: {
    category: 'inputs',
    blurb: 'Groups related controls under a legend. Disabled freezes everything inside.',
  },
  file: {
    category: 'inputs',
    blurb: 'System file picker. variant="drop" is the drag-and-drop surface.',
  },
  'hover-card': {
    category: 'overlay',
    blurb:
      'Rich panel on hover/focus of the trigger whose popovertarget names it. placement string. Paint component.hover-card.*.',
  },
  icon: {
    category: 'display',
    blurb:
      'Glyph box on the icon-size ladder. Prefer name for pack ids (local /_deck/icons resolve — never CDN); src only same-origin SVG. Empty label is decorative (aria-hidden); set label for role=img. Ink is currentColor + glyphFill (no tone prop). spin is owned by use:icon motion.',
  },
  input: {
    category: 'inputs',
    blurb: 'Single-line text fields. Passwords show a reveal control by default.',
  },
  kbd: {
    category: 'display',
    blurb:
      'Native <kbd> keycap — mono type, muted surface, doubled bottom edge. Paint from component.kbd (bg · foreground · border); no size/variant/tone props (theme the leaves, compose chords in markup).',
  },
  label: {
    category: 'inputs',
    blurb:
      'Native <label> — for= focuses the control. size sm|md|lg matches the control type ladder. required shows the * marker from component.label.required; host ink from component.label.fg and weight from typography.label.weight. No tone prop.',
  },
  meter: {
    category: 'feedback',
    blurb:
      'Native <meter> gauge — a measurement in range, not task progress. value/min/max set the reading; low/high/optimum drive platform fill bands painted from component.meter.track / optimum / suboptimum / sub-suboptimum. size scales bar height. No tone prop.',
  },
  navbar: {
    category: 'navigation',
    blurb:
      'One element, three placements — top bar, side rail, bottom bar. sticky · bordered · autohide · offset. Regions: brand, center/links, actions. Paint from component.navbar.*.',
  },
  pagination: {
    category: 'navigation',
    blurb:
      'A nav of <page> controls. value marks the current one with aria-current; clicking emits sail-page-change — your list, your fetch. Size is authored on each <page>.',
  },
  popover: {
    category: 'overlay',
    blurb:
      'Native [popover] positioned by placement. Escape and light-dismiss from the platform; use:anchor places the panel. Paint from component.popover.*.',
  },
  progress: {
    category: 'feedback',
    blurb:
      'Native progress bar, themed. value/max are the platform model; omit value for the indeterminate sweep (no separate prop). size is sm|md|lg track height. Paint always from component.progress.track + fill — no tone axis.',
  },
  prose: {
    category: 'display',
    blurb:
      'Typography scope for rendered content — wrap markdown blobs and long copy in <prose>. Host ink is component.prose.fg; nested h1…p, lists, blockquote, and code pick up the framework type map. No size/variant/tone props (theme the leaf + type roles, not a Title API).',
  },
  radio: {
    category: 'inputs',
    blurb: 'One option in a group. Prefer radio-group for exclusive choice.',
  },
  'radio-group': {
    category: 'inputs',
    blurb: 'A set of radios with one shared value and accessible name.',
  },
  'scroll-area': {
    category: 'layout',
    blurb:
      'Overflow box with thin, themed scrollbars. Only orientation is a host prop (vertical · horizontal · both; default vertical). Constrain height/width with Style box so content overflows. Rest thumb from component.scroll-area.thumb; hover uses free border-strong (SPEC-D1 — thumb-hover leaf is parked unread). Standard scrollbar-width/color only (no WebKit pseudo setters yet).',
  },
  select: {
    category: 'inputs',
    blurb: 'Closed list picker. Keyboard, form submit, and mobile sheets come from the platform.',
  },
  separator: {
    category: 'display',
    blurb:
      'Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. Omit tone for themeable component.separator.line; set muted · primary · destructive only when the line must speak. No separate `hr` catalog entry.',
  },
  skeleton: {
    category: 'feedback',
    blurb:
      'Loading placeholder sheen. Shape + size draw the geometry; paint is always the shimmer from component.skeleton.bg / highlight (themeable). No tone axis — compose width with layout Style.',
  },
  slider: {
    category: 'inputs',
    blurb: 'A single-thumb range control. Set min, max, and step as needed.',
  },
  spinner: {
    category: 'feedback',
    blurb:
      'Indeterminate wait ring. Size scales the host; omit tone for themeable spinner.foreground / spinner.track; centered parks it in the middle of its container.',
  },
  switch: {
    category: 'inputs',
    blurb: 'On / off toggle for settings. Use checkbox for multi-select lists.',
  },
  table: {
    category: 'data',
    blurb:
      'The platform\'s table model — thead, tbody, tr — skinned by the theme. dense · striped · hoverable · bordered presence attrs. Sticky thead and numeric columns are structural; sorting/selection are L2.',
  },
  tabs: {
    category: 'navigation',
    blurb:
      'Tabs with the keyboard suite. Host props: active · orientation · onTabChange. Strip look on <tab-list>; size on each <tab>.',
  },
  textarea: {
    category: 'inputs',
    blurb: 'Multi-line text. Same field look as input and select.',
  },
  'theme-picker': {
    category: 'inputs',
    blurb:
      'Theme switcher: trigger opens a native [popover] of theme cards previewing real stylesheets. Host props: value · inline · modeOnly · onThemeChange. Paint component.theme-picker.* (panel · trigger · option).',
  },
  ticker: {
    category: 'effects',
    blurb:
      'Infinite linear marquee. Host is geometry only (overflow clip); use:marquee dual-copies the slot and loops translate-x. duration · reverse on host. No component.ticker.* — author paint. Reduced motion settles via motion runtime.',
  },
  toast: {
    category: 'feedback',
    blurb:
      'Transient announcement. Auto-dismiss after duration ms (0 sticky), pauses while hovered/focused. Optional tone; onDismiss. Lives in a toast-region stack. Faces component.toast.*.',
  },
  'toggle-group': {
    category: 'inputs',
    blurb:
      'A row of toggles with one value (or many, with type=multiple). Roving focus, aria-pressed and the segmented look are all in the element. Size is authored on each <toggle>. Segmented control host — no standalone toggle hub tile.',
  },
  tooltip: {
    category: 'overlay',
    blurb:
      'Label on hover and focus from the content attribute. placement top|bottom|left|right. Bubble paint component.tooltip.bg / fg.',
  },
}
