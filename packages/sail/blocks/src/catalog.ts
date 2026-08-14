/**
 * Blocks inventory. A row here is a product slot — authored or planned.
 * Docs composition and takumi read this. Do not invent a second list.
 */

export type BlockStatus = 'authored' | 'planned'

export type BlockEntry = {
  id: string
  title: string
  intent: string
  status: BlockStatus
  /** L1 family hosts this block is meant to compose. */
  primitives: readonly string[]
}

export const BLOCKS: readonly BlockEntry[] = [
  {
    id: 'dashboard',
    title: 'Dashboard',
    intent: 'App chrome + metrics + a tabbed data surface. Defaults only.',
    status: 'authored',
    primitives: ['navbar', 'card', 'tabs', 'table', 'field', 'button', 'alert'],
  },
  {
    id: 'pricing',
    title: 'Pricing',
    intent: 'Three-tier cards, default measure, no per-card paint.',
    status: 'planned',
    primitives: ['card', 'button', 'badge', 'separator'],
  },
  {
    id: 'settings',
    title: 'Settings',
    intent: 'Fieldset groups, fields, switches. Form defaults must hold.',
    status: 'planned',
    primitives: ['fieldset', 'field', 'input', 'switch', 'button'],
  },
  {
    id: 'auth',
    title: 'Auth',
    intent: 'A single card: identity fields + primary action.',
    status: 'planned',
    primitives: ['card', 'field', 'input', 'button', 'a'],
  },
  {
    id: 'empty',
    title: 'Empty',
    intent: 'Empty-state as the page, not a decorated div.',
    status: 'planned',
    primitives: ['empty-state', 'button'],
  },
] as const

export function authoredBlocks(): BlockEntry[] {
  return BLOCKS.filter((b) => b.status === 'authored')
}
