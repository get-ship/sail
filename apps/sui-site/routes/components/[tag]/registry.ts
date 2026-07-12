// /components/[tag] — hand-curated detail data for the flagship components.
// tag → { blurb, variants, props, usage }. Deliberately NOT compiler-derived:
// ten components documented well beats forty-eight documented by machine.
// The usage strings live in ./snippets.ts so the site-wide snippet guard
// compiles every one of them.

import { usage } from './snippets'

export interface PropRow {
  name: string
  desc: string
  type: string
}

export interface DetailEntry {
  tag: string
  blurb: string
  /** What the variant chips switch on the live stage. Empty = no chips. */
  variants: string[]
  /** Which attribute the chips drive — shown beside the chips. */
  variantAttr: string
  props: PropRow[]
  usage: string
}

const TONES = 'default · primary · info · success · warning · destructive · accent · muted'

export const DETAILS: Record<string, DetailEntry> = {
  button: {
    tag: 'button',
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
    usage: usage.button ?? '',
  },
  combobox: {
    tag: 'combobox',
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
    usage: usage.combobox ?? '',
  },
  tabs: {
    tag: 'tabs',
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
    usage: usage.tabs ?? '',
  },
  dialog: {
    tag: 'dialog',
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
    usage: usage.dialog ?? '',
  },
  table: {
    tag: 'table',
    blurb:
      "The platform's table model — thead, tbody, tr — skinned by the theme. No grid component to learn: markup in, a designed table out.",
    variants: [],
    variantAttr: '',
    props: [
      { name: '<thead> <tbody> <tr>', desc: 'the platform table model', type: 'children' },
      { name: '{#for}', desc: 'rows render from state and stay live', type: 'block' },
      { name: 'density', desc: 'row padding comes from theme tokens', type: 'theme' },
    ],
    usage: usage.table ?? '',
  },
  switch: {
    tag: 'switch',
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
    usage: usage.switch ?? '',
  },
  slider: {
    tag: 'slider',
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
    usage: usage.slider ?? '',
  },
  badge: {
    tag: 'badge',
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
    usage: usage.badge ?? '',
  },
  card: {
    tag: 'card',
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
    usage: usage.card ?? '',
  },
  alert: {
    tag: 'alert',
    blurb:
      'An inline announcement with the right role for assistive tech. Tone sets the icon and colour; dismissible adds the close affordance.',
    variants: ['info', 'success', 'warning', 'destructive'],
    variantAttr: 'tone',
    props: [
      { name: 'tone', desc: 'semantic colour + icon', type: 'default · info · success · warning · destructive · muted' },
      { name: 'dismissible', desc: 'adds the close button', type: 'bool' },
      { name: 'motion:exit', desc: 'held in the DOM until the exit ends', type: 'preset' },
    ],
    usage: usage.alert ?? '',
  },
}

export const DETAIL_TAGS = Object.keys(DETAILS)
