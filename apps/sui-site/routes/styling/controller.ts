// /styling controller. The source samples live here (not inline in the
// template) so the compiler's scope scanner never treats the sample strings
// as real markup to bind.

import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const code = {
  // sx — the primary styling surface. Token utilities, right on the element.
  sx: [
    '<prop name="title" type="string" default="Card" />',
    '',
    '<column sx="p:4 gap:1 radius:lg bg:surface-1 border:border">',
    '  <h3>{title()}</h3>',
    '  <p sx="fg:muted-foreground"><slot></slot></p>',
    '</column>',
  ].join('\n'),
  // <host sx> — style the component's own element.
  host: [
    "<!-- <host sx> styles the component element itself. -->",
    '<host sx="radius:lg overflow:hidden bg:surface-1 border:border" />',
    '',
    '<row sx="py:3 px:4 bg:surface-2">',
    '  <slot name="header"></slot>',
    '</row>',
    '<column sx="p:4 gap:2">',
    '  <slot></slot>',
    '</column>',
  ].join('\n'),
  // <variants> — state-driven styling, keyed off the host's attributes.
  variants: [
    '<!-- A variant repaints an element when the host has an attribute. -->',
    '<variants class="trigger">',
    '  [open]:     bg:muted',
    '  [disabled]: fg:muted-foreground',
    '</variants>',
    '',
    '<button class="trigger" sx="w:full py:3 px:4 text:sm weight:semibold">',
    '  <slot name="trigger"></slot>',
    '</button>',
  ].join('\n'),
  // Tokens, not literals — sx is a closed, token-only vocabulary.
  tokens: [
    '<!-- sx is a closed vocabulary: every value is a token. -->',
    '<!-- There is no sx="p:16px" and no sx="bg:#ffffff" —      -->',
    '<!-- only design tokens resolve, so every theme just works. -->',
    '',
    '<column sx="p:4 gap:2 radius:md bg:surface-1 border:border">',
    '  <h3>{title()}</h3>',
    '  <p sx="fg:muted-foreground"><slot></slot></p>',
    '</column>',
    '',
    '<!-- Pick a token by INTENT, across the four layers:      -->',
    '<!--   bg:surface-1         semantic surface              -->',
    '<!--   fg:muted-foreground  brand palette role            -->',
    '<!--   radius:md            primitive from the scale       -->',
  ].join('\n'),
  // Status tones, not palette colors.
  tone: [
    '<!-- Tone ramps are status surfaces, not brand colors. -->',
    '<alert tone="success">Saved.</alert>',
    '<alert tone="warning">Quota at 80%.</alert>',
    '<alert tone="destructive">Save failed.</alert>',
    '<alert tone="info">Tip: press <kbd>⌘ K</kbd> for the palette.</alert>',
  ].join('\n'),
  // Layout primitives — keywords, never raw divs.
  layout: [
    '<!-- Layout is keywords, not divs. -->',
    '<column gap="4">',
    '  <header>...</header>',
    '  <main>...</main>',
    '</column>',
    '',
    '<!-- A horizontal row that wraps. -->',
    '<row gap="3" align="center">',
    '  <button>Cancel</button>',
    '  <button primary>Save</button>',
    '</row>',
    '',
    '<!-- A responsive grid that collapses. -->',
    '<grid min="sm" gap="6">',
    '  <article>...</article>',
    '  <article>...</article>',
    '</grid>',
  ].join('\n'),
  // The escape hatch — raw <style> for what sx structurally can't express.
  escape: [
    '<!-- <style> is the escape hatch: use it only for what sx',
    '     structurally cannot express — pseudo-elements, ::slotted,',
    '     grid-template-columns, top/left, accent-color. -->',
    '<style>',
    '  .swatches {',
    '    display: grid;',
    '    grid-template-columns: repeat(6, 1fr);',
    '    gap: var(--space-2);',
    '  }',
    "  input[type='range'] { accent-color: var(--tone-ring); }",
    '  ::slotted(svg) { width: var(--icon-size-md); }',
    '</style>',
  ].join('\n'),
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ code }),
})
