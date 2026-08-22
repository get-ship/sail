import type { SurfaceContract } from '../contract-types'

/** Surface contract for button — axes from prop-registry only. */
export const contract = {
  slug: 'button',
  tag: 'button',
  kind: 'host',
  axes: {
    variant: {
      values: ['solid', 'soft', 'outline', 'ghost'] as const,
      default: 'solid',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'danger', 'accent', 'muted'] as const,
      default: 'primary',
      control: 'swatches',
    },
    size: {
      values: ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const,
      default: 'md',
      control: 'slider',
    },
    disabled: { default: false, control: 'switch' },
    // Docs configurator does not yet expose these (visual.cross is variant/tone/size/disabled).
    loading: { default: false, control: 'switch', docsSkip: true },
    pill: { default: false, control: 'switch', docsSkip: true },
  },
  faces: [],
  visual: {
    defaultRow: true,
    cross: ['variant', 'tone', 'size', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
