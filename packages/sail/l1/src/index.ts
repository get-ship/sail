/**
 * @ship/sail-l1 — generated L1 consumer catalog.
 *
 * Not the stdlib .sail sources. Not a test package.
 */

import { COMPONENTS, TAGS, TAG_BY_ID, type L1Component, type L1Tag } from './generated/catalog'
import { L1_COPY, type L1Copy } from './copy'

export {
  COMPONENTS,
  TAGS,
  TAG_BY_ID,
  type L1Axis,
  type L1Component,
  type L1Prop,
  type L1Role,
  type L1Tag,
} from './generated/catalog'
export { L1_COPY, type L1Category, type L1Copy } from './copy'

/** Component hosts — docs hub tiles, Studio faces. */
export function hosts(): readonly L1Component[] {
  return COMPONENTS
}

export function componentOf(tag: string): string | undefined {
  return TAG_BY_ID[tag]?.component
}

export function tagOf(tag: string): L1Tag | undefined {
  return TAG_BY_ID[tag]
}

export function copyOf(tag: string): L1Copy | undefined {
  const component = TAG_BY_ID[tag]?.component ?? tag
  return L1_COPY[tag] ?? L1_COPY[component]
}

/** Concerns a host spends — the Theme Studio overlay list. Not component.* paths. */
export function concernsOf(tag: string): readonly string[] {
  return TAG_BY_ID[tag]?.concerns ?? []
}

export function allTags(): readonly string[] {
  return TAGS.map((t) => t.tag)
}
