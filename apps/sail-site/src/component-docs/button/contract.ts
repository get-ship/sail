import type { SurfaceContract } from '../contract-types'

/** Surface contract for button — axes from prop-registry only. */
export const contract = {
  slug: 'button',
  tag: 'button',
  kind: 'host',
  axes: {
    variant: {
      values: ['default', 'solid', 'soft', 'outline', 'ghost'] as const,
      default: 'default',
      control: 'select',
    },
    tone: {
      values: ['primary', 'info', 'success', 'warning', 'destructive', 'accent', 'muted'] as const,
      optional: true,
      control: 'swatches',
    },
    size: {
      values: ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const,
      default: 'md',
      control: 'slider',
    },
    // Pilot stage emphasizes variant/tone/size; disabled is PE-complete but docsSkip until pairing expands knobs.
    disabled: { default: false, control: 'switch', docsSkip: true },
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
