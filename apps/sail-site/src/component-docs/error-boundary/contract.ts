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
    { path: 'component.error-boundary.bg', label: 'Fill' },
    { path: 'component.error-boundary.border', label: 'Border' },
    { path: 'component.error-boundary.fg', label: 'Ink' },
    { path: 'component.error-boundary.icon-fg', label: 'Icon' },
    { path: 'component.error-boundary.message-fg', label: 'Message' },
    { path: 'component.error-boundary.retry-bg', label: 'Retry fill' },
    { path: 'component.error-boundary.retry-fg', label: 'Retry ink' },
  ],
  visual: {
    defaultRow: true,
    cross: ['message'],
  },
  examples: true,
} as const satisfies SurfaceContract
