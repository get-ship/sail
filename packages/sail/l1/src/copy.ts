/**
 * Authored sidecar — blurbs and category only.
 * Axes, props, concerns come from generated/catalog.ts.
 * Docs hub derives tiles from COMPONENTS / hosts(); copy is optional.
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

/** Component-host keyed. Parts inherit the host's copy unless listed. */
export const L1_COPY: Record<string, L1Copy> = {
  a: {
    category: 'navigation',
    blurb:
      'Anchor — native <a>. Bare or variant=text has no chrome: wrap a card, span, or label. variant default|solid|soft|outline|ghost is a button-link. Freeform never stamps variant/tone so a plain href stays a text link. No size axis — use <button> for sized CTAs.',
  },
  accordion: {
    category: 'display',
    blurb:
      'ONE disclosure element (replaces collapsible + details). open · disabled · name · multiple · onToggle. Shared name = exclusive multi-item (native details name). Paint component.accordion.* (bg · fg · border · hover-bg · chevron).',
  },
  alert: {
    category: 'feedback',
    blurb:
      'Inline status banner. tone defaults to info and paints the left stripe + icon. warning/danger flip liveRegion to assertive role=alert; dismissible shows the close control (onDismiss then remove).',
  },
  avatar: {
    category: 'display',
    blurb:
      'Identity frame: image, icon-slug, or initials — broken-image falls back to initials. size xs–xl, shape round|square, tone defaults to muted. Faces component.avatar.bg / fg.',
  },
  badge: {
    category: 'display',
    blurb:
      'A small statement of state. Soft by default; solid or outline when it must shout or whisper. tone defaults to muted.',
  },
  breadcrumb: {
    category: 'navigation',
    blurb: 'Where you are. Earlier steps are links.',
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
      'An interactive filter pill — the host is the control (role, keyboard, aria-pressed). tone defaults to primary. removable adds the ×; Backspace/Delete fires on-remove.',
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
  drawer: {
    category: 'overlay',
    blurb:
      'Edge-anchored non-modal sheet. side start|end|top|bottom (default end); size sm|md|lg. Native <dialog> via command="show" (not showModal). Header/section/footer as light-dom children. Swipe toward the edge closes. Paint card / card-foreground.',
  },
  dropdown: {
    category: 'overlay',
    blurb:
      'Popover with menu semantics: role=menu, roving arrows, type-ahead, <menu-item> children. placement string; onSelect event. Paint component.dropdown.*.',
  },
  'empty-state': {
    category: 'feedback',
    blurb:
      'Nothing-here surface: icon + title + body + action slots. Chrome via variant (bordered default, dashed inline, bare placeholder/connecting). tone defaults to muted and paints the icon slot. connecting upgrades to role=status / aria-live=polite via use:variantLive.',
  },
  'error-boundary': {
    category: 'feedback',
    blurb:
      'Render-fault isolation. A throw inside the subtree swaps in the fallback (built-in or slot="fallback"); Retry — or el.reset() — brings the content back.',
  },
  'error-state': {
    category: 'feedback',
    blurb:
      'Recovery surface for failed loads and partial failures. Rest chrome paints component.error-state (bg · fg · border); icon + inline border keep brand danger. code fills a mono chip; details slot gates Show details; action slot holds Retry. use:recoverySurface stamps role=alert, syncs code, and focuses the first action.',
  },
  field: {
    category: 'inputs',
    blurb:
      'Label plus control — stack+gap; hint/error gates via structure. fill (default true) makes the field a group host so the slotted control stretches; a naked input still hugs. fill=false keeps the compact search-in-a-row case. invalid wires the message and border.',
  },
  fieldset: {
    category: 'inputs',
    blurb: 'Groups related controls under a legend. Disabled freezes everything inside.',
  },
  file: {
    category: 'inputs',
    blurb: 'System file picker. variant="drop" is the drag-and-drop surface.',
  },
  form: {
    category: 'inputs',
    blurb:
      'Native form. The tag is the host; children stay the author\'s. Column integrity is the author\'s <column>; fields stay L1 <field>. layout=measure (default) is the reading column; layout=fill takes leftover width, no center. Not a pane.',
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
  page: {
    category: 'layout',
    blurb:
      'The document — inset, not a measure column. Not chrome. Not a max-width box. Children stack. Theme inset so the document never kisses the glass. Width is the viewport; measure belongs to the pieces (prose, form, a card). Layouts name this host. A rail opts out.',
  },
  pagination: {
    category: 'navigation',
    blurb:
      'A nav of <page-item> controls. value marks the current one with aria-current; clicking emits sail-page-change — your list, your fetch. Size is authored on each <page-item>.',
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
  resizable: {
    category: 'layout',
    blurb:
      'Two-pane split: start and end slots with a drag handle between. orientation horizontal (default) or vertical. Drag — or arrow-nudge — the separator to size the start pane. Handle is a focusable role=separator.',
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
      'Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. tone defaults to muted. No separate `hr` catalog entry.',
  },
  skeleton: {
    category: 'feedback',
    blurb:
      'Loading placeholder sheen. Shape + size draw the geometry; the shimmer mixes muted with muted-foreground so it reads on light and dark. No tone axis — compose width with layout Style.',
  },
  slider: {
    category: 'inputs',
    blurb: 'A single-thumb range control. Set min, max, and step as needed.',
  },
  spinner: {
    category: 'feedback',
    blurb:
      'Indeterminate wait ring. Size scales the glyph; tone defaults to info. Center it with the parent layout.',
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
      'Tabs with the keyboard suite. Write <tab title> plus the panel it owns. Host props: active · orientation · onTabChange. Vertical is a two-column grid.',
  },
  textarea: {
    category: 'inputs',
    blurb: 'Multi-line text. Same field look as input and select.',
  },
  'theme-picker': {
    category: 'inputs',
    blurb:
      'A control for picking a theme. Click to browse live previews. chrome=inline keeps the list on the page. mode is one icon that cycles Light / Dark / System.',
  },
  ticker: {
    category: 'effects',
    blurb:
      'A looping band of text or icons. Drop content in — the host clips, nowraps, and paints a quiet strip. duration and reverse set the motion. Reduced motion settles automatically.',
  },
  toast: {
    category: 'feedback',
    blurb:
      'Transient announcement. Auto-dismiss after duration ms (0 sticky), pauses while hovered/focused. tone defaults to info; onDismiss. Lives in a toast-region stack. Faces component.toast.*.',
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
