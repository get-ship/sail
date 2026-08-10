/**
 * Index of surface contracts for first-class (and contracted) docs packages.
 * Checker + tooling import from here.
 */
import { contract as alert } from './alert/contract'
import { contract as badge } from './badge/contract'
import { contract as button } from './button/contract'
import { contract as chip } from './chip/contract'
import { contract as fieldset } from './fieldset/contract'
import { contract as icon } from './icon/contract'
import { contract as meter } from './meter/contract'
import { contract as progress } from './progress/contract'
import { contract as separator } from './separator/contract'
import { contract as skeleton } from './skeleton/contract'
import { contract as spinner } from './spinner/contract'
import type { SurfaceContract } from './contract-types'

export const CONTRACTS: readonly SurfaceContract[] = [
  button,
  badge,
  fieldset,
  chip,
  alert,
  separator,
  spinner,
  skeleton,
  icon,
  progress,
  meter,
]

export const CONTRACT_BY_SLUG: Record<string, SurfaceContract> = Object.fromEntries(
  CONTRACTS.map((c) => [c.slug, c]),
)

export function contractFor(slug: string): SurfaceContract | null {
  return CONTRACT_BY_SLUG[slug] ?? null
}
