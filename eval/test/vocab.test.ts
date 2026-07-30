import { describe, expect, it } from 'vitest'
import {
  classifyNode,
  computeVocabShare,
  gradeVocab,
  STUDIO_LOW_WATER,
} from '../src/graders/vocab.ts'
import type { HostTooling, TreeNode } from '../src/host.ts'

const fakeHost = (nodes: TreeNode[]): HostTooling => ({
  sailBin: '/fake/sail',
  async check() {
    return { ok: true, diagnostics: [], autoFixed: [] }
  },
  async inspectTree() {
    return { ok: true, nodes }
  },
  async layout() {
    return { ok: true, findings: [] }
  },
})

describe('vocab share', () => {
  it('classifies layers and tags', () => {
    expect(classifyNode({ tag: 'card', layer: 'l1-element' })).toBe('l1')
    expect(classifyNode({ tag: 'row', layer: 'l1-keyword' })).toBe('l1')
    expect(classifyNode({ tag: 'div', layer: 'l1-element' })).toBe('l1') // layer wins
    expect(classifyNode({ tag: 'div' })).toBe('primitive')
    expect(classifyNode({ tag: 'span' })).toBe('primitive')
    expect(classifyNode({ tag: 'sail-foo', layer: 'l2-component' })).toBe('other')
  })

  it('computes L1/(L1+primitives)', () => {
    const nodes: TreeNode[] = [
      { tag: 'column', layer: 'l1-keyword' },
      { tag: 'card', layer: 'l1-element' },
      { tag: 'div' },
      { tag: 'div' },
      { tag: 'button', layer: 'l1-element' },
    ]
    const r = computeVocabShare(nodes)
    expect(r.l1Count).toBe(3)
    expect(r.primitiveCount).toBe(2)
    expect(r.share).toBeCloseTo(0.6)
  })

  it('studio-like low share fails default min', async () => {
    // ~17% L1: 1 L1 + 5 primitives
    const nodes: TreeNode[] = [
      { tag: 'button', layer: 'l1-element' },
      { tag: 'div' },
      { tag: 'div' },
      { tag: 'div' },
      { tag: 'div' },
      { tag: 'span' },
    ]
    const r = computeVocabShare(nodes)
    expect(r.share).toBeCloseTo(STUDIO_LOW_WATER, 1)
    const grade = await gradeVocab({
      host: fakeHost(nodes),
      workdir: '/tmp',
      nodes,
      minShare: 0.25,
    })
    expect(grade.ok).toBe(false)
  })

  it('all-L1 tree passes', async () => {
    const nodes: TreeNode[] = [
      { tag: 'column', layer: 'l1-keyword' },
      { tag: 'card', layer: 'l1-element' },
      { tag: 'button', layer: 'l1-element' },
    ]
    const grade = await gradeVocab({
      host: fakeHost(nodes),
      workdir: '/tmp',
      nodes,
    })
    expect(grade.ok).toBe(true)
    expect(grade.share).toBe(1)
  })

  it('skips when sail missing and allowSkip', async () => {
    const host: HostTooling = {
      sailBin: null,
      async check() {
        return { ok: false, diagnostics: [], autoFixed: [] }
      },
      async inspectTree() {
        return { ok: false, nodes: [] }
      },
      async layout() {
        return { ok: true, findings: [] }
      },
    }
    const grade = await gradeVocab({ host, workdir: '/tmp', allowSkip: true })
    expect(grade.skipped).toBe(true)
    expect(grade.ok).toBe(true)
  })

  it('fails when sail missing and allowSkip false', async () => {
    const host: HostTooling = {
      sailBin: null,
      async check() {
        return { ok: false, diagnostics: [], autoFixed: [] }
      },
      async inspectTree() {
        return { ok: false, nodes: [] }
      },
      async layout() {
        return { ok: true, findings: [] }
      },
    }
    const grade = await gradeVocab({ host, workdir: '/tmp', allowSkip: false })
    expect(grade.ok).toBe(false)
  })
})
