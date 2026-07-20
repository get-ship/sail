import type { Action } from '../action'
import { chip as chipController } from '../element-controllers/chip'

/**
 * `use:chip` — generic mix-in form of chip keyboard/ARIA/remove wiring.
 *
 * The L1 `<chip>` element owns this behavior via its element controller
 * (`data-sail-el="chip"`); the stdlib root no longer stamps `use:chip`.
 * This action remains for non-chip hosts that want the same interaction
 * attached through the mix-in API.
 */
export const chip: Action<void> = (node) => chipController(node)
