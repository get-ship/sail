// <sui-prop-knobs> showcase fixtures — real CEM `attributes[]` arrays the knob
// panel classifies into the four control kinds. The Studio components explorer
// / docs read these.
//
// `sui-prop-knobs` consumes its CEM array via `prop:props` IDENTITY, NOT scalar
// attributes (`ComponentExample.props` is scalar-only — verified). So each
// scenario pairs an (empty) scalar `props` row with the CEM `attributes[]` it
// classifies, exported as `propSets` keyed by scenario name. A host wires
// `prop:props={propSets[name]}` (+ optional `prop:value={…}` controlled) to
// bring the panel alive.

import type { ComponentExample } from '@ship/sui-runtime'
import type { CemAttr } from './classify'

/** CEM `attributes[]` arrays, keyed by the matching example `name`. The host
 *  injects these via `prop:props` (identity) — they can't ride a scalar
 *  attribute. Drawn from real Ship-primitive declarations; no placeholders. */
export const propSets: Record<string, CemAttr[]> = {
  // A full four-kind sweep — one enum, one bool, one number, one string — so the
  // panel demonstrates all classification branches at once.
  'All four kinds': [
    {
      name: 'variant',
      type: { text: "'outline' | 'ghost' | 'segmented'" },
      default: 'outline',
      values: ['outline', 'ghost', 'segmented'],
    },
    { name: 'disabled', type: { text: 'boolean' }, default: 'false' },
    { name: 'max', type: { text: 'number' }, default: '100' },
    { name: 'label', type: { text: 'string' }, default: '' },
  ],
  // sui-button-shaped enums — variant/tone/size — exercising the size-enum
  // largest-default seeding rule.
  'Button enums': [
    {
      name: 'variant',
      type: { text: "'solid' | 'outline' | 'ghost'" },
      default: 'solid',
      values: ['solid', 'outline', 'ghost'],
    },
    {
      name: 'tone',
      type: { text: "'default' | 'primary' | 'destructive'" },
      default: 'default',
      values: ['default', 'primary', 'destructive'],
    },
    { name: 'size', type: { text: "'sm' | 'md' | 'lg'" }, values: ['sm', 'md', 'lg'] },
  ],
  // A bool-heavy form control — switch/checkbox-style flags only.
  'Boolean flags': [
    { name: 'checked', type: { text: 'boolean' }, default: 'false' },
    { name: 'required', type: { text: 'boolean' }, default: 'false' },
    { name: 'readonly', type: { text: 'boolean' }, default: 'false' },
  ],
}

export const examples: ComponentExample[] = [
  {
    name: 'All four kinds',
    description: 'One enum, bool, number, and string prop — the full classification sweep.',
  },
  {
    name: 'Button enums',
    description: 'variant / tone / size enums; the size knob seeds its largest legible variant.',
  },
  {
    name: 'Boolean flags',
    description: 'A switch-per-flag panel — bool classification only.',
  },
]
