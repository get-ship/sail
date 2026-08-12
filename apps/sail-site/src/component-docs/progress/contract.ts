import type { SurfaceContract } from '../contract-types'

/**
 * Surface contract for progress.
 * Indeterminate = omit value (no separate host prop) — listed under demoOnly.
 */
export const contract = {
  slug: 'progress',
  tag: 'progress',
  kind: 'host',
  axes: {
    value: { default: '', control: 'number' },
    max: { default: 1, control: 'number', docsSkip: true },
    size: {
      values: ['sm', 'md', 'lg'] as const,
      default: 'md',
      control: 'slider',
    },
  },
  /** Docs configurator switch: omit value for native indeterminate sweep. */
  demoOnly: ['indeterminate'] as const,
  faces: [
    { path: 'component.progress.track', label: 'Track' },
    { path: 'component.progress.fill', label: 'Fill' },
  ],
  visual: {
    defaultRow: true,
    cross: ['value', 'size'],
  },
  examples: true,
} as const satisfies SurfaceContract
