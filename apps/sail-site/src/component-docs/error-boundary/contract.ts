import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for error-boundary — axes from prop-registry only.
 * onErrorReset / onErrorCaught are events (no docs knobs).
 */
export const contract = {
  slug: 'error-boundary',
  tag: 'error-boundary',
  kind: 'host',
  axes: {
    message: { default: '', control: 'text' },
    onErrorReset: { control: 'none', event: true },
    onErrorCaught: { control: 'none', event: true },
  },
  faces: [
    { path: 'card', label: 'Background' },
    { path: 'card-foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: ['message'],
  },
  examples: true,
} as const satisfies SurfaceContract
