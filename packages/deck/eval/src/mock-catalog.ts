// Hand-written placeholder for the primitive catalog. Replace with the
// CEM emission from Phase 4 once that lands; the shape stays the same.

import type { PrimitiveCatalog } from './catalog'

export const mockCatalog: PrimitiveCatalog = {
  version: 'mock-1',
  primitives: [
    {
      tag: 'ship-button',
      summary: 'Standard interactive button.',
      props: [
        { name: 'tone', type: 'union', values: ['neutral', 'primary', 'danger'], default: 'neutral' },
        { name: 'variant', type: 'union', values: ['solid', 'soft', 'ghost'], default: 'solid' },
        { name: 'size', type: 'union', values: ['sm', 'md', 'lg'], default: 'md' },
        { name: 'disabled', type: 'bool', default: false },
        { name: 'loading', type: 'bool', default: false },
      ],
      slots: [
        { name: 'default', summary: 'Button label.' },
        { name: 'leading', summary: 'Icon or content rendered before the label.' },
        { name: 'trailing', summary: 'Icon or content rendered after the label.' },
      ],
      events: [
        { name: 'click', summary: 'Fires on activation. Suppressed when disabled or loading.' },
      ],
    },
    {
      tag: 'ship-input',
      summary: 'Text input with two-way binding support.',
      props: [
        { name: 'value', type: 'string', default: '', bindable: true },
        { name: 'placeholder', type: 'string', default: '' },
        { name: 'type', type: 'union', values: ['text', 'email', 'password', 'search', 'number'], default: 'text' },
        { name: 'disabled', type: 'bool', default: false },
      ],
      slots: [],
      events: [
        { name: 'input', summary: 'Fires on every keystroke; carries the new value.' },
      ],
    },
    {
      tag: 'ship-spinner',
      summary: 'Indeterminate loading indicator.',
      props: [],
      slots: [],
      events: [],
    },
    {
      tag: 'ship-empty-state',
      summary: 'Empty-state placeholder with a graphic, title, and optional action.',
      props: [
        { name: 'icon', type: 'string', default: '' },
      ],
      slots: [
        { name: 'title', summary: 'Headline copy.' },
        { name: 'action', summary: 'Optional call-to-action — typically a ship-button.' },
      ],
      events: [],
    },
    {
      tag: 'ship-list-tile',
      summary: 'Single row in a list — title, optional metadata, optional chevron.',
      props: [
        { name: 'variant', type: 'union', values: ['default', 'subtle'], default: 'default' },
        { name: 'density', type: 'union', values: ['compact', 'comfortable'], default: 'comfortable' },
        { name: 'interactive', type: 'bool', default: false },
        { name: 'navigable', type: 'bool', default: false },
      ],
      slots: [
        { name: 'default', summary: 'Tile body.' },
      ],
      events: [],
    },
  ],
}
