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
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const,
      default: 'primary',
      control: 'swatches',
    },
    size: {
      values: ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const,
      default: 'md',
      control: 'slider',
    },
    disabled: { default: false, control: 'switch' },
    loading: { default: false, control: 'switch' },
    pill: { default: false, control: 'switch' },
  },
  faces: [
    { path: 'component.button.bg', label: 'Fill' },
    { path: 'component.button.fg', label: 'Ink' },
    { path: 'component.button.border', label: 'Border' },
    { path: 'component.button.hover-bg', label: 'Hover fill' },
  ],
  visual: {
    defaultRow: true,
    cross: ['variant', 'tone', 'size', 'disabled'],
  },
  examples: true,
} as const satisfies SurfaceContract
