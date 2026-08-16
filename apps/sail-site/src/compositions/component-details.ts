// /components/[tag] — detail data for the FULL L1 surface. slug → { heading,
// blurb, variants, props, usage }. Props tables are GENERATED from the
// compiler's prop registry (see genProps below); regen with
// `cargo run -p sail-compiler -- prop-registry` after touching a stdlib
// <scope> props declaration. The usage strings live in
// ./snippets.ts so the site-wide snippet guard compiles every one
// of them.
import { PROP_REGISTRY } from './prop-registry'
import { usage } from './component-detail-snippets'

export interface PropRow {
  name: string
  desc: string
  type: string
}

export interface DetailEntry {
  /** The element tag — breadcrumb + copy. */
  tag: string
  /** The scope key page.sail mounts the live stage through (`stage.<key>()`). */
  key: string
  /** The h1, already formed: '<button>', 'row · column · grid'. */
  heading: string
  pack: 'lang'
  blurb: string
  /** What the variant chips switch on the live stage. Empty = no chips. */
  variants: string[]
  /** Which attribute the chips drive — shown beside the chips. */
  variantAttr: string
  props: PropRow[]
  usage: string
}

const TONES = 'primary · info · success · warning · destructive · accent · muted'

// The props table is GENERATED: names, types and defaults come from the
// compiler's prop registry (the same <scope> props declarations the
// build validates attributes against), so this page can never drift from the
// stdlib. The curated rows below serve two narrower jobs:
//   * a hand description for a generated prop the stdlib hasn't documented
//     yet (matched by name; the stdlib JSDoc wins once it exists),
//   * language-surface rows the registry can't know — bind:/on:/fx:
//     directives, children contracts, announced events — appended after.
const genProps = (tag: string, curated: PropRow[]): PropRow[] => {
  const gen = PROP_REGISTRY[tag] ?? []
  const curatedByName = new Map(curated.map((r) => [r.name, r]))
  const rows: PropRow[] = gen.map((g) => ({
    name: g.name,
    type: g.type,
    desc: g.desc ?? curatedByName.get(g.name)?.desc ?? (g.default ? `default: ${g.default}` : ''),
  }))
  const genNames = new Set(gen.map((g) => g.name))
  for (const r of curated) {
    if (!genNames.has(r.name)) rows.push(r)
  }
  return rows
}

const lang = (
  tag: string,
  key: string,
  rest: Omit<DetailEntry, 'tag' | 'key' | 'heading' | 'pack' | 'usage'> & { heading?: string },
): DetailEntry => ({
  tag,
  key,
  heading: rest.heading ?? `<${tag}>`,
  pack: 'lang',
  blurb: rest.blurb,
  variants: rest.variants,
  variantAttr: rest.variantAttr,
  props: genProps(tag, rest.props),
  usage: usage[tag] ?? '',
})

const DETAILS_MAP = {
  // ── Inputs ───────────────────────────────────────────────────────────
  button: lang('button', 'button', {
    blurb: 'Buttons for what people do next — save, cancel, confirm, open.',
    variants: ['solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      {
        name: 'variant',
        desc: 'How loud the fill is',
        type: 'solid · soft · outline · ghost',
      },
      {
        name: 'tone',
        desc: 'Colour (default primary)',
        type: TONES,
      },
      {
        name: 'size',
        desc: 'Height, or square for icon-only',
        type: 'xs · sm · md · lg · icon · iconSm · iconLg',
      },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
      {
        name: 'label',
        desc: 'Accessible name — required on icon-only (lowers to aria-label)',
        type: 'string',
      },
    ],
  }),
  a: lang('a', 'a', {
    heading: 'anchor',
    blurb:
      'Anchor — native <a>. Bare or variant=text wraps anything as a link. Button chrome is default|solid|soft|outline|ghost.',
    variants: ['text', 'default', 'solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      {
        name: 'variant',
        desc: 'text (or omit) = no chrome, wrap a card/span. Button family: default · solid · soft · outline · ghost',
        type: 'text · default · solid · soft · outline · ghost',
      },
      {
        name: 'tone',
        desc: 'Colour when variant is set (default primary)',
        type: TONES,
      },
      {
        name: 'disabled',
        desc: 'Non-interactive (use aria-disabled on anchors)',
        type: 'bool',
      },
      { name: 'href', desc: 'Navigation target', type: 'string' },
      { name: 'children', desc: 'Link label', type: 'slot' },
    ],
  }),
  input: lang('input', 'input', {
    blurb:
      'Single-line text fields — text, email, password, search, number, tel, url, and hidden. Passwords show a reveal control by default.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'type',
        desc: 'Text field mode',
        type: 'text · email · password · number · search · tel · url · hidden',
      },
      {
        name: 'size',
        desc: 'Control height (default md)',
        type: 'xs · sm · md · lg',
      },
      {
        name: 'invalid',
        desc: 'Validation failed',
        type: 'bool',
      },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
      { name: 'placeholder', desc: 'Hint when empty', type: 'string' },
      { name: 'reveal', desc: 'Password show/hide (default true)', type: 'bool' },
      { name: 'name · required · readonly · value', desc: 'Form participation', type: 'string · bool' },
      { name: 'min · max · step', desc: 'Number bounds', type: 'string' },
      { name: 'autocomplete · pattern · maxlength · inputmode', desc: 'Text constraints', type: 'string' },
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
    ],
  }),
  textarea: lang('textarea', 'textarea', {
    blurb: 'Multi-line text. Same field surface as input and select.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
      { name: 'size', desc: 'Control scale (default md)', type: 'xs · sm · md · lg' },
      { name: 'rows', desc: 'Initial height in lines (default 3)', type: 'number' },
      { name: 'placeholder', desc: 'Hint when empty', type: 'string' },
      { name: 'invalid', desc: 'Validation failed', type: 'bool' },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
    ],
  }),
  select: lang('select', 'select', {
    blurb: 'Closed list picker. Keyboard, form submit, and mobile sheets come from the platform.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
      {
        name: 'size',
        desc: 'Control height (default md)',
        type: 'xs · sm · md · lg',
      },
      {
        name: 'invalid',
        desc: 'Validation failed',
        type: 'bool',
      },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
      {
        name: 'multiple',
        desc: 'Allow multiple selections',
        type: 'bool',
      },
      { name: '<option> / <optgroup>', desc: 'Options', type: 'children' },
    ],
  }),
  combobox: lang('combobox', 'combobox', {
    blurb: 'Type to filter a list. Arrow keys move, Enter commits, Escape closes.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placeholder', desc: 'Hint when empty', type: 'string' },
      { name: 'value', desc: 'Committed value', type: 'string' },
      { name: 'onSelect', desc: 'Commit callback — { value, label }', type: 'callback' },
      { name: '<option>', desc: 'List options', type: 'children' },
    ],
  }),
  checkbox: lang('checkbox', 'checkbox', {
    blurb: 'A boolean check. Pair with a label. Use switch for on/off settings.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:checked', desc: 'Two-way binding', type: 'signal' },
      { name: 'size', desc: 'Control size (default md)', type: 'xs · sm · md · lg' },
      { name: 'invalid', desc: 'Validation failed', type: 'bool' },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
      { name: 'name · required · value', desc: 'Form participation', type: 'string · bool' },
    ],
  }),
  date: lang('date', 'date', {
    blurb: 'Date and time fields. type picks precision; the platform supplies the calendar or clock UI.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'type',
        desc: 'Precision (default date)',
        type: 'date · time · datetime-local · month · week',
      },
      { name: 'min · max · step', desc: 'Bounds', type: 'string' },
      { name: 'size', desc: 'Control height (default md)', type: 'xs · sm · md · lg' },
      { name: 'invalid · disabled · name · required · readonly · value', desc: 'State + form', type: 'mixed' },
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
    ],
  }),
  file: lang('file', 'file', {
    blurb:
      'File drop surface. Native Browse chrome is hidden. variant="drop" (default) is the dashed well; variant="picker" is the same skin, compact.',
    variants: ['picker', 'drop'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'picker is the system control; drop is the drag target', type: 'picker · drop' },
      { name: 'accept', desc: 'MIME / extension filter', type: 'string' },
      { name: 'multiple', desc: 'Allow multiple files', type: 'bool' },
      { name: 'size', desc: 'Picker height (default md)', type: 'xs · sm · md · lg' },
      { name: 'invalid · disabled · name · required', desc: 'State + form', type: 'mixed' },
    ],
  }),
  color: lang('color', 'color', {
    blurb: 'A system colour swatch for a single hex value.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'Hex color', type: 'string' },
      { name: 'name', desc: 'Form name', type: 'string' },
      { name: 'invalid · disabled', desc: 'State', type: 'bool' },
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
    ],
  }),
  'radio-group': lang('radio-group', 'radioGroup', {
    blurb:
      'A set of radios with one shared value and accessible name. <radio> is the option part — not a hub tile.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'Selected value', type: 'string' },
      { name: 'label', desc: 'Group name', type: 'string' },
      { name: 'name', desc: 'Form name for the radios', type: 'string' },
      { name: 'disabled / required', desc: 'Group state', type: 'bool' },
      { name: '<radio value>', desc: 'Each option — value + label children; disabled per cell', type: 'children' },
    ],
  }),
  switch: lang('switch', 'switchToggle', {
    blurb: 'On / off toggle for settings. Use checkbox for multi-select lists.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:checked', desc: 'Two-way binding', type: 'signal' },
      { name: 'checked', desc: 'On/off state', type: 'bool' },
      {
        name: 'tone',
        desc: 'Colour when on (default primary)',
        type: TONES,
      },
      { name: 'size', desc: 'Track + thumb (28 · 36 · 48 wide)', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
    ],
  }),
  slider: lang('slider', 'slider', {
    blurb: 'A single-thumb range control. Set min, max, and step as needed. ticks labels sit on thumb centers.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'Two-way binding', type: 'signal' },
      { name: 'value', desc: 'Current position (default 0)', type: 'number' },
      { name: 'min', desc: 'Minimum (default 0)', type: 'number' },
      { name: 'max', desc: 'Maximum (default 100)', type: 'number' },
      { name: 'step', desc: 'Increment (default 1)', type: 'number' },
      { name: 'size', desc: 'Track size (default md)', type: 'sm · md · lg' },
      { name: 'ticks', desc: 'Comma-separated labels under each thumb stop', type: 'string' },
      { name: 'disabled', desc: 'Non-interactive', type: 'bool' },
      { name: 'name', desc: 'Form name', type: 'string' },
    ],
  }),
  'toggle-group': lang('toggle-group', 'toggleGroup', {
    blurb:
      'A row of toggles with one value (or many, with type=multiple). Roving focus, aria-pressed and the segmented look are all in the element. Size is authored on each <toggle>.',
    variants: ['outline', 'ghost', 'segmented'],
    variantAttr: 'variant',
    props: [
      { name: 'type', desc: 'one value or many', type: 'single · multiple' },
      { name: 'value', desc: 'the selected value(s), reflected', type: 'string' },
      { name: 'variant', desc: 'the look of the group', type: 'outline · ghost · segmented' },
      { name: 'orientation', desc: 'stack it vertically', type: 'horizontal · vertical' },
      { name: '<toggle value size>', desc: 'the options — size on each toggle', type: 'children' },
    ],
  }),
  field: lang('field', 'field', {
    blurb:
      'Label, control, and hint or error in one unit. invalid swaps the hint for the error message.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'invalid', desc: 'Show error instead of hint', type: 'bool' },
      { name: 'slots', desc: 'label · default control · hint · error', type: 'slots' },
    ],
  }),
  label: lang('label', 'label', {
    blurb: 'Labels a control. for= focuses the matching field. required shows a marker.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'for', desc: 'Click focuses the control', type: 'id' },
      { name: 'size', desc: 'Type scale matching the control (default md)', type: 'sm · md · lg' },
      { name: 'required', desc: 'Shows a * marker', type: 'bool' },
      { name: '(children)', desc: 'Label text', type: 'children' },
    ],
  }),
  fieldset: lang('fieldset', 'fieldset', {
    blurb:
      'Groups related controls under a legend. disabled freezes every control inside and dims the group.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'disabled', desc: 'Freezes controls inside and dims the group', type: 'bool' },
      { name: '<legend>', desc: 'Group caption (first child)', type: 'children' },
      { name: 'default', desc: 'Controls and copy inside the group', type: 'slot' },
    ],
  }),
  // legend is NOT a top-level DETAILS/catalog topic — documented inside fieldset.
  'theme-picker': lang('theme-picker', 'themePicker', {
    blurb:
      'Pick a theme. Click the control to browse live previews. value selects a theme; mode is one icon for Light / Dark / System; chrome=inline keeps the list on the page.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'Selected theme id (applies that theme)', type: 'string' },
      { name: 'chrome', desc: 'popover (default) or inline in-flow catalog', type: 'popover · inline' },
      { name: 'size', desc: 'full specimen cards or compact density', type: 'full · compact' },
      { name: 'orientation', desc: 'inline wrap: horizontal (default) or vertical', type: 'horizontal · vertical' },
      { name: 'mode', desc: 'One icon button that cycles Light / Dark / System', type: 'bool' },
      { name: 'inline', desc: 'Alias of chrome=inline', type: 'bool' },
      { name: 'mode-only', desc: 'Alias of mode', type: 'bool' },
      { name: 'onThemeChange', desc: 'Fires with { theme, mode, preference }', type: 'callback' },
    ],
  }),

  // ── Feedback ─────────────────────────────────────────────────────────
  alert: lang('alert', 'alert', {
    blurb:
      'Inline status banner. tone defaults to info and colours the left stripe and icon. warning/destructive announce assertively (role=alert); dismissible shows the close control (onDismiss then remove).',
    // No variant axis — stage owns tone/dismiss knobs (viewer chips hidden).
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'tone',
        desc: 'stripe + icon colour (default info)',
        type: 'muted · info · success · warning · destructive',
      },
      { name: 'dismissible', desc: 'shows the close control ([data-dismiss])', type: 'bool' },
      { name: 'onDismiss', desc: 'fires before the host removes itself', type: 'callback' },
      { name: 'slot="title"', desc: 'optional title line', type: 'slot' },
      { name: 'slot="icon"', desc: 'optional leading icon', type: 'slot' },
      { name: 'slot="actions"', desc: 'optional action row', type: 'slot' },
    ],
  }),
  progress: lang('progress', 'progressBar', {
    blurb:
      'Native progress bar, themed. value/max are the platform model; omit value for the indeterminate sweep (no separate prop). size is sm|md|lg track height. Track and fill always come from the theme — no tone axis.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'value',
        desc: 'current amount; omit the attribute for native indeterminate shimmer',
        type: 'number',
      },
      { name: 'max', desc: 'upper bound of the range (default 1 on the host)', type: 'number' },
      {
        name: 'size',
        desc: 'track height ladder (default md)',
        type: 'sm · md · lg',
      },
    ],
  }),
  meter: lang('meter', 'meter', {
    blurb:
      'Native <meter> gauge — a measurement in range, not task progress. value/min/max set the reading; low/high/optimum drive platform fill bands (optimum · suboptimum · even less good) from the theme. size scales bar height. No tone prop.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value / min / max', desc: 'the measurement (platform meter model)', type: 'number' },
      {
        name: 'low / high / optimum',
        desc: 'thresholds that pick optimum · suboptimum · even-less-good fill bands',
        type: 'number',
      },
      { name: 'size', desc: 'bar height', type: 'sm · md · lg' },
    ],
  }),
  spinner: lang('spinner', 'spinner', {
    blurb:
      'A compact wait mark. Default is an icon-sized ring. variant=dots or bars for the same wait, different motion. Size is the icon ladder. Center it with the parent — the mark does not own the page.',
    variants: ['ring', 'dots', 'bars'],
    variantAttr: 'variant',
    props: [
      { name: 'size', desc: 'icon ladder (xs · sm · md · lg)', type: 'xs · sm · md · lg' },
      { name: 'variant', desc: 'ring · dots · bars', type: 'ring · dots · bars' },
      {
        name: 'tone',
        desc: 'active arc colour (default info); track always uses the theme track colour',
        type: 'primary · info · success · warning · destructive · accent',
      },
    ],
  }),
  skeleton: lang('skeleton', 'skeleton', {
    blurb:
      'Loading placeholder sheen. Shape + size draw the geometry; the shimmer is muted mixed with muted-foreground so it reads on light and dark surfaces. No tone axis — compose width with layout Style. No skeleton prop on other hosts: compose circle + lines.',
    variants: ['text', 'line', 'block', 'circle'],
    variantAttr: 'shape',
    props: [
      {
        name: 'shape',
        desc: 'geometry: text/line bars, block tile, or circle avatar stand-in (default text)',
        type: 'text · line · block · circle',
      },
      {
        name: 'size',
        desc: 'text/line bar height and circle diameter; block ignores size (fixed min height)',
        type: 'sm · md · lg',
      },
    ],
  }),
  'empty-state': lang('empty-state', 'emptyState', {
    blurb:
      'Nothing-here surface: icon + title + body + action slots. Chrome via variant (bordered default, dashed inline, bare placeholder/connecting). tone defaults to muted and paints the icon slot. connecting upgrades to role=status / aria-live=polite.',
    // Stage owns variant/size/tone knobs — hide generic viewer chips.
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'variant',
        desc: 'chrome: default card, inline dashed, bare placeholder, or connecting live status',
        type: 'default · inline · placeholder · connecting',
      },
      { name: 'size', desc: 'padding scale (sm · md · lg)', type: 'sm · md · lg' },
      {
        name: 'tone',
        desc: 'icon colour (default muted)',
        type: TONES,
      },
      { name: 'slot="icon"', desc: 'optional leading glyph (hidden when empty)', type: 'slot' },
      { name: 'slot="title"', desc: 'optional headline (hidden when empty)', type: 'slot' },
      { name: 'slot="action"', desc: 'optional action row (hidden when empty)', type: 'slot' },
      { name: 'children', desc: 'description body (hidden when empty)', type: 'slot' },
    ],
  }),
  'error-state': lang('error-state', 'errorState', {
    blurb:
      'Recovery surface for failed loads and partial failures. Default chrome uses theme colors; icon + inline border stay destructive. code fills a mono chip; details slot gates Show details; action slot holds Retry. Marks role=alert, syncs code, and focuses the first action.',
    variants: ['default', 'inline'],
    variantAttr: 'variant',
    props: [
      {
        name: 'variant',
        desc: 'default = centered card with theme colors; inline = start-aligned row with destructive border + muted fill',
        type: 'default · inline',
      },
      { name: 'size', desc: 'padding scale (default md)', type: 'sm · md · lg' },
      {
        name: 'code',
        desc: 'machine-readable line for the code chip; omit or empty hides the chip',
        type: 'string',
      },
      { name: 'slot="title"', desc: 'headline (default: Something went wrong)', type: 'slot' },
      { name: 'children', desc: 'description body', type: 'slot' },
      { name: 'slot="details"', desc: 'stack / diagnostic block — reveals Show details when non-empty', type: 'slot' },
      { name: 'slot="action"', desc: 'recovery controls (Retry, Go back); first control is auto-focused', type: 'slot' },
    ],
  }),
  'error-boundary': lang('error-boundary', 'errorBoundary', {
    blurb:
      'Render-fault isolation. A throw inside the subtree swaps in the fallback (built-in or slot="fallback"); Retry — or el.reset() — brings the content back.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'message', desc: 'what the user reads — not the raw error', type: 'string' },
      { name: 'slot="fallback"', desc: 'your own fallback markup', type: 'slot' },
      { name: 'sail-error-caught', desc: 'the real error, for telemetry', type: 'callback' },
    ],
  }),
  toast: lang('toast', 'toast', {
    blurb:
      'A transient announcement. It auto-dismisses after duration ms (0 = sticky), pauses while hovered or focused, and lives in a <toast-region> stack. Dismiss removes the instance — re-push from your controller to show again.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'tone',
        desc: 'semantic colour (default info)',
        type: TONES,
      },
      { name: 'duration', desc: 'ms until dismiss as string — 0 is sticky', type: 'string' },
      { name: 'onDismiss', desc: 'fires before remove — drop the instance from your list', type: 'callback' },
      { name: 'data-toast-close', desc: 'marks a custom close control (L1 ships one)', type: 'attr' },
      { name: '<toast-region position>', desc: 'the fixed stack it lives in', type: 'top/bottom-start/end' },
    ],
  }),

  // ── Display ──────────────────────────────────────────────────────────
  badge: lang('badge', 'badge', {
    blurb:
      'A small statement of state. Default tone is muted. Soft by default; solid or outline when it must shout or whisper.',
    variants: ['soft', 'solid', 'outline'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'fill strength — soft default', type: 'soft · solid · outline' },
      {
        name: 'tone',
        desc: 'semantic colour (default muted)',
        type: TONES,
      },
      { name: 'size', desc: 'badge scale for table cells and chrome', type: 'sm · md · lg' },
      { name: 'pill', desc: 'fully rounded — counters and filters', type: 'bool' },
      { name: 'children', desc: 'label text (keep short)', type: 'slot' },
    ],
  }),
  chip: lang('chip', 'chip', {
    blurb:
      'An interactive filter pill — the host is the control (role, keyboard, aria-pressed). tone defaults to primary. removable adds the ×; Backspace/Delete fires on-remove.',
    variants: ['solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'fill strength — solid default', type: 'solid · soft · outline · ghost' },
      {
        name: 'tone',
        desc: 'semantic colour (default primary)',
        type: TONES,
      },
      { name: 'size', desc: 'chip scale for filters and dense chrome', type: 'sm · md · lg' },
      { name: 'selected', desc: 'pressed state, ARIA-synced', type: 'bool' },
      { name: 'removable', desc: 'the × button + Backspace/Delete', type: 'bool' },
      { name: 'disabled', desc: 'non-interactive; muted default colours', type: 'bool' },
      { name: 'on-remove', desc: 'remove callback (× and keyboard)', type: 'callback' },
      { name: 'leading', desc: 'optional leading icon/slot', type: 'slot' },
      { name: 'children', desc: 'label text (keep short)', type: 'slot' },
    ],
  }),
  card: lang('card', 'card', {
    blurb:
      'The surface everything else sits on. variant default|outline, padding, tone (default muted), interactive, accent-bar, status-rail, fill — no soft/elevated/ghost invent.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'tone', desc: 'border tint (default muted)', type: TONES },
      { name: 'variant', desc: 'filled or transparent — default · outline only', type: 'default · outline' },
      { name: 'padding', desc: 'inset scale', type: 'none · sm · md · lg' },
      { name: 'interactive', desc: 'pointer cursor + CSS hover lift', type: 'bool' },
      { name: 'accent-bar', desc: 'tone-coloured top bar', type: 'off · hover · static' },
      { name: 'status-rail', desc: 'leading status stripe', type: 'bool' },
      { name: 'fill', desc: 'muted fill override', type: 'bool' },
    ],
  }),
  avatar: lang('avatar', 'avatar', {
    blurb:
      'An identity frame: image, icon slug, or initials — and the broken-image path falls back to initials automatically, via one shared action.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'src', desc: 'the image — falls back on error', type: 'string' },
      { name: 'icon-slug', desc: 'pack glyph id when no photo', type: 'string' },
      { name: 'name', desc: 'identity label for fallback initials', type: 'string' },
      { name: 'initials', desc: 'explicit fallback glyphs', type: 'string' },
      { name: 'tone', desc: 'frame colour (default muted)', type: TONES },
      { name: 'size', desc: 'frame scale', type: 'xs · sm · md · lg · xl' },
      { name: 'shape', desc: 'circle or rounded square', type: 'round · square' },
    ],
  }),
  icon: lang('icon', 'icon', {
    blurb:
      'Glyph box on the icon-size ladder. Prefer name for pack ids (local /_deck/icons resolve — never CDN); src only same-origin SVG. Empty label is decorative (aria-hidden); set label for role=img. Ink is currentColor + glyphFill (no tone prop). spin is owned by use:icon motion.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'name', desc: 'pack glyph id → local /_deck/icons/{name}.svg', type: 'string' },
      { name: 'src', desc: 'same-origin / relative SVG only (no CDN)', type: 'string' },
      { name: 'label', desc: 'a11y name; empty → decorative aria-hidden', type: 'string' },
      { name: 'size', desc: 'glyph box on icon-size scale (xs→2xs … xl→2xl)', type: 'xs · sm · md · lg · xl' },
      { name: 'spin', desc: 'loop rotate via use:icon (not a CSS class)', type: 'bool' },
      { name: '(slot)', desc: 'optional one-off SVG when not using name/src', type: 'children' },
    ],
  }),
  separator: lang('separator', 'separator', {
    blurb:
      'Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. tone defaults to muted. No separate `hr` catalog entry.',
    variants: ['solid', 'dashed'],
    variantAttr: 'variant',
    props: [
      { name: 'orientation', desc: 'which axis it divides', type: 'horizontal · vertical' },
      { name: 'tone', desc: 'line colour (default muted)', type: 'muted · primary · destructive' },
      { name: 'variant', desc: 'line style', type: 'solid · dashed' },
    ],
  }),
  accordion: lang('accordion', 'accordion', {
    blurb:
      'ONE disclosure element (replaces collapsible + details). open · disabled · name · multiple · onToggle. Shared name = exclusive multi-item (native details name). Background, text, border, hover, and chevron come from the theme.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'open', desc: 'expanded state, reflected', type: 'bool' },
      { name: 'disabled', desc: 'locks activation', type: 'bool' },
      { name: 'name', desc: 'exclusive group (native details name)', type: 'string' },
      { name: 'multiple', desc: 'allow several open when not exclusiving via name', type: 'bool' },
      { name: 'onToggle', desc: 'fires with { open } on expand/collapse', type: 'callback' },
      { name: 'slot="trigger"', desc: 'the always-visible line', type: 'slot' },
    ],
  }),
  kbd: lang('kbd', 'kbd', {
    blurb:
      'Native <kbd> keycap — mono type, muted surface, doubled bottom edge. Colours come from the theme; no size/variant/tone props (compose chords in markup).',
    variants: [],
    variantAttr: '',
    props: [
      { name: '(children)', desc: 'key label — Ctrl, K, ⌘, F12, …', type: 'children' },
    ],
  }),
  'code-block': lang('code-block', 'codeBlock', {
    blurb:
      'Syntax-highlighted source block. Host is a div with use:syntaxHighlight; the inner pre uses theme colours plus global border/radius. Pass code as a string prop (not children). lang defaults to sail and is an authoring attr — the highlighter classifies by content shape (markup vs code), not by inventing line numbers or a copy chrome.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'code', desc: 'source text to highlight into the pre', type: 'string' },
      { name: 'lang', desc: 'authoring language label (default sail); highlighter auto-classifies markup vs code', type: 'string' },
    ],
  }),
  prose: lang('prose', 'prose', {
    heading: 'prose',
    blurb:
      'Typography scope for rendered content — wrap markdown blobs and long copy in <prose>. Host text colour comes from the theme; nested h1…p, lists, blockquote, and code pick up the framework type map. No size/variant/tone props (theme type roles, not a Title API).',
    variants: [],
    variantAttr: '',
    props: [
      { name: '(children)', desc: 'plain tags — h1…h6, p, lists, blockquote, table, pre/code, inlines', type: 'children' },
    ],
  }),

  // ── Overlay ──────────────────────────────────────────────────────────
  tooltip: lang('tooltip', 'tooltip', {
    blurb:
      'A label on hover and focus, from the content attribute — the bubble is filled by the action, positioned by placement, and never steals focus.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'which side it opens on', type: 'top · bottom · left · right' },
      { name: 'content', desc: 'the tooltip text', type: 'string' },
      { name: '(children)', desc: 'the trigger — whatever you wrap', type: 'children' },
    ],
  }),
  'hover-card': lang('hover-card', 'hoverCard', {
    blurb:
      'A rich panel on hover or focus of its trigger — the button whose popovertarget names it. Native top layer; open/close delays so it never flickers.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'which side it opens on (free string)', type: 'string' },
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: '(children)', desc: 'the panel content', type: 'children' },
    ],
  }),
  popover: lang('popover', 'popover', {
    blurb:
      'The native [popover], positioned. Top layer, Escape and light-dismiss come from the platform; use:anchor adds placement — that is the whole element.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'which side it opens on (free string)', type: 'string' },
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: '(children)', desc: 'the panel content', type: 'children' },
    ],
  }),
  dropdown: lang('dropdown', 'dropdown', {
    blurb:
      'A popover with menu semantics: role=menu, roving arrows, type-ahead, and <menu-item> children that close it on select.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'which side it opens on (free string)', type: 'string' },
      { name: 'onSelect', desc: 'fires with { value } when a menu-item is chosen', type: 'callback' },
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: '<menu-item value>', desc: 'the items — tone for destructive', type: 'children' },
    ],
  }),
  'context-menu': lang('context-menu', 'contextMenu', {
    blurb:
      'The dropdown’s right-click sibling: same menu semantics, opened at the pointer on contextmenu of the target that for= names.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'for', desc: 'id of the right-click target', type: 'string' },
      { name: 'onSelect', desc: 'fires with { value } when a menu-item is chosen', type: 'callback' },
      { name: '<menu-item value>', desc: 'the items', type: 'children' },
    ],
  }),
  dialog: lang('dialog', 'dialog', {
    blurb:
      'The platform dialog — top layer, focus trap, Escape, backdrop — opened and closed by invoker commands. Zero JavaScript; side= docks it to an edge as a sheet.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'size', desc: 'clamp the surface width', type: 'sm · md · lg · full' },
      { name: 'backdrop', desc: 'scrim treatment', type: 'default · blur · solid' },
      { name: 'side', desc: 'center or dock as sheet', type: 'center · start · end · top · bottom' },
      { name: 'dismissible', desc: 'ESC + backdrop can close', type: 'bool' },
      { name: 'onClose', desc: 'close callback', type: 'callback' },
      { name: 'onDismiss', desc: 'dismiss callback', type: 'callback' },
      { name: 'command / commandfor', desc: 'invoker buttons open and close it', type: 'show-modal · close' },
      { name: '<header> <section> <footer>', desc: 'title / scrollable body / actions', type: 'children' },
    ],
  }),
  drawer: lang('drawer', 'drawer', {
    blurb:
      'Edge-anchored non-modal sheet. side start|end|top|bottom (default end); size sm|md|lg. Native <dialog> via command="show" (not showModal). Header/section/footer as light-dom children. Swipe toward the edge closes. Paint card / card-foreground.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'size', desc: 'clamp the surface width / height', type: 'sm · md · lg' },
      { name: 'side', desc: 'edge the sheet docks to', type: 'start · end · top · bottom' },
      { name: 'dismissible', desc: 'ESC + swipe + outside pointer can close', type: 'bool' },
      { name: 'onClose', desc: 'close callback', type: 'callback' },
      { name: 'onDismiss', desc: 'dismiss callback', type: 'callback' },
      { name: 'command / commandfor', desc: 'invoker buttons open and close it', type: 'show · close' },
      { name: '<header> <section> <footer>', desc: 'title / scrollable body / actions', type: 'children' },
    ],
  }),

  // ── Navigation ───────────────────────────────────────────────────────
  tabs: lang('tabs', 'tabs', {
    blurb:
      'Write a tab as its title plus the panel it owns. Arrows rove, Home/End jump, ARIA is automatic. orientation=vertical puts the rail beside the panel. Strip look via variant on <tab> or <tab-list>.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'orientation', desc: 'horizontal (triggers above) or vertical (list beside — parts form)', type: 'horizontal · vertical' },
      { name: 'onTabChange', desc: 'fires with { name } when the user picks a tab', type: 'callback' },
      {
        name: '<tab title>',
        desc: 'title is the trigger; default slot is the panel',
        type: 'title + children',
      },
      {
        name: '<tab-list variant>',
        desc: 'optional strip for pill/enclosed/vertical rail',
        type: 'line · pill · enclosed',
      },
    ],
  }),
  breadcrumb: lang('breadcrumb', 'breadcrumb', {
    blurb: 'Where you are. Earlier steps are links.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'size', desc: 'sm · md · lg', type: 'sm · md · lg' },
      { name: '<a>', desc: 'a step you can go back to', type: 'children' },
      { name: '<small>', desc: 'this page', type: 'children' },
    ],
  }),
  pagination: lang('pagination', 'pagination', {
    blurb:
      'A nav of <page-item> controls. value marks the current one with aria-current; clicking emits sail-page-change — your list, your fetch. Size is authored on each <page-item>.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'the current page', type: 'string' },
      { name: '<page-item value size>', desc: 'each control — prev/next are items too; size on each item', type: 'children' },
      { name: 'on-page-change', desc: 'page-change callback, { page }', type: 'callback' },
    ],
  }),
  navbar: lang('navbar', 'navbar', {
    blurb:
      'One element, three placements — top bar, side rail, bottom bar. sticky · bordered · autohide · offset. Regions brand / center / actions.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'top · bottom · rail', type: 'top · bottom · rail' },
      { name: 'sticky', desc: 'pins to the edge it sits on', type: 'bool' },
      { name: 'bordered', desc: 'hairline on the content-facing edge', type: 'bool' },
      { name: 'autohide', desc: 'tuck on scroll-down when sticky', type: 'bool' },
      { name: 'offset', desc: 'sticky inset (number)', type: 'number' },
      { name: 'slot="brand" / "center" / "actions"', desc: 'the regions — links go unslotted', type: 'slots' },
    ],
  }),

  // ── Data ─────────────────────────────────────────────────────────────
  table: lang('table', 'table', {
    blurb:
      "The platform's table model — thead, tbody, tr — skinned by the theme. Hairline, header fill, and row hover on by default. searchable adds a filter field; <th sortable> cycles row order.",
    variants: [],
    variantAttr: '',
    props: [
      { name: 'dense', desc: 'tighter cell padding', type: 'bool' },
      { name: 'striped', desc: 'even rows get a light stripe from the theme', type: 'bool' },
      { name: 'hoverable', desc: 'row hover fill from the stripe colour', type: 'bool' },
      { name: 'bordered', desc: 'box every cell', type: 'bool' },
      { name: 'searchable', desc: 'search field above the table; filters rows as you type', type: 'bool' },
      { name: '<th sortable>', desc: 'click cycles none → asc → desc; numeric compares as numbers', type: 'bool' },
      { name: '<thead> <tbody> <tr>', desc: 'the platform table model', type: 'children' },
      { name: '{#for}', desc: 'rows render from state and stay live', type: 'block' },
    ],
  }),

  // ── Layout ───────────────────────────────────────────────────────────
  layout: lang('layout', 'layout', {
    heading: 'row · column · grid · center',
    blurb:
      'Not components — keywords. The preprocessor lowers them to a div plus exactly the rules used, and they are the one family that accepts dynamic values.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'row / column', desc: 'the x and y axes', type: 'keyword' },
      { name: 'grid cols=', desc: 'two dimensions', type: 'keyword' },
      { name: 'center', desc: 'both axes at once', type: 'keyword' },
      { name: 'gap / align / justify', desc: 'sibling spacing and flow — dynamic values allowed', type: 'scale · keywords' },
    ],
  }),
  'scroll-area': lang('scroll-area', 'scrollArea', {
    blurb:
      'Overflow box with thin, themed scrollbars. Only orientation is a host prop (vertical · horizontal · both; default vertical). Constrain height/width with Style box so content overflows. Thumb colour comes from the theme; hover uses the stronger border colour. Standard scrollbar-width/color only (no WebKit pseudo setters yet).',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'orientation',
        desc: 'scroll axis: vertical (overflow-y auto, x hidden), horizontal (x auto, y hidden), or both (overflow auto); default vertical',
        type: 'vertical · horizontal · both',
      },
      {
        name: 'children',
        desc: 'content that overflows the constrained host',
        type: 'slot',
      },
    ],
  }),
  resizable: lang('resizable', 'resizable', {
    blurb:
      'Two-pane split: start and end slots with a drag handle between. orientation horizontal (default) or vertical. Drag — or arrow-nudge — the separator to size the start pane. Handle is a focusable role=separator.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'orientation',
        desc: 'split axis: horizontal (default) or vertical',
        type: 'horizontal · vertical',
      },
      { name: 'slot="start" / "end"', desc: 'the two panes', type: 'slots' },
    ],
  }),

  // ── Effects ──────────────────────────────────────────────────────────
  ticker: lang('ticker', 'ticker', {
    blurb:
      'A looping band of text or icons. Drop content in — the host already clips, nowraps, and paints a quiet strip. duration and reverse set the motion. Reduced motion settles automatically.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'duration',
        desc: 'loop length for use:marquee (fast 16s · normal 24s · slow 32s)',
        type: 'fast · normal · slow',
      },
      {
        name: 'reverse',
        desc: 'presence attr — flip to +50% (rightward)',
        type: 'boolean',
      },
      {
        name: 'children',
        desc: 'band content — second copy is decorative/aria-hidden for the seam',
        type: 'slot',
      },
    ],
  }),
  // Actions live in the action table, not in this element detail
  // catalog. (`spotlight` / `targetCursor` were here when there was
  // an `fx` pack; both packs and entries are gone now.)
} satisfies Record<string, DetailEntry>

// Closed map: DETAILS.card / DETAILS['hover-card'] are DetailEntry.
// Record<string, …> would make every index `| undefined` under
// noUncheckedIndexedAccess (kind gate). Dynamic slugs stay honest
// via detailFor().
export const DETAILS = DETAILS_MAP

export type DetailTag = keyof typeof DETAILS

export const detailFor = (tag: string): DetailEntry | undefined =>
  (DETAILS as Record<string, DetailEntry>)[tag]

export const DETAIL_TAGS = Object.keys(DETAILS)
