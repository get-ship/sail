import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for accordion — axes from prop-registry only.
 * Faces: component.accordion.bg · border · chevron · fg · hover-bg.
 */
export const contract = {
  slug: 'accordion',
  tag: 'accordion',
  kind: 'host',
  axes: {
    open: { default: false, control: 'switch' },
    disabled: { default: false, control: 'switch' },
    name: { default: '', control: 'text' },
    multiple: { default: false, control: 'switch' },
    onToggle: { control: 'none', event: true },
  },
  faces: [
    { path: 'component.accordion.bg', label: 'Fill' },
    { path: 'component.accordion.fg', label: 'Ink' },
    { path: 'component.accordion.border', label: 'Border' },
    { path: 'component.accordion.hover-bg', label: 'Hover fill' },
    { path: 'component.accordion.chevron', label: 'Chevron' },
  ],
  visual: {
    defaultRow: true,
    cross: ['open', 'disabled', 'name', 'multiple'],
  },
  examples: true,
} as const satisfies SurfaceContract
