// /components/[tag] — detail data for the FULL L1 surface (the 48 elements
// the l1-conformance gate renders, minus nothing). slug → { heading,
// blurb, variants, props, usage }. Props tables are GENERATED from the
// compiler's prop registry (see genProps below); regen with
// `cargo run -p sail-compiler -- prop-registry` after touching a stdlib
// <scope> props declaration. The usage strings live in
// ./snippets.ts so the site-wide snippet guard compiles every one
// of them.
import { PROP_REGISTRY } from '../prop-registry.gen'
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

export const DETAILS: Record<string, DetailEntry> = {
  // ── Inputs ───────────────────────────────────────────────────────────
  button: lang('button', 'button', {
    blurb:
      'The platform button. tone paints the colour; variant is how much of it you get (solid fill, soft, outline, ghost). default is an alias of solid.',
    variants: ['solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      {
        name: 'variant',
        desc: 'fill model — default ≡ solid; soft / outline / ghost take less paint',
        type: 'default · solid · soft · outline · ghost',
      },
      { name: 'tone', desc: 'semantic colour — always applied when set', type: TONES },
      { name: 'size', desc: 'text control height, or square icon hit target', type: 'xs · sm · md · lg · icon · iconSm · iconLg' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
      { name: 'children', desc: 'label and optional <icon> siblings (gap is built-in)', type: 'slot' },
      { name: 'style({ animate: press })', desc: 'press scale feedback', type: 'preset' },
    ],
  }),
  input: lang('input', 'input', {
    blurb:
      'The platform input, skinned by the theme. bind:value keeps it and a signal in lockstep; every type=, form wiring and validity state is the native one.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'two-way signal binding', type: 'signal' },
      { name: 'placeholder', desc: 'hint text in the empty field', type: 'string' },
      { name: 'type', desc: 'the full platform set', type: 'text · email · password · …' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
    ],
  }),
  textarea: lang('textarea', 'textarea', {
    blurb:
      'The platform textarea, themed. bind:value for two-way state; rows and the rest of the native surface pass straight through.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'two-way signal binding', type: 'signal' },
      { name: 'placeholder', desc: 'hint text', type: 'string' },
      { name: 'rows', desc: 'initial height, in lines', type: 'number' },
    ],
  }),
  select: lang('select', 'select', {
    blurb:
      "The native select — keyboard, form submission and mobile pickers come from the platform. The theme restyles the closed control; the option list is the OS's own.",
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:value', desc: 'two-way signal binding', type: 'signal' },
      { name: '<option>', desc: 'the platform option model', type: 'children' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
    ],
  }),
  combobox: lang('combobox', 'combobox', {
    blurb:
      'A text input that filters a list. Type to filter, arrows rove, Enter commits, Escape closes — the ARIA combobox pattern wired for you, themed like everything else.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placeholder', desc: 'hint text in the empty field', type: 'string' },
      { name: 'value', desc: 'the committed value, reflected', type: 'string' },
      { name: '<option>', desc: 'the list — plain option children', type: 'children' },
      { name: 'on-select', desc: 'commit callback, { value, label }', type: 'callback' },
    ],
  }),
  checkbox: lang('checkbox', 'checkbox', {
    blurb:
      'The platform checkbox in theme colours. bind:checked pairs it with a signal; indeterminate, forms and keyboard are the native ones.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:checked', desc: 'two-way signal binding', type: 'signal' },
      { name: 'checked', desc: 'initial state', type: 'bool' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
    ],
  }),
  radio: lang('radio', 'radio', {
    blurb:
      'One option — native radio inside a label host. value is this option’s value; disabled is a host attribute radio-group syncs onto the nested input (input when:disabled dims the control). Label ink from component.radio.fg (--radio-foreground); control face uses free foreground/background, checked primary + RADIO_INPUT_MAP inset disc. No size/tone props. Group exclusivity, name, and onChange are radio-group composition (separate L1).',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'this option’s value (group selects by match)', type: 'string' },
      {
        name: 'disabled',
        desc: 'host attr for radio-group → input.disabled + PE dim',
        type: 'bool',
      },
      { name: '(children)', desc: 'option label text', type: 'children' },
    ],
  }),
  'radio-group': lang('radio-group', 'radioGroup', {
    blurb:
      'Groups native radios under one accessible name and one value. The platform owns exclusivity and keyboard; the group syncs ARIA and emits one change.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'the selected value, reflected', type: 'string' },
      { name: 'label', desc: 'accessible group name', type: 'string' },
      { name: 'name', desc: 'the native radio name', type: 'string' },
      { name: 'disabled / required', desc: 'platform states, styled', type: 'bool' },
    ],
  }),
  switch: lang('switch', 'switchToggle', {
    blurb:
      'Native checkbox restyled as a sliding toggle (role=switch). bind:checked locks the host to a signal. size sm|md|lg scales track + thumb. tone paints the checked track only; omit tone and checked uses primary. Off track/border from component.switch.track + border; thumb always component.switch.thumb (never tone.foreground).',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'bind:checked', desc: 'two-way signal binding (lockstep with host)', type: 'signal' },
      { name: 'checked', desc: 'on/off state (also reflectable)', type: 'bool' },
      {
        name: 'tone',
        desc: 'checked track (+ focus) only; omit → primary when on',
        type: TONES,
      },
      { name: 'size', desc: 'track + thumb scale (default md)', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'native freeze; PE keeps off-track tokens + lighter thumb shadow', type: 'bool' },
    ],
  }),
  slider: lang('slider', 'slider', {
    blurb:
      'A native range input, skinned. The platform owns drag, keyboard, role=slider and form submission; the language adds the filled track and the theme.',
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    props: [
      { name: 'bind:value', desc: 'two-way signal binding', type: 'signal' },
      { name: 'min / max / step', desc: 'the platform range model', type: 'number' },
      { name: 'size', desc: 'control scale', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
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
      'The label-plus-control unit. It wires label/for, spaces the pair, and carries the invalid state so the message and the border agree.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'invalid', desc: 'error state for the whole unit', type: 'bool' },
      { name: '<label> <input>', desc: 'the pair — for/id wiring is yours', type: 'children' },
    ],
  }),
  label: lang('label', 'label', {
    blurb:
      'Native <label> — for= focuses the control. size sm|md|lg matches the control type ladder. required shows the * marker from component.label.required; host ink from component.label.fg and weight from typography.label.weight. No tone prop.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'for', desc: 'native association — click focuses the control', type: 'id' },
      { name: 'size', desc: 'type scale matching control size', type: 'sm · md · lg' },
      {
        name: 'required',
        desc: 'shows * marker (component.label.required via group-required)',
        type: 'bool',
      },
      { name: '(children)', desc: 'label text', type: 'children' },
    ],
  }),
  fieldset: lang('fieldset', 'fieldset', {
    blurb:
      'Native form group with a themeable border (component.fieldset.border). Put a legend first, then controls. disabled freezes every control inside (platform) and dims the host; no tone/size/variant axes.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'disabled', desc: 'native — freezes controls inside; PE dims opacity', type: 'bool' },
      { name: '<legend>', desc: 'group caption (compose legend PE)', type: 'children' },
      { name: 'default', desc: 'controls and copy inside the group', type: 'slot' },
    ],
  }),
  legend: lang('legend', 'legend', {
    blurb:
      'Native <legend> fieldset caption — medium sm type with horizontal pad. Ink from component.legend.fg (--legend-foreground); no size/variant/tone props (theme the leaf; nest inside <fieldset>).',
    variants: [],
    variantAttr: '',
    props: [
      { name: '(children)', desc: 'caption text for the group', type: 'children' },
    ],
  }),
  dropzone: lang('dropzone', 'dropzone', {
    blurb:
      'An attribute, not a component: put dropzone on a surface and use:dropzone wires dragover states, click-to-browse and the file input.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'dropzone', desc: 'the attribute that makes any surface one', type: 'attr' },
      { name: 'tone', desc: 'border and highlight colour', type: TONES },
    ],
  }),
  'theme-picker': lang('theme-picker', 'themePicker', {
    blurb:
      'The theme switcher: a trigger that opens a native [popover] panel of theme cards, each previewing the surface it selects with the real stylesheet.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'the active theme, reflected', type: 'string' },
      { name: 'ship:theme:change', desc: 'announced on selection', type: 'callback' },
    ],
  }),

  // ── Feedback ─────────────────────────────────────────────────────────
  alert: lang('alert', 'alert', {
    blurb:
      'Inline status banner. Omit tone for component.alert tokens (themeable rest); set tone and the left stripe + icon brand paint. warning/destructive flip liveRegion to assertive role=alert; dismissible shows the close control (onDismiss then remove).',
    // No variant axis — stage owns tone/dismiss knobs (viewer chips hidden).
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'tone',
        desc: 'stripe + icon brand when set; omit for component.alert rest tokens',
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
      'Native progress bar, themed. value/max are the platform model; omit value for the indeterminate sweep (no separate prop). size is sm|md|lg track height. Paint always from component.progress.track + fill — no tone axis.',
    variants: [],
    variantAttr: '',
    props: [
      {
        name: 'value',
        desc: 'current amount; omit attribute entirely for native indeterminate + PROGRESS_MAP shimmer',
        type: 'number',
      },
      { name: 'max', desc: 'upper bound of the range (default 1 on the PE host)', type: 'number' },
      {
        name: 'size',
        desc: 'track height ladder (default md)',
        type: 'sm · md · lg',
      },
    ],
  }),
  meter: lang('meter', 'meter', {
    blurb:
      'Native <meter> gauge — a measurement in range, not task progress. value/min/max set the reading; low/high/optimum drive platform fill bands painted from component.meter.track / optimum / suboptimum / sub-suboptimum. size scales bar height. No tone prop.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value / min / max', desc: 'the measurement (platform meter model)', type: 'number' },
      {
        name: 'low / high / optimum',
        desc: 'thresholds → optimum · suboptimum · even-less-good band tokens',
        type: 'number',
      },
      { name: 'size', desc: 'bar height', type: 'sm · md · lg' },
    ],
  }),
  spinner: lang('spinner', 'spinner', {
    blurb:
      'Indeterminate wait ring. Size scales the host; omit tone for themeable spinner.foreground / spinner.track; centered parks it in the middle of its container.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'size', desc: 'ring scale (icon-size ladder)', type: 'xs · sm · md · lg' },
      {
        name: 'tone',
        desc: 'active arc colour (fg). Omit → component.spinner.fg rest; track always spinner.track',
        type: 'primary · info · success · warning · destructive · accent',
      },
      { name: 'centered', desc: 'block + auto horizontal margin in the parent', type: 'bool' },
    ],
  }),
  skeleton: lang('skeleton', 'skeleton', {
    blurb:
      'Loading placeholder sheen. Shape + size draw the geometry; paint is always the shimmer from component.skeleton.bg / highlight (themeable). No tone axis — compose width with layout Style.',
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
      'Nothing-here surface: icon + title + body + action slots. Chrome via variant (bordered default, dashed inline, bare placeholder/connecting). Omit tone for component.empty-state.* (themeable rest); set tone to brand the icon only. connecting upgrades to role=status / aria-live=polite via use:variantLive.',
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
        desc: 'icon brand when set; omit for component.empty-state rest; muted keeps rest icon fg',
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
      'Recovery surface for failed loads and partial failures. Rest chrome paints component.error-state (bg · fg · border); icon + inline border keep brand destructive. code fills a mono chip; details slot gates Show details; action slot holds Retry. use:recoverySurface stamps role=alert, syncs code, and focuses the first action.',
    variants: ['default', 'inline'],
    variantAttr: 'variant',
    props: [
      {
        name: 'variant',
        desc: 'default = centered card on component leaves; inline = start-aligned row with destructive border + muted fill',
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
    variants: ['success', 'info', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'tone', desc: 'semantic colour token', type: TONES },
      { name: 'duration', desc: 'ms until dismiss — 0 is sticky', type: 'number' },
      { name: 'onDismiss', desc: 'fires before remove — drop the instance from your list', type: 'callback' },
      { name: 'data-toast-close', desc: 'marks a custom close control (L1 ships one)', type: 'attr' },
      { name: '<toast-region position>', desc: 'the fixed stack it lives in', type: 'top/bottom-start/end' },
    ],
  }),

  // ── Display ──────────────────────────────────────────────────────────
  badge: lang('badge', 'badge', {
    blurb:
      'A small statement of state. Soft by default; solid or outline when it must shout or whisper. Omit tone to use component.badge tokens (themeable rest); set tone and the semantic colour always paints.',
    variants: ['soft', 'solid', 'outline'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'fill strength — soft default', type: 'soft · solid · outline' },
      {
        name: 'tone',
        desc: 'semantic colour when set; omit for component.badge rest tokens',
        type: TONES,
      },
      { name: 'size', desc: 'badge scale for table cells and chrome', type: 'sm · md · lg' },
      { name: 'pill', desc: 'fully rounded — counters and filters', type: 'bool' },
      { name: 'children', desc: 'label text (keep short)', type: 'slot' },
    ],
  }),
  chip: lang('chip', 'chip', {
    blurb:
      'An interactive filter pill — the host is the control (role, keyboard, aria-pressed). Omit tone for component.chip rest tokens; set tone and the semantic colour always paints. removable adds the ×; Backspace/Delete fires on-remove.',
    variants: ['solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'fill strength — solid default', type: 'solid · soft · outline · ghost' },
      {
        name: 'tone',
        desc: 'semantic colour when set; omit for component.chip rest tokens',
        type: TONES,
      },
      { name: 'size', desc: 'chip scale for filters and dense chrome', type: 'sm · md · lg' },
      { name: 'selected', desc: 'pressed state, ARIA-synced', type: 'bool' },
      { name: 'removable', desc: 'the × button + Backspace/Delete', type: 'bool' },
      { name: 'disabled', desc: 'non-interactive; muted rest paint', type: 'bool' },
      { name: 'on-remove', desc: 'remove callback (× and keyboard)', type: 'callback' },
      { name: 'leading', desc: 'optional leading icon/slot', type: 'slot' },
      { name: 'children', desc: 'label text (keep short)', type: 'slot' },
    ],
  }),
  card: lang('card', 'card', {
    blurb:
      'The surface everything else sits on. Padding, tone, an accent bar and the hover lift are attributes — the card never needs a stylesheet.',
    variants: ['default', 'outline'],
    variantAttr: 'variant',
    props: [
      { name: 'tone', desc: 'tint the surface', type: TONES },
      { name: 'variant', desc: 'filled or transparent', type: 'default · outline' },
      { name: 'padding', desc: 'inset scale', type: 'none · sm · md · lg' },
      { name: 'accent-bar', desc: 'a tone-coloured leading bar', type: 'off · hover · static' },
      { name: 'interactive', desc: 'pointer cursor + CSS hover lift, one attribute', type: 'boolean' },
    ],
  }),
  avatar: lang('avatar', 'avatar', {
    blurb:
      'An identity frame: image, icon slug, or initials — and the broken-image path falls back to initials automatically, via one shared action.',
    variants: ['sm', 'md', 'lg', 'xl'],
    variantAttr: 'size',
    props: [
      { name: 'src', desc: 'the image — falls back on error', type: 'url' },
      { name: 'name / initials', desc: 'what the fallback draws', type: 'string' },
      { name: 'tone', desc: 'frame colour token', type: TONES },
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
      'Themed divider — lowers to a native <hr>. Horizontal by default, vertical between inline things; solid or dashed. Omit tone for themeable component.separator.line; set muted · primary · destructive only when the line must speak. No separate `hr` catalog entry.',
    variants: ['solid', 'dashed'],
    variantAttr: 'variant',
    props: [
      { name: 'orientation', desc: 'which axis it divides', type: 'horizontal · vertical' },
      { name: 'tone', desc: 'optional line colour — omit for rest separator.line tokens', type: 'muted · primary · destructive' },
      { name: 'variant', desc: 'line style', type: 'solid · dashed' },
    ],
  }),
  accordion: lang('accordion', 'accordion', {
    blurb:
      'ONE disclosure element (replaces collapsible + details). Trigger slot, disabled, onToggle; exclusive multi-item via shared name.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'slot="trigger"', desc: 'the always-visible line', type: 'slot' },
      { name: 'open', desc: 'initial state, reflected', type: 'bool' },
      { name: 'disabled', desc: 'locks activation', type: 'bool' },
      { name: 'name', desc: 'exclusive group (native details name)', type: 'string' },
      { name: 'multiple', desc: 'allow several open when not exclusiving via name', type: 'bool' },
    ],
  }),
  kbd: lang('kbd', 'kbd', {
    blurb:
      'Native <kbd> keycap — mono type, muted surface, doubled bottom edge. Paint from component.kbd (bg · foreground · border); no size/variant/tone props (theme the leaves, compose chords in markup).',
    variants: [],
    variantAttr: '',
    props: [
      { name: '(children)', desc: 'key label — Ctrl, K, ⌘, F12, …', type: 'children' },
    ],
  }),
  'code-block': lang('code-block', 'codeBlock', {
    blurb:
      'Syntax-highlighted source block. Host is a div with use:syntaxHighlight; paint on the inner pre from component.code-block (bg · foreground) plus global border/radius. Pass code as a string prop (not children). lang defaults to sail and is an authoring attr — the highlighter classifies by content shape (markup vs code), not by inventing line numbers or a copy chrome.',
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
      'Typography scope for rendered content — wrap markdown blobs and long copy in <prose>. Host ink is component.prose.fg; nested h1…p, lists, blockquote, and code pick up the framework type map. No size/variant/tone props (theme the leaf + type roles, not a Title API).',
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
    variants: ['top', 'bottom', 'left', 'right'],
    variantAttr: 'placement',
    props: [
      { name: 'content', desc: 'the tooltip text', type: 'string' },
      { name: 'placement', desc: 'which side it opens on', type: 'top · bottom · left · right' },
      { name: '(children)', desc: 'the trigger — whatever you wrap', type: 'children' },
    ],
  }),
  'hover-card': lang('hover-card', 'hoverCard', {
    blurb:
      'A rich panel on hover or focus of its trigger — the button whose popovertarget names it. Native top layer; open/close delays so it never flickers.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: 'placement', desc: 'which side it opens on', type: 'top · bottom · left · right (+-end)' },
      { name: 'open-delay / close-delay', desc: 'hover intent, in ms', type: 'number' },
    ],
  }),
  popover: lang('popover', 'popover', {
    blurb:
      'The native [popover], positioned. Top layer, Escape and light-dismiss come from the platform; use:anchor adds placement — that is the whole element.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: 'placement', desc: 'which side it opens on', type: 'top · bottom · left · right (+-end)' },
      { name: '(children)', desc: 'the panel content', type: 'children' },
    ],
  }),
  dropdown: lang('dropdown', 'dropdown', {
    blurb:
      'A popover with menu semantics: role=menu, roving arrows, type-ahead, and <menu-item> children that close it on select.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'id + popovertarget', desc: 'the trigger link — no refs', type: 'wiring' },
      { name: '<menu-item value>', desc: 'the items — tone for destructive', type: 'children' },
      { name: 'placement', desc: 'which side it opens on', type: 'top · bottom · left · right (+-end)' },
    ],
  }),
  'context-menu': lang('context-menu', 'contextMenu', {
    blurb:
      'The dropdown’s right-click sibling: same menu semantics, opened at the pointer on contextmenu of the target that for= names.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'for', desc: 'id of the right-click target', type: 'id' },
      { name: '<menu-item value>', desc: 'the items', type: 'children' },
    ],
  }),
  dialog: lang('dialog', 'dialog', {
    blurb:
      'The platform dialog — top layer, focus trap, Escape, backdrop — opened and closed by invoker commands. Zero JavaScript; side= docks it to an edge as a sheet.',
    // Chips label modal vs sheet; page.sail maps sheet → side=end (not a real attr).
    variants: ['modal', 'sheet'],
    variantAttr: 'data-dialog-mode',
    props: [
      { name: 'command / commandfor', desc: 'invoker buttons open and close it', type: 'show-modal · close' },
      { name: 'size', desc: 'clamp the surface width', type: 'sm · md · lg' },
      { name: 'side', desc: 'dock to an edge — the sheet', type: 'start · end' },
      { name: '<header> <section> <footer>', desc: 'title / scrollable body / actions', type: 'children' },
    ],
  }),

  // ── Navigation ───────────────────────────────────────────────────────
  tabs: lang('tabs', 'tabs', {
    blurb:
      'Tabs with the full keyboard suite — arrows rove, Home/End jump, panels wire their ARIA automatically. Three looks via variant on <tab-list>; size is authored on each <tab>.',
    variants: ['line', 'pill', 'enclosed'],
    variantAttr: 'variant',
    props: [
      { name: 'active', desc: 'initially selected tab', type: 'string' },
      { name: 'orientation', desc: 'stack the list vertically', type: 'horizontal · vertical' },
      {
        name: '<tab-list variant>',
        desc: 'the look of the tab strip — line · pill · enclosed (lives on the strip, not <tabs>)',
        type: 'line · pill · enclosed',
      },
      {
        name: '<tab size>',
        desc: 'each tab — size Style match lives here',
        type: 'sm · md · lg',
      },
    ],
  }),
  breadcrumb: lang('breadcrumb', 'breadcrumb', {
    blurb:
      'A trail of links with the right nav/ARIA and themed separators. Anchors for the path, something inert for where you are.',
    variants: [],
    variantAttr: '',
    props: [
      { name: '<a>', desc: 'the path — ordinary links', type: 'children' },
      { name: '<small>', desc: 'the current page, inert', type: 'children' },
    ],
  }),
  pagination: lang('pagination', 'pagination', {
    blurb:
      'A nav of <page> controls. value marks the current one with aria-current; clicking emits sail-page-change — your list, your fetch. Size is authored on each <page>.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'the current page', type: 'string' },
      { name: '<page value size>', desc: 'each control — prev/next are pages too; size on each page', type: 'children' },
      { name: 'on-page-change', desc: 'page-change callback, { page }', type: 'callback' },
    ],
  }),
  navbar: lang('navbar', 'navbar', {
    blurb:
      'One element, three placements — top bar, side rail, bottom bar. Four regions (brand, links, center, actions); bordered and sticky follow the placement.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'placement', desc: 'a rail is a navbar on its side', type: 'top · rail · bottom' },
      { name: 'sticky', desc: 'pins to the edge it sits on', type: 'bool' },
      { name: 'bordered', desc: 'hairline on the content-facing edge', type: 'bool' },
      { name: 'slot="brand" / "center" / "actions"', desc: 'the regions — links go unslotted', type: 'slots' },
    ],
  }),

  // ── Data ─────────────────────────────────────────────────────────────
  table: lang('table', 'table', {
    blurb:
      "The platform's table model — thead, tbody, tr — skinned by the theme. No grid component to learn: markup in, a designed table out.",
    variants: [],
    variantAttr: '',
    props: [
      { name: '<thead> <tbody> <tr>', desc: 'the platform table model', type: 'children' },
      { name: '{#for}', desc: 'rows render from state and stay live', type: 'block' },
      { name: 'density', desc: 'row padding comes from theme tokens', type: 'theme' },
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
      'A scroll container with thin, themed scrollbars. Constrain it with sx and the overflow scrolls; orientation picks the axis.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'orientation', desc: 'which axis scrolls', type: 'vertical · horizontal · both' },
      { name: 'sx="max-h:…"', desc: 'the constraint that makes it scroll', type: 'sx' },
    ],
  }),

  // ── Effects ──────────────────────────────────────────────────────────
  ticker: lang('ticker', 'ticker', {
    blurb:
      'Infinite linear marquee. Two copies of the slot ride a track; use:marquee loops translate-x so the seam never shows. duration and reverse are props.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'duration', desc: 'loop speed', type: 'fast · normal · slow' },
      { name: 'reverse', desc: 'flip direction', type: 'boolean' },
      { name: 'children', desc: 'the band content — duplicated for a seamless seam', type: 'slot' },
    ],
  }),
  // Actions live in the action table, not in this element detail
  // catalog. (`spotlight` / `targetCursor` were here when there was
  // an `fx` pack; both packs and entries are gone now.)
}

export const DETAIL_TAGS = Object.keys(DETAILS)
