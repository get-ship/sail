// /components/[tag] — detail data for the FULL L1 surface (the 48 elements
// the l1-conformance gate renders, minus nothing). slug → { heading,
// blurb, variants, props, usage }. Props tables are GENERATED from the
// compiler's prop registry (see genProps below); regen with
// `cargo run -p sui-compiler -- prop-registry` after touching a stdlib
// <component>{ props } declaration. The usage strings live in
// ./snippets.ts so the site-wide snippet guard compiles every one
// of them.
import { PROP_REGISTRY } from '../../../src/prop-registry.gen'
import { usage } from './snippets'

export interface PropRow {
  name: string
  desc: string
  type: string
}

export interface DetailEntry {
  /** The element tag — breadcrumb + copy. */
  tag: string
  /** The scope key page.sui mounts the live stage through (`stage.<key>()`). */
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

const TONES = 'default · primary · info · success · warning · destructive · accent · muted'

// The props table is GENERATED: names, types and defaults come from the
// compiler's prop registry (the same <component>{ props } declarations the
// build validates attributes against), so this page can never drift from the
// stdlib. The curated rows below serve two narrower jobs:
//   * a hand description for a generated prop the stdlib hasn't documented
//     yet (matched by name; the stdlib JSDoc wins once it exists),
//   * language-surface rows the registry can't know — bind:/on:/motion:
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
      'The platform button, already styled. tone picks the colour, variant picks how much of it you get — five variants by eight tones, and disabled/focus/press states come with it.',
    variants: ['default', 'solid', 'soft', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'how much colour it gets', type: 'default · solid · soft · outline · ghost' },
      { name: 'tone', desc: 'semantic colour token', type: TONES },
      { name: 'size', desc: 'control height', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
      { name: 'motion:press', desc: 'the press animation, one attribute', type: 'preset' },
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
      { name: 'on:sui-combobox-select', desc: 'commit event, { value, label }', type: 'event' },
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
  'radio-group': lang('radio-group', 'radioGroup', {
    blurb:
      'Groups native radios under one accessible name and one value. The platform owns exclusivity and keyboard; the group syncs ARIA and emits one change.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value', desc: 'the selected value, reflected', type: 'string' },
      { name: 'label', desc: 'accessible group name', type: 'string' },
      { name: 'name', desc: 'the native radio name', type: 'string' },
      { name: 'tone', desc: 'accent colour token', type: TONES },
      { name: 'size', desc: 'control scale', type: 'sm · md · lg' },
      { name: 'disabled / required', desc: 'platform states, styled', type: 'bool' },
    ],
  }),
  switch: lang('switch', 'switchToggle', {
    blurb:
      'A checkbox in a switch costume — role, keyboard and forms come from the platform. bind:checked keeps it and a signal in lockstep.',
    variants: ['md', 'sm', 'lg'],
    variantAttr: 'size',
    props: [
      { name: 'bind:checked', desc: 'two-way signal binding', type: 'signal' },
      { name: 'checked', desc: 'initial state', type: 'bool' },
      { name: 'tone', desc: 'track colour when on', type: TONES },
      { name: 'size', desc: 'switch scale', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
    ],
  }),
  slider: lang('slider', 'slider', {
    blurb:
      'A native range input, skinned. The platform owns drag, keyboard, role=slider and form submission; the language adds the filled track and the theme.',
    variants: ['primary', 'success', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'bind:value', desc: 'two-way signal binding', type: 'signal' },
      { name: 'min / max / step', desc: 'the platform range model', type: 'number' },
      { name: 'tone', desc: 'track and thumb colour', type: TONES },
      { name: 'size', desc: 'control scale', type: 'sm · md · lg' },
      { name: 'disabled', desc: 'the platform state, styled', type: 'bool' },
    ],
  }),
  'toggle-group': lang('toggle-group', 'toggleGroup', {
    blurb:
      'A row of toggles with one value (or many, with type=multiple). Roving focus, aria-pressed and the segmented look are all in the element.',
    variants: ['outline', 'ghost', 'segmented'],
    variantAttr: 'variant',
    props: [
      { name: 'type', desc: 'one value or many', type: 'single · multiple' },
      { name: 'value', desc: 'the selected value(s), reflected', type: 'string' },
      { name: 'variant', desc: 'the look of the group', type: 'outline · ghost · segmented' },
      { name: 'size', desc: 'control scale', type: 'sm · md · lg' },
      { name: 'orientation', desc: 'stack it vertically', type: 'horizontal · vertical' },
      { name: '<toggle value>', desc: 'the options', type: 'children' },
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
      'A real <label> — for= lands on the native element, so clicking it focuses the control. required renders the marker; size matches the control scale.',
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    props: [
      { name: 'for', desc: 'the native association — click to focus', type: 'id' },
      { name: 'required', desc: 'renders the required marker', type: 'bool' },
      { name: 'size', desc: 'label scale', type: 'sm · md · lg' },
    ],
  }),
  fieldset: lang('fieldset', 'fieldset', {
    blurb:
      'The platform fieldset with a themed border and legend. Grouping, disabling a whole set, and form semantics are all native.',
    variants: [],
    variantAttr: '',
    props: [
      { name: '<legend>', desc: 'the group title', type: 'children' },
      { name: 'disabled', desc: 'disables every control inside — platform', type: 'bool' },
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
      { name: 'ship:theme:change', desc: 'announced on selection', type: 'event' },
    ],
  }),

  // ── Feedback ─────────────────────────────────────────────────────────
  alert: lang('alert', 'alert', {
    blurb:
      'An inline announcement with the right role for assistive tech. Tone sets the icon and colour; dismissible adds the close affordance.',
    variants: ['info', 'success', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'tone', desc: 'semantic colour + icon', type: 'default · info · success · warning · destructive · muted' },
      { name: 'dismissible', desc: 'adds the close button', type: 'bool' },
      { name: 'motion:exit', desc: 'held in the DOM until the exit ends', type: 'preset' },
    ],
  }),
  progress: lang('progress', 'progressBar', {
    blurb:
      'The native progress element, themed. Give it value/max and the platform announces it; leave value off for the indeterminate sweep.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value / max', desc: 'the platform progress model', type: 'number' },
      { name: '(no value)', desc: 'indeterminate — the platform sweep', type: 'state' },
    ],
  }),
  meter: lang('meter', 'meter', {
    blurb:
      'The native meter — a measurement, not progress. low/high/optimum drive the platform’s good/warn colours, mapped onto theme tokens.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'value / min / max', desc: 'the measurement', type: 'number' },
      { name: 'low / high / optimum', desc: 'platform thresholds → tone colours', type: 'number' },
    ],
  }),
  spinner: lang('spinner', 'spinner', {
    blurb: 'An indeterminate wait. Size and tone are attributes; centered parks it in the middle of its container.',
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    props: [
      { name: 'size', desc: 'spinner scale', type: 'xs · sm · md · lg' },
      { name: 'tone', desc: 'ring colour', type: 'default · primary · info · success · warning · destructive · accent' },
      { name: 'centered', desc: 'centers itself in the parent', type: 'bool' },
    ],
  }),
  skeleton: lang('skeleton', 'skeleton', {
    blurb:
      'An attribute on a span: skeleton plus a shape draws the shimmer placeholder. No component to import — the loading state is markup.',
    variants: ['text', 'block', 'circle'],
    variantAttr: 'shape',
    props: [
      { name: 'skeleton', desc: 'the attribute that makes a span one', type: 'attr' },
      { name: 'shape', desc: 'what it stands in for', type: 'text · line · block · circle' },
      { name: 'size', desc: 'text-row height', type: 'sm · md · lg' },
    ],
  }),
  'empty-state': lang('empty-state', 'emptyState', {
    blurb:
      'The nothing-here surface: icon slot, title slot, body — sized and toned by attributes so every list, table and panel empties the same way.',
    variants: ['default', 'inline', 'placeholder', 'connecting'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'the container it empties', type: 'default · inline · placeholder · connecting' },
      { name: 'size', desc: 'padding scale', type: 'sm · md · lg' },
      { name: 'tone', desc: 'accent colour token', type: TONES },
      { name: 'slot="title"', desc: 'the headline', type: 'slot' },
    ],
  }),
  'error-state': lang('error-state', 'errorState', {
    blurb:
      'The something-broke surface — destructive-toned sibling of empty-state, with a code line for the part a human pastes into a bug report.',
    variants: ['default', 'inline'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'block or inline', type: 'default · inline' },
      { name: 'size', desc: 'padding scale', type: 'sm · md · lg' },
      { name: 'code', desc: 'the machine-readable line', type: 'string' },
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
      { name: 'sui-error-caught', desc: 'the real error, for telemetry', type: 'event' },
    ],
  }),
  toast: lang('toast', 'toast', {
    blurb:
      'A transient announcement. It auto-dismisses after duration ms (0 = sticky), pauses while hovered or focused, and lives in a <toast-region> stack.',
    variants: ['success', 'info', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'tone', desc: 'semantic colour token', type: TONES },
      { name: 'duration', desc: 'ms until dismiss — 0 is sticky', type: 'number' },
      { name: 'data-toast-close', desc: 'marks the close button', type: 'attr' },
      { name: '<toast-region position>', desc: 'the fixed stack it lives in', type: 'top/bottom-start/end' },
    ],
  }),

  // ── Display ──────────────────────────────────────────────────────────
  badge: lang('badge', 'badge', {
    blurb:
      'A small statement of state. Soft by default, solid or outline when it must shout or whisper — all eight tones, sized to sit inside a table cell.',
    variants: ['soft', 'solid', 'outline'],
    variantAttr: 'variant',
    props: [
      { name: 'tone', desc: 'semantic colour token', type: TONES },
      { name: 'variant', desc: 'fill strength', type: 'solid · soft · outline' },
      { name: 'size', desc: 'badge scale', type: 'sm · md · lg' },
      { name: 'pill', desc: 'fully rounded — counters', type: 'bool' },
    ],
  }),
  chip: lang('chip', 'chip', {
    blurb:
      'An interactive filter pill — the element itself is the button (role, keyboard, aria-pressed). removable adds the ×, and Backspace fires sui-remove.',
    variants: ['soft', 'solid', 'outline', 'ghost'],
    variantAttr: 'variant',
    props: [
      { name: 'tone', desc: 'semantic colour token', type: TONES },
      { name: 'variant', desc: 'fill strength', type: 'solid · soft · outline · ghost' },
      { name: 'selected', desc: 'pressed state, ARIA-synced', type: 'bool' },
      { name: 'removable', desc: 'the × button + Backspace/Delete', type: 'bool' },
      { name: 'on:sui-remove', desc: 'fired by × and the keyboard', type: 'event' },
    ],
  }),
  'status-dot': lang('status-dot', 'statusDot', {
    blurb: 'The smallest possible statement: a toned dot. Pair it with text; the tone is the message.',
    variants: ['success', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [{ name: 'tone', desc: 'semantic colour token', type: TONES }],
  }),
  card: lang('card', 'card', {
    blurb:
      'The surface everything else sits on. Padding, tone, an accent bar and hover motion are attributes — the card never needs a stylesheet.',
    variants: ['default', 'outline'],
    variantAttr: 'variant',
    props: [
      { name: 'tone', desc: 'tint the surface', type: TONES },
      { name: 'variant', desc: 'filled or transparent', type: 'default · outline' },
      { name: 'padding', desc: 'inset scale', type: 'none · sm · md · lg' },
      { name: 'accent-bar', desc: 'a tone-coloured leading bar', type: 'off · hover · static' },
      { name: 'motion:hover', desc: 'lift on hover, one attribute', type: 'preset' },
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
      'A sizing and ink wrapper around a slotted SVG. currentColor inheritance does the colouring; spin reuses the spinner’s keyframe.',
    variants: ['sm', 'md', 'lg', 'xl'],
    variantAttr: 'size',
    props: [
      { name: '<svg>', desc: 'the glyph — slotted, inked by currentColor', type: 'children' },
      { name: 'size', desc: 'glyph scale', type: 'xs · sm · md · lg · xl' },
      { name: 'spin', desc: 'rotate it — loading glyphs', type: 'bool' },
    ],
  }),
  separator: lang('separator', 'separator', {
    blurb: 'A themed rule with a role. Horizontal by default, vertical between inline things; tone and dashed when it must speak.',
    variants: ['solid', 'dashed'],
    variantAttr: 'variant',
    props: [
      { name: 'orientation', desc: 'which axis it divides', type: 'horizontal · vertical' },
      { name: 'tone', desc: 'line colour', type: 'default · muted · primary · destructive' },
      { name: 'variant', desc: 'line style', type: 'solid · dashed' },
    ],
  }),
  hr: lang('hr', 'hrRule', {
    blurb:
      'The platform’s own rule, themed — the zero-JS sibling of <separator> for prose and markdown-rendered content.',
    variants: ['default', 'muted', 'primary', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'tone', desc: 'line colour', type: 'default · muted · primary · destructive' },
      { name: 'variant', desc: 'line style', type: 'solid · dashed' },
      { name: 'orientation', desc: 'vertical, between inline things', type: 'horizontal · vertical' },
    ],
  }),
  details: lang('details', 'detailsEl', {
    blurb:
      'Native disclosure — <details>/<summary> with the theme’s chevron, header states and open/close animation. Zero JavaScript.',
    variants: [],
    variantAttr: '',
    props: [
      { name: '<summary>', desc: 'the always-visible line', type: 'children' },
      { name: 'open', desc: 'the platform state, reflected', type: 'bool' },
    ],
  }),
  collapsible: lang('collapsible', 'collapsible', {
    blurb:
      'Details with manners: a trigger slot, animated open/close, and it remembers its state. The platform still owns the semantics underneath.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'slot="trigger"', desc: 'the always-visible line', type: 'slot' },
      { name: 'open', desc: 'initial state, reflected', type: 'bool' },
      { name: 'disabled', desc: 'locks it — details has no native disabled', type: 'bool' },
    ],
  }),
  prose: lang('prose', 'prose', {
    heading: 'prose',
    blurb:
      'Not a tag — a promise. Plain h1/p/blockquote/code get taste from the framework stylesheet, so text never needs sx or a styling escape hatch.',
    variants: [],
    variantAttr: '',
    props: [
      { name: 'h1 … h4, p', desc: 'the type scale, already set', type: 'tags' },
      { name: 'blockquote · ul · ol · dl', desc: 'structured text, styled', type: 'tags' },
      { name: 'code · mark · abbr · small', desc: 'inline semantics, styled', type: 'tags' },
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
    variants: ['modal', 'sheet'],
    variantAttr: 'open as',
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
      'Tabs with the full keyboard suite — arrows rove, Home/End jump, panels wire their ARIA automatically. Three looks via one attribute.',
    variants: ['line', 'pill', 'enclosed'],
    variantAttr: 'variant',
    props: [
      { name: 'variant', desc: 'the look of the tab list', type: 'line · pill · enclosed' },
      { name: 'active', desc: 'initially selected tab', type: 'string' },
      { name: 'size', desc: 'tab height and text', type: 'sm · md · lg' },
      { name: 'tone', desc: 'accent colour token', type: TONES },
      { name: 'orientation', desc: 'stack the list vertically', type: 'horizontal · vertical' },
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
      'A nav of <page> controls. value marks the current one with aria-current; clicking emits sui-page-change — your list, your fetch.',
    variants: ['sm', 'md', 'lg'],
    variantAttr: 'size',
    props: [
      { name: 'value', desc: 'the current page', type: 'string' },
      { name: '<page value>', desc: 'each control — prev/next are pages too', type: 'children' },
      { name: 'size', desc: 'control scale', type: 'sm · md · lg' },
      { name: 'on:sui-page-change', desc: 'navigation event, { value }', type: 'event' },
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
  // Actions live in the action table, not in this element detail
  // catalog. (`spotlight` / `targetCursor` were here when there was
  // an `fx` pack; both packs and entries are gone now.)
}

export const DETAIL_TAGS = Object.keys(DETAILS)
