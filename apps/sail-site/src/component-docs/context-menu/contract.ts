import type { SurfaceContract } from '../contract-types'

/** Surface contract for context-menu — for string + onSelect event. */
export const contract = {
  slug: 'context-menu',
  tag: 'context-menu',
  kind: 'host',
  axes: {
    for: { default: '', control: 'text', docsSkip: true },
    onSelect: { control: 'none', event: true },
  },
  faces: [
    { path: 'popover', label: 'Background' },
    { path: 'foreground', label: 'Foreground' },
    { path: 'border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
