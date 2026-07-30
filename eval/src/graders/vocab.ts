import type { HostTooling, TreeNode } from '../host.ts'
import type { VocabGrade } from '../types.ts'
import { collectSailFiles } from './compile.ts'

/** Default minimum share — studio failure (~17%) is the low-water reference. */
export const DEFAULT_MIN_VOCAB_SHARE = 0.25
export const STUDIO_LOW_WATER = 0.17

export interface VocabGradeOpts {
  host: HostTooling
  workdir: string
  minShare?: number
  allowSkip?: boolean
  /** Injected nodes for unit tests (bypasses host). */
  nodes?: TreeNode[]
}

/**
 * Vocabulary share = L1 / (L1 + primitives).
 * L1 = layer l1-element | l1-keyword (or known layout keywords / sail tags).
 * primitives = raw div/span (and anonymous expansion chrome).
 */
export function computeVocabShare(nodes: TreeNode[]): {
  l1Count: number
  primitiveCount: number
  share: number
} {
  let l1Count = 0
  let primitiveCount = 0
  for (const n of nodes) {
    const kind = classifyNode(n)
    if (kind === 'l1') l1Count++
    else if (kind === 'primitive') primitiveCount++
    // pack / l2 / userland counted neither side of the L1-vs-primitive ratio
  }
  const denom = l1Count + primitiveCount
  const share = denom === 0 ? 0 : l1Count / denom
  return { l1Count, primitiveCount, share }
}

export function classifyNode(n: TreeNode): 'l1' | 'primitive' | 'other' {
  const layer = (n.layer ?? '').toLowerCase()
  const tag = (n.tag ?? '').toLowerCase()

  if (layer === 'l1-element' || layer === 'l1-keyword') return 'l1'
  if (layer === 'pack' || layer === 'l2-component' || layer === 'userland') return 'other'

  // Fallback when layer missing (static tree parse / fixtures)
  if (
    ['column', 'row', 'grid', 'center', 'stack', 'card', 'button', 'input', 'label', 'badge', 'field', 'navbar', 'table', 'tabs', 'textarea', 'dialog', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'a', 'ul', 'ol', 'li', 'img', 'form', 'section', 'header', 'footer', 'main', 'nav', 'aside', 'article'].includes(
      tag,
    )
  ) {
    return 'l1'
  }
  if (tag === 'div' || tag === 'span') return 'primitive'
  if (tag.includes('-') && !tag.startsWith('sail-')) return 'other'
  if (tag.startsWith('sail-')) return 'other'
  return 'l1'
}

export async function gradeVocab(opts: VocabGradeOpts): Promise<VocabGrade> {
  const minShare = opts.minShare ?? DEFAULT_MIN_VOCAB_SHARE
  const allowSkip = opts.allowSkip ?? false

  let nodes = opts.nodes
  if (!nodes) {
    if (!opts.host.sailBin) {
      if (allowSkip) {
        return {
          ok: true,
          l1Count: 0,
          primitiveCount: 0,
          share: 0,
          minShare,
          skipped: true,
          detail: 'sail binary not available — vocab grade skipped',
        }
      }
      return {
        ok: false,
        l1Count: 0,
        primitiveCount: 0,
        share: 0,
        minShare,
        detail: 'sail binary not available',
      }
    }
    const files = collectSailFiles(opts.workdir)
    if (files.length === 0) {
      return {
        ok: false,
        l1Count: 0,
        primitiveCount: 0,
        share: 0,
        minShare,
        detail: 'no .sail files for vocab inspection',
      }
    }
    nodes = []
    for (const f of files) {
      const tree = await opts.host.inspectTree(f)
      if (tree.error && tree.nodes.length === 0) {
        return {
          ok: false,
          l1Count: 0,
          primitiveCount: 0,
          share: 0,
          minShare,
          detail: tree.error,
        }
      }
      nodes.push(...tree.nodes)
    }
  }

  const { l1Count, primitiveCount, share } = computeVocabShare(nodes)
  const ok = share >= minShare || (l1Count > 0 && primitiveCount === 0)
  return {
    ok,
    l1Count,
    primitiveCount,
    share,
    minShare,
    detail:
      share < STUDIO_LOW_WATER
        ? `share ${share.toFixed(3)} below studio low-water (${STUDIO_LOW_WATER})`
        : undefined,
  }
}
