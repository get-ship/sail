/**
 * @ship/sail-l1 — generated L1 consumer catalog.
 *
 * Not the stdlib .sail sources. Not a test package.
 */

import { FAMILIES, TAGS, TAG_BY_ID, type L1Family, type L1Tag } from './generated/catalog'
import { L1_COPY, type L1Copy } from './copy'

export {
  FAMILIES,
  TAGS,
  TAG_BY_ID,
  type L1Axis,
  type L1Family,
  type L1Prop,
  type L1Role,
  type L1Tag,
} from './generated/catalog'
export { L1_COPY, type L1Category, type L1Copy } from './copy'

/** Family hosts — docs hub tiles, Studio faces. */
export function hosts(): readonly L1Family[] {
  return FAMILIES
}

export function familyOf(tag: string): string | undefined {
  return TAG_BY_ID[tag]?.family
}

export function tagOf(tag: string): L1Tag | undefined {
  return TAG_BY_ID[tag]
}

export function copyOf(tag: string): L1Copy | undefined {
  const family = TAG_BY_ID[tag]?.family ?? tag
  return L1_COPY[tag] ?? L1_COPY[family]
}

/** Concerns a host spends — the Theme Studio overlay list. Not component.* paths. */
export function concernsOf(tag: string): readonly string[] {
  return TAG_BY_ID[tag]?.concerns ?? []
}

export function allTags(): readonly string[] {
  return TAGS.map((t) => t.tag)
}
