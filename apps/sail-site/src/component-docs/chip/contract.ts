import type { SurfaceContract } from '../contract-types'

/** Surface contract for chip — axes from prop-registry only. */
export const contract = {
  slug: 'chip',
  tag: 'chip',
  kind: 'host',
  axes: {
    variant: {
      values: ['solid', 'soft', 'outline', 'ghost'] as const,
      default: 'solid',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const,
      default: 'primary',
      control: 'swatches',
    },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
    selected: { default: false, control: 'switch' },
    removable: { default: false, control: 'switch' },
    // host props complete; visual row exercises disabled. Docs knob deferred (docsSkip).
    disabled: { default: false, control: 'switch', docsSkip: true },
    onRemove: { control: 'none', event: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    // disabled stays on visual.cross for harness coverage; docsSkip only skips docs knob.
    cross: ['variant', 'tone', 'size', 'selected', 'removable', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
