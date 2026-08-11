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
    { path: 'component.context-menu.bg', label: 'Fill' },
    { path: 'component.context-menu.fg', label: 'Ink' },
    { path: 'component.context-menu.border', label: 'Border' },
  ],
  visual: {
    defaultRow: true,
    cross: [],
  },
  examples: true,
} as const satisfies SurfaceContract
