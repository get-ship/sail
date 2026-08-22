import type { SurfaceContract } from '../contract-types'

/** Surface contract for switch — axes from prop-registry only. */
export const contract = {
  slug: 'switch',
  tag: 'switch',
  kind: 'host',
  axes: {
    // Specimen switch is the checked control — no duplicate On knob.
    checked: { default: false, control: 'switch', docsSkip: true },
    disabled: { default: false, control: 'switch' },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const,
      default: 'primary',
      control: 'swatches',
    },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'select',
    },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['checked', 'disabled', 'tone', 'size'],
  },
  examples: true,
} as const satisfies SurfaceContract
